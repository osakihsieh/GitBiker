use std::path::PathBuf;

use tauri::{AppHandle, State};

use crate::git::types::*;
use crate::git::local_git::{get_disable_auto_crlf, set_disable_auto_crlf, get_ignore_eol, set_ignore_eol};
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

/// Scan a folder for git repositories up to `max_depth` levels deep.
/// Once a `.git` directory is found, that subtree is not descended further.
#[tauri::command]
pub fn scan_git_repos(path: String, max_depth: Option<u32>) -> Result<Vec<String>, GitError> {
    let folder = std::path::PathBuf::from(&path);
    if !folder.is_dir() {
        return Err(GitError::OperationFailed("不是有效的資料夾".to_string()));
    }

    let depth_limit = max_depth.unwrap_or(2);
    let mut repos = Vec::new();
    scan_repos_recursive(&folder, depth_limit, 0, &mut repos)?;
    repos.sort();
    Ok(repos)
}

fn scan_repos_recursive(
    dir: &std::path::Path,
    max_depth: u32,
    current_depth: u32,
    repos: &mut Vec<String>,
) -> Result<(), GitError> {
    if current_depth >= max_depth {
        return Ok(());
    }

    let entries = std::fs::read_dir(dir)
        .map_err(|e| GitError::OperationFailed(format!("無法讀取資料夾: {e}")))?;

    for entry in entries.flatten() {
        let entry_path = entry.path();
        if !entry_path.is_dir() {
            continue;
        }

        // Skip common non-project directories
        if let Some(name) = entry_path.file_name().and_then(|n| n.to_str()) {
            if name.starts_with('.') || name == "node_modules" || name == "target" || name == "vendor" {
                continue;
            }
        }

        if entry_path.join(".git").exists() {
            if let Some(p) = entry_path.to_str() {
                repos.push(p.to_string());
            }
            // Don't descend into git repos (no nested scanning)
        } else {
            // Recurse into subdirectory
            let _ = scan_repos_recursive(&entry_path, max_depth, current_depth + 1, repos);
        }
    }

    Ok(())
}

/// Stage a patch (hunk-level staging)
#[tauri::command]
pub fn git_stage_hunk(path: String, patch: String) -> Result<(), GitError> {
    let repo_path = std::path::PathBuf::from(&path);
    crate::git::LocalGit::apply_patch(&repo_path, &patch, true, false)
}

/// Unstage a patch (hunk-level unstaging)
#[tauri::command]
pub fn git_unstage_hunk(path: String, patch: String) -> Result<(), GitError> {
    let repo_path = std::path::PathBuf::from(&path);
    crate::git::LocalGit::apply_patch(&repo_path, &patch, true, true)
}

/// Stash a single hunk by applying it to index then stashing staged changes
#[tauri::command]
pub fn git_stash_hunk(path: String, patch: String, message: Option<String>) -> Result<String, GitError> {
    let repo_path = std::path::PathBuf::from(&path);
    crate::git::LocalGit::check_index_lock(&repo_path)?;

    // Step 1: Apply the patch to the index
    crate::git::LocalGit::apply_patch(&repo_path, &patch, true, false)?;

    // Step 2: Stash only the staged changes (requires Git 2.35+)
    let mut args = vec!["stash", "push", "--staged"];
    let msg;
    if let Some(ref m) = message {
        msg = m.clone();
        args.push("-m");
        args.push(&msg);
    }

    match crate::git::LocalGit::run_git(&repo_path, &args) {
        Ok(output) => Ok(output),
        Err(e) => {
            // If --staged is not supported, unstage and return error
            let _ = crate::git::LocalGit::apply_patch(&repo_path, &patch, true, true);
            Err(e)
        }
    }
}

/// Cherry-pick 指定 commit 到當前分支
#[tauri::command]
pub fn git_cherry_pick(
    path: String,
    commit_id: String,
) -> Result<crate::git::types::CherryPickResult, GitError> {
    let repo_path = std::path::PathBuf::from(&path);
    crate::git::LocalGit::check_index_lock(&repo_path)?;
    match crate::git::LocalGit::run_git(&repo_path, &["cherry-pick", "--no-edit", &commit_id]) {
        Ok(output) => Ok(crate::git::types::CherryPickResult {
            commit_id,
            success: true,
            message: output,
            conflicts: Vec::new(),
        }),
        Err(GitError::OperationFailed(stderr)) => {
            if stderr.contains("CONFLICT") || stderr.contains("could not apply") {
                let conflicts: Vec<String> = stderr
                    .lines()
                    .filter(|l| l.contains("CONFLICT"))
                    .map(|l| l.to_string())
                    .collect();
                Ok(crate::git::types::CherryPickResult {
                    commit_id,
                    success: false,
                    message: stderr,
                    conflicts,
                })
            } else {
                Err(GitError::OperationFailed(stderr))
            }
        }
        Err(e) => Err(e),
    }
}

/// Cherry-pick 中止
#[tauri::command]
pub fn git_cherry_pick_abort(path: String) -> Result<(), GitError> {
    let repo_path = std::path::PathBuf::from(&path);
    crate::git::LocalGit::run_git(&repo_path, &["cherry-pick", "--abort"])?;
    Ok(())
}

/// Cherry-pick 繼續（衝突解決後）
#[tauri::command]
pub fn git_cherry_pick_continue(path: String) -> Result<String, GitError> {
    let repo_path = std::path::PathBuf::from(&path);
    crate::git::LocalGit::check_index_lock(&repo_path)?;
    crate::git::LocalGit::run_git(&repo_path, &["cherry-pick", "--continue"])
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

#[tauri::command]
pub fn set_git_disable_auto_crlf(disabled: bool) {
    set_disable_auto_crlf(disabled);
}

#[tauri::command]
pub fn get_git_disable_auto_crlf() -> bool {
    get_disable_auto_crlf()
}

#[tauri::command]
pub fn set_git_ignore_eol(enabled: bool) {
    set_ignore_eol(enabled);
}

#[tauri::command]
pub fn get_git_ignore_eol() -> bool {
    get_ignore_eol()
}
