use std::path::PathBuf;

use tauri::State;

use crate::git::types::*;
use crate::git::{GitError, LocalGit};

use super::git_commands::GitState;

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
    _state: State<GitState>,
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
                return Err(GitError::OperationFailed(
                    "無法還原 untracked 檔案，請手動刪除。".to_string(),
                ));
            }
            LocalGit::run_git(&repo_path, &["checkout", "--", &file])?;
        }
        "Staged" => {
            if kind == "Added" {
                LocalGit::run_git(&repo_path, &["rm", "--cached", "--", &file])?;
            } else {
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

    let mut hunks: Vec<DiffHunk> = Vec::new();
    let mut additions: usize = 0;
    let mut deletions: usize = 0;
    let is_binary = output.contains("Binary files");

    if !is_binary {
        let mut old_line: u32 = 0;
        let mut new_line: u32 = 0;

        for raw_line in output.lines() {
            if raw_line.starts_with("@@") {
                let header = raw_line.to_string();
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
                let (kind, old_lineno, new_lineno) = if raw_line.starts_with('+') {
                    additions += 1;
                    let ln = new_line;
                    new_line += 1;
                    (DiffLineKind::Addition, None, Some(ln))
                } else if raw_line.starts_with('-') {
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
            staging: StagingState::Staged,
        });
    }

    Ok(files)
}

/// 搜尋 commit 歷史
#[tauri::command]
pub fn git_log_search(
    _state: State<GitState>,
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
        let _ = lines.next(); // Skip ---END--- marker

        commits.push(Commit {
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
