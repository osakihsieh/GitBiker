pub mod gemini;
pub mod openai_compatible;

use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use std::time::Duration;

// ── HTTP Client (global singleton) ────────────────────
static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("Failed to create HTTP client")
});

pub fn http_client() -> &'static reqwest::Client {
    &CLIENT
}

// ── Error Types ───────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum AiError {
    #[error("未設定 API Key")]
    NoApiKey,

    #[error("API Key 無效: {0}")]
    Auth(String),

    #[error("已達 API 請求限制，請稍後再試")]
    RateLimit,

    #[error("AI 服務暫時不可用: {0}")]
    ServerError(String),

    #[error("AI 回傳格式異常: {0}")]
    Parse(String),

    #[error("AI 未生成任何內容")]
    EmptyResponse,

    #[error("無法連接 AI 服務: {0}")]
    Connection(String),

    #[error("AI 服務連線逾時")]
    Timeout,

    #[error("不支援的 AI 提供者: {0}")]
    UnsupportedProvider(String),

    #[error("HTTP 錯誤: {0}")]
    Http(String),

    #[error("找不到指定的模型: {0}")]
    ModelNotFound(String),
}

impl From<reqwest::Error> for AiError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            AiError::Timeout
        } else if err.is_connect() {
            AiError::Connection(err.to_string())
        } else {
            AiError::Http(err.to_string())
        }
    }
}

// ── Provider Trait ────────────────────────────────────

#[async_trait::async_trait]
pub trait AiProvider: Send + Sync {
    fn name(&self) -> &str;
    async fn generate(&self, context: &CommitContext) -> Result<String, AiError>;
}

// ── Context Types ────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitContext {
    pub diff_summary: String,
    pub staged_files: Vec<FileSummary>,
    pub recent_messages: Vec<String>,
    pub language: String,
    pub custom_prompt: Option<String>,
    pub commit_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSummary {
    pub path: String,
    pub kind: String,
    pub stats: Option<(usize, usize)>,
}

// ── Provider Config ──────────────────────────────────

#[derive(Debug, Clone)]
pub struct ProviderConfig {
    pub api_key: String,
    pub model: String,
    pub endpoint: Option<String>,
}

// ── Provider Factory ─────────────────────────────────

pub fn create_provider(
    name: &str,
    config: ProviderConfig,
) -> Result<Box<dyn AiProvider>, AiError> {
    match name {
        "gemini" => Ok(Box::new(gemini::GeminiProvider::new(config))),
        "openai" => Ok(Box::new(
            openai_compatible::OpenAiCompatibleProvider::openai(config),
        )),
        "ollama" => Ok(Box::new(
            openai_compatible::OpenAiCompatibleProvider::ollama(config),
        )),
        other => Err(AiError::UnsupportedProvider(other.to_string())),
    }
}

// ── Model Listing ───────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
}

pub async fn list_models(
    provider: &str,
    api_key: &str,
    ollama_endpoint: Option<&str>,
) -> Result<Vec<ModelInfo>, AiError> {
    match provider {
        "gemini" => gemini::list_models(api_key).await,
        "openai" => openai_compatible::list_openai_models(api_key).await,
        "ollama" => {
            let endpoint = ollama_endpoint.unwrap_or("http://localhost:11434");
            openai_compatible::list_ollama_models(endpoint).await
        }
        other => Err(AiError::UnsupportedProvider(other.to_string())),
    }
}

// ── System Prompt Builder ────────────────────────────

