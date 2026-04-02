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

// ── Commit Detail + Search ─────────────────────────────

/// 顯示特定 commit 中某個檔案的 diff（相對於其 parent）
#[tauri::command]
pub fn git_show_file_diff(
    path: String,
    commit_id: String,
    file: String,
) -> Result<DiffResult, GitError> {
    let repo_path = PathBuf::from(&path);
    let output = LocalGit::run_git(
        &repo_path,
        &["show", "--format=", "--no-color", &commit_id, "--", &file],
    )?;

    // Parse unified diff output into DiffResult
    let mut hunks: Vec<DiffHunk> = Vec::new();
    let mut additions: usize = 0;
    let mut deletions: usize = 0;
    let mut is_binary = output.contains("Binary files");

    if !is_binary {
        let mut old_line: u32 = 0;
        let mut new_line: u32 = 0;

        for raw_line in output.lines() {
            if raw_line.starts_with("@@") {
                // Parse hunk header like @@ -1,3 +1,4 @@
                let header = raw_line.to_string();
                // Extract line numbers
                if let Some(plus) = raw_line.find('+') {
                    let after_plus = &raw_line[plus + 1..];
                    if let Some(comma_or_space) = after_plus.find(|c: char| c == ',' || c == ' ') {
                        new_line = after_plus[..comma_or_space].parse().unwrap_or(1);
                    }
                }
                if let Some(minus) = raw_line.find('-') {
                    let after_minus = &raw_line[minus + 1..];
                    if let Some(comma_or_space) = after_minus.find(|c: char| c == ',' || c == ' ') {
                        old_line = after_minus[..comma_or_space].parse().unwrap_or(1);
                    }
                }
                hunks.push(DiffHunk { header, lines: Vec::new() });
            } else if let Some(last_hunk) = hunks.last_mut() {
                let (kind, old_lineno, new_lineno) = if let Some(content) = raw_line.strip_prefix('+') {
                    additions += 1;
                    let ln = new_line;
                    new_line += 1;
                    (DiffLineKind::Addition, None, Some(ln))
                } else if let Some(content) = raw_line.strip_prefix('-') {
                    deletions += 1;
                    let ln = old_line;
                    old_line += 1;
                    (DiffLineKind::Deletion, Some(ln), None)
                } else if raw_line.starts_with(' ') || raw_line.is_empty() {
                    let oln = old_line;
                    let nln = new_line;
                    old_line += 1;
                    new_line += 1;
                    (DiffLineKind::Context, Some(oln), Some(nln))
                } else {
                    continue;
                };

                let content = if raw_line.len() > 1 { raw_line[1..].to_string() } else { String::new() };
                last_hunk.lines.push(DiffLine { kind, content, old_lineno, new_lineno });
            }
        }
    }

    Ok(DiffResult {
        file_path: PathBuf::from(&file),
        hunks,
        stats: DiffStats { additions, deletions },
        is_binary,
        is_truncated: false,
    })
}

/// 取得特定 commit 的檔案列表
#[tauri::command]
pub fn git_show_files(path: String, commit_id: String) -> Result<Vec<FileStatus>, GitError> {
    let repo_path = PathBuf::from(&path);
    let output = LocalGit::run_git(
        &repo_path,
        &["diff-tree", "--no-commit-id", "-r", "--name-status", &commit_id],
    )?;

    let mut files = Vec::new();
    for line in output.lines() {
        let parts: Vec<&str> = line.splitn(2, '\t').collect();
        if parts.len() != 2 { continue; }
        let (status_char, file_path) = (parts[0], parts[1]);
        let kind = match status_char {
            "M" => FileStatusKind::Modified,
            "A" => FileStatusKind::Added,
            "D" => FileStatusKind::Deleted,
            "R" => FileStatusKind::Renamed,
            "C" => FileStatusKind::Copied,
            _ => FileStatusKind::Unknown,
        };
        files.push(FileStatus {
            path: PathBuf::from(file_path),
            kind,
            staging: StagingState::Staged, // commit 裡的檔案都算 "staged"
        });
    }

    Ok(files)
}

