use serde::{Deserialize, Serialize};

use super::{build_system_prompt, http_client, AiError, AiProvider, CommitContext, ProviderConfig};

pub struct OpenAiCompatibleProvider {
    display_name: String,
    api_key: String,
    model: String,
    endpoint: String,
    requires_auth: bool,
}

impl OpenAiCompatibleProvider {
    pub fn openai(config: ProviderConfig) -> Self {
        Self {
            display_name: "OpenAI".to_string(),
            api_key: config.api_key,
            model: if config.model.is_empty() {
                "gpt-4o-mini".to_string()
            } else {
                config.model
            },
            endpoint: "https://api.openai.com/v1/chat/completions".to_string(),
            requires_auth: true,
        }
    }

    pub fn ollama(config: ProviderConfig) -> Self {
        let base = config
            .endpoint
            .unwrap_or_else(|| "http://localhost:11434".to_string());
        let endpoint = if base.ends_with("/v1/chat/completions") {
            base
        } else {
            format!("{}/v1/chat/completions", base.trim_end_matches('/'))
        };

        Self {
            display_name: "Ollama".to_string(),
            api_key: config.api_key,
            model: if config.model.is_empty() {
                "llama3".to_string()
            } else {
                config.model
            },
            endpoint,
            requires_auth: false,
        }
    }
}

// ── OpenAI API Types ─────────────────────────────────

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
}

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Option<Vec<ChatChoice>>,
    error: Option<ChatErrorBody>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: Option<ChatResponseMessage>,
}

#[derive(Deserialize)]
struct ChatResponseMessage {
    content: Option<String>,
}

#[derive(Deserialize)]
struct ChatErrorBody {
    message: Option<String>,
}

// ── AiProvider impl ──────────────────────────────────

#[async_trait::async_trait]
impl AiProvider for OpenAiCompatibleProvider {
    fn name(&self) -> &str {
        &self.display_name
    }

    async fn generate(&self, context: &CommitContext) -> Result<String, AiError> {
        if self.requires_auth && self.api_key.is_empty() {
            return Err(AiError::NoApiKey);
        }

        let system_prompt = build_system_prompt(context);
        let user_message = format!(
            "Based on the following staged changes, generate a commit message:\n\n{}",
            context.diff_summary
        );

        let request = ChatRequest {
            model: self.model.clone(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: system_prompt,
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: user_message,
                },
            ],
            temperature: 0.3,
        };

        let mut req_builder = http_client().post(&self.endpoint).json(&request);

        if self.requires_auth && !self.api_key.is_empty() {
            req_builder = req_builder.bearer_auth(&self.api_key);
        }

        // Retry once on timeout or 5xx
        let response = match req_builder.try_clone() {
            Some(retry_builder) => {
                let first = http_client()
                    .post(&self.endpoint)
                    .json(&request)
                    .bearer_auth_if(self.requires_auth, &self.api_key)
                    .send()
                    .await;

                match first {
                    Ok(resp) if resp.status().is_server_error() => {
                        tracing::warn!("AI provider {} returned 5xx, retrying", self.display_name);
                        retry_builder.send().await?
                    }
                    Err(e) if e.is_timeout() => {
                        tracing::warn!("AI provider {} timed out, retrying", self.display_name);
                        retry_builder.send().await?
                    }
                    Ok(resp) => resp,
                    Err(e) => return Err(e.into()),
                }
            }
            None => req_builder.send().await?,
        };

        let status = response.status();
        if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
            return Err(AiError::Auth("API Key 無效，請確認設定".to_string()));
        }
        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(AiError::RateLimit);
        }
        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(AiError::ModelNotFound(self.model.clone()));
        }
        if status.is_server_error() {
            return Err(AiError::ServerError(format!("HTTP {}", status)));
        }

        let body = response.text().await?;
        let parsed: ChatResponse =
            serde_json::from_str(&body).map_err(|e| AiError::Parse(e.to_string()))?;

        // Check for API error in body
        if let Some(err) = parsed.error {
            let msg = err.message.unwrap_or_else(|| "Unknown error".to_string());
            return Err(AiError::ServerError(msg));
        }

        let message = parsed
            .choices
            .and_then(|c| c.into_iter().next())
            .and_then(|c| c.message)
            .and_then(|m| m.content)
            .map(|s| s.trim().to_string())
            .unwrap_or_default();

        if message.is_empty() {
            return Err(AiError::EmptyResponse);
        }

        Ok(message)
    }
}

// ── Helper trait for conditional auth ────────────────

trait RequestBuilderExt {
    fn bearer_auth_if(self, condition: bool, token: &str) -> Self;
}

impl RequestBuilderExt for reqwest::RequestBuilder {
    fn bearer_auth_if(self, condition: bool, token: &str) -> Self {
        if condition && !token.is_empty() {
            self.bearer_auth(token)
        } else {
            self
        }
    }
}