pub fn build_system_prompt(context: &CommitContext) -> String {
    let mut prompt = String::new();

    prompt.push_str("You are a commit message generator for a Git repository.\n");
    prompt.push_str("Generate a commit message with a title and body based on the staged changes.\n\n");

    // Format instruction (shared across languages)
    prompt.push_str("Format:\n");
    prompt.push_str("- Line 1: title (type prefix + concise summary, max 72 chars)\n");
    prompt.push_str("- Line 2: blank line\n");
    prompt.push_str("- Line 3+: body — explain WHAT changed and WHY, use bullet points for multiple changes\n");
    prompt.push_str("- Output only the commit message, no explanations or markdown fences\n\n");

    // Commit type instruction
    match context.commit_type.as_deref() {
        Some("auto") | None => {
            prompt.push_str("Commit type selection:\n");
            prompt.push_str("- Analyze the staged diff carefully to determine the correct commit type\n");
            prompt.push_str("- Do NOT default to 'feat:'. Use 'fix:' for bug fixes, 'refactor:' for code restructuring, 'docs:' for documentation changes, 'test:' for test changes, 'chore:' for maintenance, 'perf:' for performance improvements, 'ci:' for CI changes\n");
            prompt.push_str("- Only use 'feat:' for genuinely new features or capabilities\n\n");
        }
        Some(commit_type) => {
            prompt.push_str(&format!("Commit type: You MUST use the prefix: {}:\n", commit_type));
            prompt.push_str(&format!("- Do not use any other prefix. Generate a description that matches the '{}' intent.\n\n", commit_type));
        }
    }

    // Language instruction
    match context.language.as_str() {
        "zh-TW" => {
            prompt.push_str("語言規則：\n");
            prompt.push_str("- 標題前綴使用英文（feat:, fix:, refactor:, docs:, test:, chore:, perf:, ci:）\n");
            prompt.push_str("- 標題描述與內文使用繁體中文\n");
        }
        "en" => {
            prompt.push_str("Language rules:\n");
            prompt.push_str("- Use conventional commit format (feat:, fix:, refactor:, etc.)\n");
            prompt.push_str("- Write in English\n");
        }
        _ => {
            // auto: follow recent commit style
            prompt.push_str("Language rules:\n");
            prompt.push_str("- Use conventional commit format (feat:, fix:, refactor:, etc.)\n");
            prompt.push_str("- Match the language style of the recent commits below\n");
        }
    }

    // Recent commit style examples
    if !context.recent_messages.is_empty() {
        prompt.push_str("\nRecent commit messages (match this style):\n");
        for msg in &context.recent_messages {
            prompt.push_str(&format!("- {}\n", msg.lines().next().unwrap_or("")));
        }
    }

    // Custom prompt
    if let Some(custom) = &context.custom_prompt {
        let trimmed = custom.trim();
        if !trimmed.is_empty() {
            prompt.push_str(&format!("\nAdditional instructions:\n{}\n", trimmed));
        }
    }

    prompt
}

// ── Diff Truncation ──────────────────────────────────

const MAX_DIFF_CHARS: usize = 4000;
const MAX_LINES_PER_FILE: usize = 50;

/// Build a summarized diff string from staged file data.
/// Each file gets a dynamic budget based on total file count.
pub fn truncate_diff(files: &[FileSummary], diffs: &[(String, String)]) -> String {
    // diffs: Vec of (file_path, raw_diff_text)
    if files.is_empty() {
        return String::new();
    }

    let per_file_budget = MAX_DIFF_CHARS / files.len().max(1);
    let mut result = String::new();

    // File summary header
    result.push_str("Staged files:\n");
    for f in files {
        let stats_str = match f.stats {
            Some((add, del)) => format!(" (+{add} -{del})"),
            None => String::new(),
        };
        result.push_str(&format!("  {} [{}]{}\n", f.path, f.kind, stats_str));
    }
    result.push('\n');

    // Per-file diff (truncated)
    for (path, diff_text) in diffs {
        if diff_text.is_empty() {
            continue;
        }

        result.push_str(&format!("--- {} ---\n", path));

        let mut chars_used = 0;
        let mut lines_used = 0;

        for line in diff_text.lines() {
            if chars_used + line.len() > per_file_budget || lines_used >= MAX_LINES_PER_FILE {
                result.push_str("  ... (truncated)\n");
                break;
            }
            result.push_str(line);
            result.push('\n');
            chars_used += line.len() + 1;
            lines_used += 1;
        }
    }

    // Final global truncation safety net
    if result.len() > MAX_DIFF_CHARS + 500 {
        result.truncate(MAX_DIFF_CHARS);
        result.push_str("\n... (truncated)");
    }

    result
}

