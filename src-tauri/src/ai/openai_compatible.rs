use serde::{Deserialize, Serialize};

use super::{
    build_system_prompt, http_client, AiError, AiProvider, CommitContext, ModelInfo, ProviderConfig,
};

// ── List Models ─────────────────────────────────────

#[derive(Deserialize)]
struct OpenAiModelsResponse {
    data: Option<Vec<OpenAiModelEntry>>,
}

#[derive(Deserialize)]
struct OpenAiModelEntry {
    id: String,
}

pub async fn list_openai_models(api_key: &str) -> Result<Vec<ModelInfo>, AiError> {
    if api_key.is_empty() {
        return Err(AiError::NoApiKey);
    }

    let response = http_client()
        .get("https://api.openai.com/v1/models")
        .bearer_auth(api_key)
        .send()
        .await?;

    let status = response.status();
    if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
        return Err(AiError::Auth("API Key 無效，請確認設定".to_string()));
    }

    let body = response.text().await?;
    let parsed: OpenAiModelsResponse =
        serde_json::from_str(&body).map_err(|e| AiError::Parse(e.to_string()))?;

    let mut models: Vec<ModelInfo> = parsed
        .data
        .unwrap_or_default()
        .into_iter()
        .filter(|m| {
            let id = &m.id;
            id.starts_with("gpt-")
                || id.starts_with("o1")
                || id.starts_with("o3")
                || id.starts_with("o4")
        })
        .map(|m| ModelInfo {
            name: m.id.clone(),
            id: m.id,
        })
        .collect();

    models.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(models)
}

#[derive(Deserialize)]
struct OllamaTagsResponse {
    models: Option<Vec<OllamaModelEntry>>,
}

#[derive(Deserialize)]
struct OllamaModelEntry {
    name: Option<String>,
    model: Option<String>,
}

pub async fn list_ollama_models(endpoint: &str) -> Result<Vec<ModelInfo>, AiError> {
    let url = format!("{}/api/tags", endpoint.trim_end_matches('/'));

    let response = http_client().get(&url).send().await?;

    let body = response.text().await?;
    let parsed: OllamaTagsResponse =
        serde_json::from_str(&body).map_err(|e| AiError::Parse(e.to_string()))?;

    let models = parsed
        .models
        .unwrap_or_default()
        .into_iter()
        .filter_map(|m| {
            let id = m.model.or(m.name)?;
            let name = id.clone();
            Some(ModelInfo { id, name })
        })
        .collect();

    Ok(models)
}

// ── Provider ────────────────────────────────────────

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

    async fn analyze_branches(
        &self,
        branches: &[super::BranchInfo],
        language: &str,
    ) -> Result<String, AiError> {
        if self.requires_auth && self.api_key.is_empty() {
            return Err(AiError::NoApiKey);
        }

        let mut system_prompt = String::new();
        system_prompt.push_str("You are a Git repository expert and cleanup assistant.\n");
        system_prompt.push_str("Analyze the provided branch list and suggest cleanup actions.\n");
        system_prompt.push_str("Identify merged branches, stale branches, and potential candidates for deletion.\n");

        match language {
            "zh-TW" => system_prompt.push_str("使用繁體中文回答，語氣專業且精簡。\n"),
            _ => system_prompt.push_str("Respond in English, be professional and concise.\n"),
        }

        let mut user_message = String::new();
        user_message.push_str("Branch List:\n");
        for b in branches {
            let status = if b.is_merged { "Merged" } else { "Unmerged" };
            user_message.push_str(&format!(
                "- {}: [{}], Ahead: {}, Behind: {}, Last Commit: \"{}\" (Timestamp: {})\n",
                b.name, status, b.ahead, b.behind, b.last_commit_message, b.last_commit_timestamp
            ));
        }

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

        let response = req_builder.send().await?;
        let body = response.text().await?;
        let parsed: ChatResponse =
            serde_json::from_str(&body).map_err(|e| AiError::Parse(e.to_string()))?;

        let message = parsed
            .choices
            .and_then(|c| c.into_iter().next())
            .and_then(|c| c.message)
            .and_then(|m| m.content)
            .map(|s| s.trim().to_string())
            .unwrap_or_default();

        Ok(message)
    }

    async fn resolve_conflict(
        &self,
        path: &str,
        hunk: &crate::git::types::ConflictHunk,
        language: &str,
    ) -> Result<String, AiError> {
        if self.requires_auth && self.api_key.is_empty() {
            return Err(AiError::NoApiKey);
        }

        let mut system_prompt = String::new();
        system_prompt.push_str("You are a code fusion expert. Your task is to resolve a Git merge conflict.\n");
        system_prompt.push_str("Analyze the 'ours' (HEAD) and 'theirs' versions of the code and provide a clean, integrated version.\n");
        system_prompt.push_str("If there is a base version, use it to understand the changes from both sides.\n");
        system_prompt.push_str("Output ONLY the resolved code without any explanation or markdown markers.\n");

        let mut user_message = String::new();
        user_message.push_str(&format!("File: {}\n\n", path));
        user_message.push_str("<<< OURS (HEAD) <<<\n");
        user_message.push_str(&hunk.ours);
        user_message.push_str("\n=======\n");
        user_message.push_str(&hunk.theirs);
        user_message.push_str("\n>>> THEIRS >>>\n");

        if let Some(base) = &hunk.base {
            user_message.push_str("\n||| BASE |||\n");
            user_message.push_str(base);
            user_message.push_str("\n");
        }

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

        let response = req_builder.send().await?;
        let body = response.text().await?;
        let parsed: ChatResponse =
            serde_json::from_str(&body).map_err(|e| AiError::Parse(e.to_string()))?;

        let message = parsed
            .choices
            .and_then(|c| c.into_iter().next())
            .and_then(|c| c.message)
            .and_then(|m| m.content)
            .map(|s| s.trim().to_string())
            .unwrap_or_default();

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
