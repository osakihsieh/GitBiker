use std::path::PathBuf;

use tauri::State;

use crate::git::types::*;
use crate::git::{GitError, GitOperations};

use super::git_commands::GitState;

#[tauri::command]
pub fn git_branches(state: State<GitState>, path: String) -> Result<Vec<Branch>, GitError> {
    state.git.branches(&PathBuf::from(&path))
}

#[tauri::command]
pub fn git_switch_branch(
    state: State<GitState>,
    path: String,
    name: String,
) -> Result<(), GitError> {
    state.git.switch_branch(&PathBuf::from(&path), &name)
}

#[tauri::command]
pub fn git_create_branch(
    state: State<GitState>,
    path: String,
    name: String,
) -> Result<(), GitError> {
    state.git.create_branch(&PathBuf::from(&path), &name)
}

#[tauri::command]
pub fn git_delete_branch(
    state: State<GitState>,
    path: String,
    name: String,
    force: Option<bool>,
) -> Result<(), GitError> {
    if force.unwrap_or(false) {
        state.git.force_delete_branch(&PathBuf::from(&path), &name)
    } else {
        state.git.delete_branch(&PathBuf::from(&path), &name)
    }
}

#[tauri::command]
pub fn git_rename_branch(
    state: State<GitState>,
    path: String,
    old_name: String,
    new_name: String,
) -> Result<(), GitError> {
    state.git.rename_branch(&PathBuf::from(&path), &old_name, &new_name)
}

#[tauri::command]
pub fn git_checkout_remote_branch(
    state: State<GitState>,
    path: String,
    remote_branch: String,
) -> Result<String, GitError> {
    state.git.checkout_remote_branch(&PathBuf::from(&path), &remote_branch)
}

#[tauri::command]
pub fn git_branch_merge_status(
    state: State<GitState>,
    path: String,
    branch_name: String,
    base: Option<String>,
) -> Result<BranchMergeStatus, GitError> {
    let base_branch = base.unwrap_or_else(|| "HEAD".to_string());
    state.git.branch_merge_status(&PathBuf::from(&path), &branch_name, &base_branch)
}

#[tauri::command]
pub fn git_merge_branch(
    state: State<GitState>,
    path: String,
    branch_name: String,
) -> Result<MergeResult, GitError> {
    state.git.merge_branch(&PathBuf::from(&path), &branch_name)
}

#[tauri::command]
pub fn git_merge_abort(
    state: State<GitState>,
    path: String,
) -> Result<(), GitError> {
    state.git.merge_abort(&PathBuf::from(&path))
}

#[tauri::command]
pub fn git_branch_compare(
    state: State<GitState>,
    path: String,
    base: String,
    compare: String,
) -> Result<BranchCompareResult, GitError> {
    state.git.branch_compare(&PathBuf::from(&path), &base, &compare)
}

// ── Conflict Resolution Commands ─────────────────────

#[tauri::command]
pub fn git_merge_dry_run(
    state: State<GitState>,
    path: String,
    branch_name: String,
) -> Result<MergeDryRunResult, GitError> {
    state.git.merge_dry_run(&PathBuf::from(&path), &branch_name)
}

#[tauri::command]
pub fn git_get_conflict_files(
    state: State<GitState>,
    path: String,
) -> Result<Vec<ConflictFile>, GitError> {
    state.git.get_conflict_files(&PathBuf::from(&path))
}

#[tauri::command]
pub fn git_get_conflict_content(
    state: State<GitState>,
    path: String,
    file_path: String,
) -> Result<ConflictContent, GitError> {
    state.git.get_conflict_content(&PathBuf::from(&path), &file_path)
}

#[tauri::command]
pub fn git_resolve_conflict_content(
    state: State<GitState>,
    path: String,
    file_path: String,
    resolved_content: String,
    content_hash: String,
) -> Result<(), GitError> {
    state.git.resolve_conflict_content(
        &PathBuf::from(&path),
        &file_path,
        &resolved_content,
        &content_hash,
    )
}

#[tauri::command]
pub fn git_resolve_conflict_choice(
    state: State<GitState>,
    path: String,
    file_path: String,
    choice: ResolveChoice,
) -> Result<(), GitError> {
    state.git.resolve_conflict_choice(&PathBuf::from(&path), &file_path, &choice)
}

#[tauri::command]
pub fn git_complete_merge(
    state: State<GitState>,
    path: String,
    message: Option<String>,
) -> Result<MergeCompleteResult, GitError> {
    state.git.complete_merge(&PathBuf::from(&path), &message.unwrap_or_default())
}

// ── Stash Commands ──────────────────────────────────

#[tauri::command]
pub fn git_stash_list(
    state: State<GitState>,
    path: String,
) -> Result<Vec<StashEntry>, GitError> {
    state.git.stash_list(&PathBuf::from(&path))
}

#[tauri::command]
pub fn git_stash_push(
    state: State<GitState>,
    path: String,
    message: Option<String>,
) -> Result<String, GitError> {
    state.git.stash_push(&PathBuf::from(&path), message.as_deref())
}

#[tauri::command]
pub fn git_stash_pop(
    state: State<GitState>,
    path: String,
    index: Option<usize>,
) -> Result<String, GitError> {
    state.git.stash_pop(&PathBuf::from(&path), index.unwrap_or(0))
}

#[tauri::command]
pub fn git_stash_apply(
    state: State<GitState>,
    path: String,
    index: Option<usize>,
) -> Result<String, GitError> {
    state.git.stash_apply(&PathBuf::from(&path), index.unwrap_or(0))
}

#[tauri::command]
pub fn git_stash_drop(
    state: State<GitState>,
    path: String,
    index: usize,
) -> Result<String, GitError> {
    state.git.stash_drop(&PathBuf::from(&path), index)
}

#[tauri::command]
pub fn git_stash_push_files(
    state: State<GitState>,
    path: String,
    message: Option<String>,
    files: Vec<String>,
) -> Result<String, GitError> {
    let file_paths: Vec<PathBuf> = files.into_iter().map(PathBuf::from).collect();
    state.git.stash_push_files(&PathBuf::from(&path), message.as_deref(), &file_paths)
}
