use serde::{Deserialize, Serialize};

use super::{
    build_system_prompt, http_client, AiError, AiProvider, CommitContext, ModelInfo, ProviderConfig,
};

// ── List Models ─────────────────────────────────────

#[derive(Deserialize)]
struct GeminiModelsResponse {
    models: Option<Vec<GeminiModelEntry>>,
}

#[derive(Deserialize)]
struct GeminiModelEntry {
    name: Option<String>,
    #[serde(rename = "displayName")]
    display_name: Option<String>,
    #[serde(rename = "supportedGenerationMethods")]
    supported_generation_methods: Option<Vec<String>>,
}

pub async fn list_models(api_key: &str) -> Result<Vec<ModelInfo>, AiError> {
    if api_key.is_empty() {
        return Err(AiError::NoApiKey);
    }

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models?key={}",
        api_key
    );

    let response = http_client().get(&url).send().await?;

    let status = response.status();
    if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
        return Err(AiError::Auth("API Key 無效，請確認設定".to_string()));
    }

    let body = response.text().await?;
    let parsed: GeminiModelsResponse =
        serde_json::from_str(&body).map_err(|e| AiError::Parse(e.to_string()))?;

    let models = parsed
        .models
        .unwrap_or_default()
        .into_iter()
        .filter(|m| {
            m.supported_generation_methods
                .as_ref()
                .map(|methods| methods.iter().any(|m| m == "generateContent"))
                .unwrap_or(false)
        })
        .filter_map(|m| {
            let full_name = m.name?;
            // "models/gemini-2.0-flash" → "gemini-2.0-flash"
            let id = full_name
                .strip_prefix("models/")
                .unwrap_or(&full_name)
                .to_string();
            let name = m.display_name.unwrap_or_else(|| id.clone());
            Some(ModelInfo { id, name })
        })
        .collect();

    Ok(models)
}

// ── Provider ────────────────────────────────────────

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
                parts: vec![GeminiPart { text: user_message }],
                role: Some("user".to_string()),
            }],
            system_instruction: Some(GeminiContent {
                parts: vec![GeminiPart {
                    text: system_prompt,
                }],
                role: None,
            }),
        };

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

    async fn analyze_branches(
        &self,
        branches: &[super::BranchInfo],
        language: &str,
    ) -> Result<String, AiError> {
        if self.api_key.is_empty() {
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

        let request = GeminiRequest {
            contents: vec![GeminiContent {
                parts: vec![GeminiPart { text: user_message }],
                role: Some("user".to_string()),
            }],
            system_instruction: Some(GeminiContent {
                parts: vec![GeminiPart {
                    text: system_prompt,
                }],
                role: None,
            }),
        };

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );

        let response = http_client().post(&url).json(&request).send().await?;
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

        Ok(message)
    }

    async fn resolve_conflict(
        &self,
        path: &str,
        hunk: &crate::git::types::ConflictHunk,
        _language: &str,
    ) -> Result<String, AiError> {
        if self.api_key.is_empty() {
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
            user_message.push('\n');
        }

        let request = GeminiRequest {
            contents: vec![GeminiContent {
                parts: vec![GeminiPart { text: user_message }],
                role: Some("user".to_string()),
            }],
            system_instruction: Some(GeminiContent {
                parts: vec![GeminiPart {
                    text: system_prompt,
                }],
                role: None,
            }),
        };

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );

        let response = http_client().post(&url).json(&request).send().await?;
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

        Ok(message)
    }

    async fn review_code(
        &self,
        diff_summary: &str,
        language: &str,
    ) -> Result<String, AiError> {
        if self.api_key.is_empty() {
            return Err(AiError::NoApiKey);
        }

        let mut system_prompt = String::new();
        system_prompt.push_str("You are an elite code reviewer. Your mission is to perform a pre-commit check.\n");
        system_prompt.push_str("Identify potential bugs, performance bottlenecks, security risks, and style issues in the provided diff.\n");
        system_prompt.push_str("Be critical but constructive. Suggest specific improvements.\n");

        match language {
            "zh-TW" => system_prompt.push_str("使用繁體中文回答，使用 Markdown 格式，條列式說明建議。保持專業且精簡。\n"),
            _ => system_prompt.push_str("Respond in English using Markdown. Use bullet points for suggestions. Be professional and concise.\n"),
        }

        let request = GeminiRequest {
            contents: vec![GeminiContent {
                parts: vec![GeminiPart { text: diff_summary.to_string() }],
                role: Some("user".to_string()),
            }],
            system_instruction: Some(GeminiContent {
                parts: vec![GeminiPart {
                    text: system_prompt,
                }],
                role: None,
            }),
        };

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );

        let response = http_client().post(&url).json(&request).send().await?;
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

        Ok(message)
    }
}