// ── Tests ────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_diff_empty_files() {
        let result = truncate_diff(&[], &[]);
        assert_eq!(result, "");
    }

    #[test]
    fn truncate_diff_under_budget() {
        let files = vec![
            FileSummary {
                path: "src/main.rs".to_string(),
                kind: "Modified".to_string(),
                stats: Some((5, 2)),
            },
        ];
        let diffs = vec![("src/main.rs".to_string(), "+line1\n-line2\n".to_string())];
        let result = truncate_diff(&files, &diffs);
        assert!(result.contains("src/main.rs"));
        assert!(result.contains("+line1"));
        assert!(!result.contains("truncated"));
    }

    #[test]
    fn truncate_diff_over_budget() {
        let files: Vec<FileSummary> = (0..10)
            .map(|i| FileSummary {
                path: format!("file_{i}.rs"),
                kind: "Modified".to_string(),
                stats: Some((100, 50)),
            })
            .collect();
        let long_diff = "x".repeat(1000);
        let diffs: Vec<(String, String)> = files
            .iter()
            .map(|f| (f.path.clone(), long_diff.clone()))
            .collect();
        let result = truncate_diff(&files, &diffs);
        assert!(result.contains("truncated"));
    }

    #[test]
    fn truncate_diff_100_files() {
        let files: Vec<FileSummary> = (0..100)
            .map(|i| FileSummary {
                path: format!("file_{i}.rs"),
                kind: "Added".to_string(),
                stats: None,
            })
            .collect();
        let diffs: Vec<(String, String)> = files
            .iter()
            .map(|f| (f.path.clone(), "+new content\n".to_string()))
            .collect();
        let result = truncate_diff(&files, &diffs);
        // Should not exceed budget + safety margin
        assert!(result.len() <= MAX_DIFF_CHARS + 600);
    }

    #[test]
    fn build_prompt_zh_tw() {
        let ctx = CommitContext {
            diff_summary: String::new(),
            staged_files: vec![],
            recent_messages: vec!["feat: 新增登入功能".to_string(), "fix: 修正 bug".to_string()],
            language: "zh-TW".to_string(),
            custom_prompt: Some("保持簡潔".to_string()),
            commit_type: None,
        };
        let prompt = build_system_prompt(&ctx);
        assert!(prompt.contains("繁體中文"));
        assert!(prompt.contains("feat: 新增登入功能"));
        assert!(prompt.contains("保持簡潔"));
    }

    #[test]
    fn build_prompt_en() {
        let ctx = CommitContext {
            diff_summary: String::new(),
            staged_files: vec![],
            recent_messages: vec![],
            language: "en".to_string(),
            custom_prompt: None,
            commit_type: None,
        };
        let prompt = build_system_prompt(&ctx);
        assert!(prompt.contains("English"));
        assert!(!prompt.contains("Recent commit"));
    }

    #[test]
    fn build_prompt_auto_with_recent() {
        let ctx = CommitContext {
            diff_summary: String::new(),
            staged_files: vec![],
            recent_messages: vec!["refactor: split module".to_string()],
            language: "auto".to_string(),
            custom_prompt: None,
            commit_type: None,
        };
        let prompt = build_system_prompt(&ctx);
        assert!(prompt.contains("Match the language style"));
        assert!(prompt.contains("refactor: split module"));
    }

    #[test]
    fn build_prompt_commit_type_auto() {
        let ctx = CommitContext {
            diff_summary: String::new(),
            staged_files: vec![],
            recent_messages: vec![],
            language: "en".to_string(),
            custom_prompt: None,
            commit_type: Some("auto".to_string()),
        };
        let prompt = build_system_prompt(&ctx);
        assert!(prompt.contains("Do NOT default to 'feat:'"));
        assert!(!prompt.contains("MUST use the prefix"));
    }

    #[test]
    fn build_prompt_commit_type_fix() {
        let ctx = CommitContext {
            diff_summary: String::new(),
            staged_files: vec![],
            recent_messages: vec![],
            language: "en".to_string(),
            custom_prompt: None,
            commit_type: Some("fix".to_string()),
        };
        let prompt = build_system_prompt(&ctx);
        assert!(prompt.contains("MUST use the prefix: fix:"));
        assert!(!prompt.contains("Do NOT default to 'feat:'"));
    }

    #[test]
    fn build_prompt_commit_type_none() {
        let ctx = CommitContext {
            diff_summary: String::new(),
            staged_files: vec![],
            recent_messages: vec![],
            language: "en".to_string(),
            custom_prompt: None,
            commit_type: None,
        };
        let prompt = build_system_prompt(&ctx);
        assert!(prompt.contains("Do NOT default to 'feat:'"));
        assert!(!prompt.contains("MUST use the prefix"));
    }

    #[test]
    fn create_provider_gemini() {
        let config = ProviderConfig {
            api_key: "test-key".to_string(),
            model: "gemini-2.0-flash".to_string(),
            endpoint: None,
        };
        let provider = create_provider("gemini", config).unwrap();
        assert_eq!(provider.name(), "Gemini");
    }

    #[test]
    fn create_provider_openai() {
        let config = ProviderConfig {
            api_key: "sk-test".to_string(),
            model: "gpt-4o-mini".to_string(),
            endpoint: None,
        };
        let provider = create_provider("openai", config).unwrap();
        assert_eq!(provider.name(), "OpenAI");
    }

    #[test]
    fn create_provider_ollama() {
        let config = ProviderConfig {
            api_key: String::new(),
            model: "llama3".to_string(),
            endpoint: Some("http://localhost:11434".to_string()),
        };
        let provider = create_provider("ollama", config).unwrap();
        assert_eq!(provider.name(), "Ollama");
    }

    #[test]
    fn create_provider_unknown() {
        let config = ProviderConfig {
            api_key: String::new(),
            model: String::new(),
            endpoint: None,
        };
        let result = create_provider("xxx", config);
        assert!(result.is_err());
    }
}
