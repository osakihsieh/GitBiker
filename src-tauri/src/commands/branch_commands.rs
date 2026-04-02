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
