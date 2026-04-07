use tauri::State;

use crate::ai::{self, CommitContext, ProviderConfig};
use crate::commands::git_commands::GitState;
use crate::git::{GitError, GitOperations};

use std::path::PathBuf;

#[tauri::command]
pub async fn generate_commit_message(
    state: State<'_, GitState>,
    path: String,
    provider: String,
    api_key: String,
    model: String,
    language: String,
    custom_prompt: Option<String>,
    ollama_endpoint: Option<String>,
) -> Result<String, GitError> {
    let repo_path = PathBuf::from(&path);

    // 1. Get staged diff (sync git2 operation — fast, <100ms)
    let (file_summaries, file_diffs) = state.git.staged_diff_all(&repo_path)?;

    if file_summaries.is_empty() {
        return Err(GitError::OperationFailed("沒有已暫存的檔案".to_string()));
    }

    // 2. Get recent commit messages for style matching
    let recent_messages: Vec<String> = {
        let commits = state.git.log(
            &repo_path,
            5,
            Some(crate::git::types::LogFilter::Head),
        ).unwrap_or_default();
        commits.into_iter().map(|c| c.message).collect()
    };

    // 3. Build diff summary (truncated)
    let diff_summary = ai::truncate_diff(&file_summaries, &file_diffs);

    // 4. Build context
    let context = CommitContext {
        diff_summary,
        staged_files: file_summaries,
        recent_messages,
        language,
        custom_prompt,
    };

    // 5. Create provider and generate (async HTTP)
    let config = ProviderConfig {
        api_key,
        model,
        endpoint: ollama_endpoint,
    };

    let ai_provider =
        ai::create_provider(&provider, config).map_err(|e| GitError::OperationFailed(e.to_string()))?;

    let message = ai_provider
        .generate(&context)
        .await
        .map_err(|e| GitError::OperationFailed(e.to_string()))?;

    Ok(message)
}
