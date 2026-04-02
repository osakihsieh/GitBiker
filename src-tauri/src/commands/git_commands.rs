use std::path::PathBuf;
use std::process::Command;

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
) -> Result<Vec<Commit>, GitError> {
    state.git.log(&PathBuf::from(&path), limit.unwrap_or(100))
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
) -> Result<(), GitError> {
    state.git.delete_branch(&PathBuf::from(&path), &name)
}

#[tauri::command]
pub fn git_clone(url: String, dest: String) -> Result<(), GitError> {
    let output = Command::new("git")
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

#[tauri::command]
pub fn check_git_version() -> Result<String, GitError> {
    let output = Command::new("git")
        .args(["--version"])
        .output()
        .map_err(|e| GitError::OperationFailed(format!("找不到 git: {e}")))?;

    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(version)
}

// ── File Operations ────────────────────────────────────

/// 將 pattern 加入 .gitignore（如果不存在的話）
#[tauri::command]
pub fn git_ignore(path: String, pattern: String) -> Result<(), GitError> {
    let gitignore_path = PathBuf::from(&path).join(".gitignore");

    // 讀取現有內容，檢查 pattern 是否已存在
    let existing = std::fs::read_to_string(&gitignore_path).unwrap_or_default();
    let already_exists = existing.lines().any(|line| line.trim() == pattern.trim());
    if already_exists {
        return Ok(());
    }

    // 追加 pattern（確保前面有換行）
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&gitignore_path)
        .map_err(|e| GitError::OperationFailed(format!("無法寫入 .gitignore: {e}")))?;

    if !existing.is_empty() && !existing.ends_with('\n') {
        writeln!(file).map_err(|e| GitError::OperationFailed(e.to_string()))?;
    }
    writeln!(file, "{}", pattern.trim())
        .map_err(|e| GitError::OperationFailed(format!("寫入 .gitignore 失敗: {e}")))?;

    Ok(())
}

/// 還原檔案：unstaged 區域用 checkout，staged 區域用 restore --staged，新增檔案用 rm --cached
#[tauri::command]
pub fn git_checkout_file(
    state: State<GitState>,
    path: String,
    file: String,
    staging: String,
    kind: String,
) -> Result<(), GitError> {
    let repo_path = PathBuf::from(&path);
    LocalGit::check_index_lock(&repo_path)?;

    match staging.as_str() {
        "Unstaged" => {
            if kind == "Untracked" {
                // Untracked 檔案不能 checkout，只能刪除（但我們不自動刪除）
                return Err(GitError::OperationFailed(
                    "無法還原 untracked 檔案，請手動刪除。".to_string(),
                ));
            }
            // git checkout -- <file>
            LocalGit::run_git(&repo_path, &["checkout", "--", &file])?;
        }
        "Staged" => {
            if kind == "Added" {
                // 新增的 staged 檔案用 rm --cached（從 index 移除，不刪磁碟檔案）
                LocalGit::run_git(&repo_path, &["rm", "--cached", "--", &file])?;
            } else {
                // 已有的 staged 檔案用 restore --staged（只 unstage，不刪除變更）
                LocalGit::run_git(&repo_path, &["restore", "--staged", "--", &file])?;
            }
        }
        _ => {
            return Err(GitError::OperationFailed(format!(
                "未知的 staging 狀態: {staging}"
            )));
        }
    }

    Ok(())
}

// ── External Tools ─────────────────────────────────────

#[tauri::command]
pub fn open_in_folder(path: String) -> Result<(), String> {
    let path_ref = std::path::Path::new(&path);
    if !path_ref.exists() {
        return Err(format!("路徑不存在: {path}"));
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("explorer")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("無法開啟資料夾: {e}"))?;
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("無法開啟資料夾: {e}"))?;
    }
    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("無法開啟資料夾: {e}"))?;
    }

    Ok(())
}

#[tauri::command]
pub fn open_in_editor(path: String) -> Result<(), String> {
    let path_ref = std::path::Path::new(&path);
    if !path_ref.exists() {
        return Err(format!("路徑不存在: {path}"));
    }

    // Check environment variables first
    if let Ok(editor) = std::env::var("VISUAL").or_else(|_| std::env::var("EDITOR")) {
        if Command::new(&editor).arg(&path).spawn().is_ok() {
            return Ok(());
        }
    }

    // Try common editors in order of preference
    let candidates = if cfg!(target_os = "windows") {
        vec!["code.cmd", "code", "cursor.cmd", "cursor", "zed", "subl"]
    } else {
        vec!["code", "cursor", "zed", "subl"]
    };

    for editor in &candidates {
        if Command::new(editor).arg(&path).spawn().is_ok() {
            return Ok(());
        }
    }

    Err("找不到編輯器。請確認 VS Code、Cursor 或其他編輯器已安裝並加入 PATH。".to_string())
}

#[tauri::command]
pub fn open_in_terminal(path: String) -> Result<(), String> {
    let path_ref = std::path::Path::new(&path);
    if !path_ref.exists() {
        return Err(format!("路徑不存在: {path}"));
    }

    #[cfg(target_os = "windows")]
    {
        // Try Windows Terminal first, fallback to cmd
        if Command::new("wt").args(["-d", &path]).spawn().is_ok() {
            return Ok(());
        }
        Command::new("cmd")
            .args(["/c", "start", "cmd", "/k", &format!("cd /d {}", &path)])
            .spawn()
            .map_err(|e| format!("無法開啟終端機: {e}"))?;
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .args(["-a", "Terminal", &path])
            .spawn()
            .map_err(|e| format!("無法開啟終端機: {e}"))?;
    }
    #[cfg(target_os = "linux")]
    {
        // Try common terminals
        for term in &["gnome-terminal", "konsole", "xterm"] {
            if Command::new(term)
                .args(["--working-directory", &path])
                .spawn()
                .is_ok()
            {
                return Ok(());
            }
        }
        return Err("找不到終端機程式。".to_string());
    }

    Ok(())
}
