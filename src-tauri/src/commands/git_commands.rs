use std::path::PathBuf;

use tauri::{AppHandle, State};

use crate::git::types::*;
use crate::git::{GitError, GitOperations, LocalGit};
use crate::watcher::WatcherState;

pub struct GitState {
    pub git: LocalGit,
}

#[tauri::command]
pub fn git_status(state: State<GitState>, path: String) -> Result<Vec<FileStatus>, GitError> {
    state.git.status(&PathBuf::from(&path))
}

#[tauri::command]
pub fn git_log(
    state: State<GitState>,
    path: String,
    limit: Option<usize>,
    filter: Option<LogFilter>,
) -> Result<Vec<Commit>, GitError> {
    state.git.log(&PathBuf::from(&path), limit.unwrap_or(100), filter)
}

#[tauri::command]
pub fn git_diff(
    state: State<GitState>,
    path: String,
    file: String,
) -> Result<DiffResult, GitError> {
    state
        .git
        .diff(&PathBuf::from(&path), &PathBuf::from(&file))
}

#[tauri::command]
pub fn git_stage(
    state: State<GitState>,
    path: String,
    files: Vec<String>,
) -> Result<(), GitError> {
    let file_paths: Vec<PathBuf> = files.into_iter().map(PathBuf::from).collect();
    state.git.stage(&PathBuf::from(&path), &file_paths)
}

#[tauri::command]
pub fn git_unstage(
    state: State<GitState>,
    path: String,
    files: Vec<String>,
) -> Result<(), GitError> {
    let file_paths: Vec<PathBuf> = files.into_iter().map(PathBuf::from).collect();
    state.git.unstage(&PathBuf::from(&path), &file_paths)
}

#[tauri::command]
pub fn git_commit(
    state: State<GitState>,
    path: String,
    message: String,
) -> Result<String, GitError> {
    state.git.commit(&PathBuf::from(&path), &message)
}

#[tauri::command]
pub fn git_push(
    state: State<GitState>,
    path: String,
    remote: Option<String>,
    branch: Option<String>,
) -> Result<PushResult, GitError> {
    state.git.push(
        &PathBuf::from(&path),
        &remote.unwrap_or_else(|| "origin".to_string()),
        &branch.unwrap_or_else(|| "HEAD".to_string()),
    )
}

#[tauri::command]
pub fn git_pull(
    state: State<GitState>,
    path: String,
    remote: Option<String>,
    branch: Option<String>,
) -> Result<PullResult, GitError> {
    state.git.pull(
        &PathBuf::from(&path),
        &remote.unwrap_or_else(|| "origin".to_string()),
        &branch.unwrap_or_else(|| "HEAD".to_string()),
    )
}

#[tauri::command]
pub fn git_clone(url: String, dest: String) -> Result<(), GitError> {
    let output = LocalGit::git_command()
        .args(["clone", &url, &dest])
        .output()
        .map_err(|e| GitError::OperationFailed(format!("無法執行 git clone: {e}")))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(GitError::OperationFailed(stderr))
    }
}

#[tauri::command]
pub fn start_watching(
    path: String,
    watcher: State<WatcherState>,
    app_handle: AppHandle,
) -> Result<(), GitError> {
    watcher.start(&path, app_handle)
}

#[tauri::command]
pub fn stop_watching(watcher: State<WatcherState>) -> Result<(), GitError> {
    watcher.stop()
}

/// 初始化新 Git 倉庫
#[tauri::command]
pub fn git_init(path: String) -> Result<(), GitError> {
    let repo_path = std::path::PathBuf::from(&path);
    if !repo_path.exists() {
        return Err(GitError::PathNotFound(path));
    }
    // 檢查是否已經是 git repo
    if repo_path.join(".git").exists() {
        return Err(GitError::OperationFailed(
            "此資料夾已經是 Git repository".to_string(),
        ));
    }
    git2::Repository::init(&repo_path)
        .map_err(|e| GitError::OperationFailed(format!("初始化 Git 倉庫失敗: {e}")))?;
    Ok(())
}

/// Revert 指定 commit（產生新的 revert commit）
#[tauri::command]
pub fn git_revert(
    path: String,
    commit_id: String,
    is_merge: bool,
) -> Result<String, GitError> {
    let repo_path = std::path::PathBuf::from(&path);
    crate::git::LocalGit::check_index_lock(&repo_path)?;
    let mut args = vec!["revert", "--no-edit"];
    if is_merge {
        args.push("-m");
        args.push("1");
    }
    args.push(&commit_id);
    crate::git::LocalGit::run_git(&repo_path, &args)
}

/// Soft reset 到指定 commit（保留變更到 staged）
#[tauri::command]
pub fn git_reset_soft(path: String, target: String) -> Result<(), GitError> {
    let repo_path = std::path::PathBuf::from(&path);
    crate::git::LocalGit::check_index_lock(&repo_path)?;
    crate::git::LocalGit::run_git(&repo_path, &["reset", "--soft", &target])?;
    Ok(())
}

/// Hard reset 到指定 commit（丟棄所有變更）
#[tauri::command]
pub fn git_reset_hard(path: String, target: String) -> Result<(), GitError> {
    let repo_path = std::path::PathBuf::from(&path);
    crate::git::LocalGit::check_index_lock(&repo_path)?;
    crate::git::LocalGit::run_git(&repo_path, &["reset", "--hard", &target])?;
    Ok(())
}

/// 查看單一檔案的 commit 歷史（支援追蹤 rename）
#[tauri::command]
pub fn git_file_log(
    path: String,
    file: String,
    limit: Option<usize>,
) -> Result<Vec<crate::git::types::Commit>, GitError> {
    let repo_path = std::path::PathBuf::from(&path);
    let limit_str = (limit.unwrap_or(200)).to_string();
    let output = crate::git::LocalGit::run_git(
        &repo_path,
        &[
            "log",
            "--follow",
            "--format=%H%n%an%n%ae%n%at%n%P%n%s%n---END---",
            "-n",
            &limit_str,
            "--",
            &file,
        ],
    )?;

    let mut commits = Vec::new();
    let mut lines = output.lines().peekable();

    while lines.peek().is_some() {
        let id = match lines.next() {
            Some(l) if !l.is_empty() => l.to_string(),
            _ => break,
        };
        let author = lines.next().unwrap_or("").to_string();
        let email = lines.next().unwrap_or("").to_string();
        let timestamp: i64 = lines.next().unwrap_or("0").parse().unwrap_or(0);
        let parents: Vec<String> = lines
            .next()
            .unwrap_or("")
            .split_whitespace()
            .map(String::from)
            .collect();
        let message = lines.next().unwrap_or("").to_string();
        let _ = lines.next(); // Skip ---END---

        commits.push(crate::git::types::Commit {
            id,
            message,
            author,
            email,
            timestamp,
            parents,
            refs: Vec::new(),
        });
    }

    Ok(commits)
}

#[tauri::command]
pub fn check_git_version() -> Result<String, GitError> {
    let output = LocalGit::git_command()
        .args(["--version"])
        .output()
        .map_err(|e| GitError::OperationFailed(format!("找不到 git: {e}")))?;

    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(version)
}
