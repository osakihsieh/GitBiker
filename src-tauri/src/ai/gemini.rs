use serde::{Deserialize, Serialize};

use super::{build_system_prompt, http_client, AiError, AiProvider, CommitContext, ProviderConfig};

pub struct GeminiProvider {
    api_key: String,
    model: String,
}

impl GeminiProvider {
    pub fn new(config: ProviderConfig) -> Self {
        Self {
            api_key: config.api_key,
            model: if config.model.is_empty() {
                "gemini-2.0-flash".to_string()
            } else {
                config.model
            },
        }
    }
}

// ── Gemini API Types ─────────────────────────────────

#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    #[serde(rename = "systemInstruction", skip_serializing_if = "Option::is_none")]
    system_instruction: Option<GeminiContent>,
}

#[derive(Serialize, Deserialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
    #[serde(skip_serializing_if = "Option::is_none")]
    role: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct GeminiPart {
    text: String,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Option<Vec<GeminiCandidate>>,
}

#[derive(Deserialize)]
struct GeminiCandidate {
    content: Option<GeminiContent>,
}

// ── AiProvider impl ──────────────────────────────────

#[async_trait::async_trait]
impl AiProvider for GeminiProvider {
    fn name(&self) -> &str {
        "Gemini"
    }

    async fn generate(&self, context: &CommitContext) -> Result<String, AiError> {
        if self.api_key.is_empty() {
            return Err(AiError::NoApiKey);
        }

        let system_prompt = build_system_prompt(context);
        let user_message = format!(
            "Based on the following staged changes, generate a commit message:\n\n{}",
            context.diff_summary
        );

        let request = GeminiRequest {
            contents: vec![GeminiContent {
                parts: vec![GeminiPart {
                    text: user_message,
                }],
                role: Some("user".to_string()),
            }],
            system_instruction: Some(GeminiContent {
                parts: vec![GeminiPart {
                    text: system_prompt,
                }],
                role: None,
            }),
        };

        // Note: API key is in query param per Google's design.
        // reqwest does not log URLs by default; we don't enable tracing for requests.
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );

        let response = http_client().post(&url).json(&request).send().await?;

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
        let parsed: GeminiResponse =
            serde_json::from_str(&body).map_err(|e| AiError::Parse(e.to_string()))?;

        let message = parsed
            .candidates
            .and_then(|c| c.into_iter().next())
            .and_then(|c| c.content)
            .and_then(|c| c.parts.into_iter().next())
            .map(|p| p.text.trim().to_string())
            .unwrap_or_default();

        if message.is_empty() {
            return Err(AiError::EmptyResponse);
        }

        Ok(message)
    }
}
