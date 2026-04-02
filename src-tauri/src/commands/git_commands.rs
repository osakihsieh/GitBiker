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
