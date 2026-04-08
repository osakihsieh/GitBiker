use std::path::PathBuf;

use crate::git::types::*;
use crate::git::{GitError, LocalGit};

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

#[tauri::command]
pub fn git_tags(path: String) -> Result<Vec<TagInfo>, GitError> {
    let repo_path = PathBuf::from(&path);
    // format: refname, objectname, subject, creatordate (unix timestamp)
    let output = LocalGit::run_git(
        &repo_path,
        &[
            "tag", "-l",
            "--format=%(refname:short)\t%(objectname:short)\t%(contents:subject)\t%(creatordate:unix)",
            "--sort=-creatordate",
        ],
    )?;

    let tags: Vec<TagInfo> = output
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| {
            let parts: Vec<&str> = line.splitn(4, '\t').collect();
            TagInfo {
                name: parts.first().unwrap_or(&"").to_string(),
                commit_id: parts.get(1).unwrap_or(&"").to_string(),
                message: parts.get(2).filter(|s| !s.is_empty()).map(|s| s.to_string()),
                timestamp: parts.get(3).and_then(|s| s.trim().parse::<i64>().ok()),
            }
        })
        .collect();

    Ok(tags)
}

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

#[tauri::command]
pub fn git_tag_delete(path: String, name: String) -> Result<(), GitError> {
    let repo_path = PathBuf::from(&path);
    LocalGit::run_git(&repo_path, &["tag", "-d", "--", &name])?;
    Ok(())
}

#[tauri::command]
pub fn git_tag_delete_remote(path: String, name: String, remote: Option<String>) -> Result<PushResult, GitError> {
    let repo_path = PathBuf::from(&path);
    let remote_name = remote.unwrap_or_else(|| "origin".to_string());
    let refspec = format!(":refs/tags/{}", name);
    match LocalGit::run_git(&repo_path, &["push", &remote_name, &refspec]) {
        Ok(output) => Ok(PushResult {
            remote: remote_name,
            branch: name,
            success: true,
            message: output,
        }),
        Err(e) => Ok(PushResult {
            remote: remote_name,
            branch: name,
            success: false,
            message: e.to_string(),
        }),
    }
}

#[tauri::command]
pub fn git_push_tag(path: String, name: String, remote: Option<String>) -> Result<PushResult, GitError> {
    let repo_path = PathBuf::from(&path);
    let remote_name = remote.unwrap_or_else(|| "origin".to_string());
    match LocalGit::run_git(&repo_path, &["push", &remote_name, &format!("refs/tags/{}", name)]) {
        Ok(output) => Ok(PushResult {
            remote: remote_name,
            branch: name,
            success: true,
            message: output,
        }),
        Err(e) => Ok(PushResult {
            remote: remote_name,
            branch: name,
            success: false,
            message: e.to_string(),
        }),
    }
}

#[tauri::command]
pub fn git_push_tags(path: String, remote: Option<String>) -> Result<PushResult, GitError> {
    let repo_path = PathBuf::from(&path);
    let remote_name = remote.unwrap_or_else(|| "origin".to_string());
    let args = vec!["push", &remote_name, "--tags"];
    match LocalGit::run_git(&repo_path, &args) {
        Ok(output) => Ok(PushResult {
            remote: remote_name,
            branch: "--tags".to_string(),
            success: true,
            message: output,
        }),
        Err(e) => Ok(PushResult {
            remote: remote_name,
            branch: "--tags".to_string(),
            success: false,
            message: e.to_string(),
        }),
    }
}

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