/// 搜尋 commit 歷史
#[tauri::command]
pub fn git_log_search(
    state: State<GitState>,
    path: String,
    query: String,
    search_type: String,
    limit: Option<usize>,
) -> Result<Vec<Commit>, GitError> {
    let repo_path = PathBuf::from(&path);
    let limit_str = (limit.unwrap_or(200)).to_string();

    let mut args = vec!["log", "--format=%H%n%an%n%ae%n%at%n%P%n%s%n---END---", "-n", &limit_str];

    match search_type.as_str() {
        "message" => {
            args.push("--grep");
            args.push(&query);
            args.push("--fixed-strings");
        }
        "author" => {
            args.push("--author");
            args.push(&query);
        }
        _ => {
            args.push("--grep");
            args.push(&query);
            args.push("--fixed-strings");
        }
    }

    let output = LocalGit::run_git(&repo_path, &args)?;

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
        let parents: Vec<String> = lines.next().unwrap_or("").split_whitespace().map(String::from).collect();
        let message = lines.next().unwrap_or("").to_string();
        // Skip ---END--- marker
        let _ = lines.next();

        commits.push(Commit {
            id,
            message,
            author,
            email,
            timestamp,
            parents,
            refs: Vec::new(), // search results don't need refs
        });
    }

    Ok(commits)
}

// ── Remote Management ──────────────────────────────────

#[tauri::command]
pub fn git_remote_list(path: String) -> Result<Vec<RemoteInfo>, GitError> {
    let repo_path = PathBuf::from(&path);
    let output = LocalGit::run_git(&repo_path, &["remote", "-v"])?;

    let mut remotes: Vec<RemoteInfo> = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for line in output.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let name = parts[0].to_string();
            if seen.insert(name.clone()) {
                remotes.push(RemoteInfo { name, url: parts[1].to_string() });
            }
        }
    }

    Ok(remotes)
}

#[tauri::command]
pub fn git_remote_add(path: String, name: String, url: String) -> Result<(), GitError> {
    // Validate URL format
    if !url.starts_with("https://") && !url.starts_with("http://") && !url.starts_with("git@") && !url.starts_with("ssh://") {
        return Err(GitError::OperationFailed("Remote URL 格式不正確，請使用 https:// 或 git@ 格式。".to_string()));
    }
    let repo_path = PathBuf::from(&path);
    LocalGit::run_git(&repo_path, &["remote", "add", "--", &name, &url])?;
    Ok(())
}

#[tauri::command]
pub fn git_remote_remove(path: String, name: String) -> Result<(), GitError> {
    let repo_path = PathBuf::from(&path);
    LocalGit::run_git(&repo_path, &["remote", "remove", "--", &name])?;
    Ok(())
}

#[tauri::command]
pub fn git_remote_rename(path: String, old_name: String, new_name: String) -> Result<(), GitError> {
    let repo_path = PathBuf::from(&path);
    LocalGit::run_git(&repo_path, &["remote", "rename", "--", &old_name, &new_name])?;
    Ok(())
}

// ── Tag ────────────────────────────────────────────────

#[tauri::command]
pub fn git_tag_create(path: String, name: String, commit_id: Option<String>) -> Result<(), GitError> {
    let repo_path = PathBuf::from(&path);
    let mut args = vec!["tag", "--", &name];
    let cid;
    if let Some(ref id) = commit_id {
        cid = id.clone();
        args.push(&cid);
    }
    LocalGit::run_git(&repo_path, &args)?;
    Ok(())
}

// ── Fetch ──────────────────────────────────────────────

#[tauri::command]
pub fn git_fetch(path: String, remote: Option<String>) -> Result<String, GitError> {
    let repo_path = PathBuf::from(&path);
    let mut args = vec!["fetch"];
    let r;
    if let Some(ref remote_name) = remote {
        r = remote_name.clone();
        args.push(&r);
    }
    let output = LocalGit::run_git(&repo_path, &args)?;
    Ok(output)
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
