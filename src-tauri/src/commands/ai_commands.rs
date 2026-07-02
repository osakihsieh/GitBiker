use tauri::State;

use crate::ai::{self, CommitContext, ProviderConfig};
use crate::commands::git_commands::GitState;
use crate::git::{GitError, GitOperations};

use std::path::PathBuf;

// ── List AI Models ──────────────────────────────────

#[tauri::command]
pub async fn list_ai_models(
    provider: String,
    api_key: String,
    ollama_endpoint: Option<String>,
) -> Result<Vec<ai::ModelInfo>, String> {
    ai::list_models(&provider, &api_key, ollama_endpoint.as_deref())
        .await
        .map_err(|e| e.to_string())
}

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
    commit_type: Option<String>,
) -> Result<String, GitError> {
    let repo_path = PathBuf::from(&path);

    // 1. Get staged diff (sync git2 operation — fast, <100ms)
    let (file_summaries, file_diffs) = state.git.staged_diff_all(&repo_path)?;

    if file_summaries.is_empty() {
        return Err(GitError::OperationFailed("沒有已暫存的檔案".to_string()));
    }

    // 2. Get recent commit messages for style matching
    let recent_messages: Vec<String> = {
        let commits = state
            .git
            .log(&repo_path, 5, Some(crate::git::types::LogFilter::Head))
            .unwrap_or_default();
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
        commit_type,
    };

    // 5. Create provider and generate (async HTTP)
    let config = ProviderConfig {
        api_key,
        model,
        endpoint: ollama_endpoint,
    };

    let ai_provider = ai::create_provider(&provider, config)
        .map_err(|e| GitError::OperationFailed(e.to_string()))?;

    let message = ai_provider
        .generate(&context)
        .await
        .map_err(|e| GitError::OperationFailed(e.to_string()))?;

    Ok(message)
}

#[tauri::command]
pub async fn analyze_branches(
    state: State<'_, GitState>,
    path: String,
    provider: String,
    api_key: String,
    model: String,
    language: String,
    ollama_endpoint: Option<String>,
) -> Result<String, GitError> {
    let repo_path = PathBuf::from(&path);
    let branches = state.git.branches(&repo_path)?;
    let mut branch_infos = Vec::new();

    for b in branches {
        if b.is_remote {
            continue;
        }
        let is_merged = state
            .git
            .branch_merge_status(&repo_path, &b.name, "main")
            .map(|s| s.merged)
            .unwrap_or(false);

        let last_commit = state
            .git
            .log(&repo_path, 1, Some(crate::git::types::LogFilter::Branch(b.name.clone())))
            .ok()
            .and_then(|log| log.into_iter().next());

        branch_infos.push(ai::BranchInfo {
            name: b.name,
            last_commit_message: last_commit.as_ref().map(|c| c.message.clone()).unwrap_or_default(),
            last_commit_timestamp: last_commit.as_ref().map(|c| c.timestamp).unwrap_or(0),
            is_merged,
            ahead: b.ahead.unwrap_or(0),
            behind: b.behind.unwrap_or(0),
            upstream: b.upstream,
        });
    }

    let config = ProviderConfig {
        api_key,
        model,
        endpoint: ollama_endpoint,
    };

    let ai_provider = ai::create_provider(&provider, config)
        .map_err(|e| GitError::OperationFailed(e.to_string()))?;

    let analysis = ai_provider
        .analyze_branches(&branch_infos, &language)
        .await
        .map_err(|e| GitError::OperationFailed(e.to_string()))?;

    Ok(analysis)
}

#[tauri::command]
pub async fn ai_resolve_conflict(
    _state: State<'_, GitState>,
    path: String,
    hunk: crate::git::types::ConflictHunk,
    provider: String,
    api_key: String,
    model: String,
    language: String,
    ollama_endpoint: Option<String>,
) -> Result<String, GitError> {
    let config = ProviderConfig {
        api_key,
        model,
        endpoint: ollama_endpoint,
    };

    let ai_provider = ai::create_provider(&provider, config)
        .map_err(|e| GitError::OperationFailed(e.to_string()))?;

    let resolved = ai_provider
        .resolve_conflict(&path, &hunk, &language)
        .await
        .map_err(|e| GitError::OperationFailed(e.to_string()))?;

    Ok(resolved)
}

#[tauri::command]
pub async fn ai_review_staged(
    state: State<'_, GitState>,
    path: String,
    provider: String,
    api_key: String,
    model: String,
    language: String,
    ollama_endpoint: Option<String>,
) -> Result<String, GitError> {
    let repo_path = PathBuf::from(&path);

    // 1. Get staged diff
    let (file_summaries, file_diffs) = state.git.staged_diff_all(&repo_path)?;
    if file_summaries.is_empty() {
        return Err(GitError::OperationFailed("沒有已暫存的檔案可供審查".to_string()));
    }

    // 2. Build diff summary
    let diff_summary = ai::truncate_diff(&file_summaries, &file_diffs);

    // 3. Create provider and generate
    let config = ProviderConfig {
        api_key,
        model,
        endpoint: ollama_endpoint,
    };

    let ai_provider = ai::create_provider(&provider, config)
        .map_err(|e| GitError::OperationFailed(e.to_string()))?;

    let review = ai_provider
        .review_code(&diff_summary, &language)
        .await
        .map_err(|e| GitError::OperationFailed(e.to_string()))?;

    Ok(review)
}
