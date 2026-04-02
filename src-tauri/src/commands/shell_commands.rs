use std::path::PathBuf;
use std::process::Command;

use crate::git::GitError;

#[derive(serde::Serialize)]
pub struct ShellOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

#[tauri::command]
pub fn run_shell_command(path: String, command: String) -> Result<ShellOutput, GitError> {
    let cwd = PathBuf::from(&path);
    if !cwd.exists() {
        return Err(GitError::PathNotFound(path));
    }

    // Split command into program + args (simple shell-like splitting)
    let parts: Vec<&str> = command.split_whitespace().collect();
    if parts.is_empty() {
        return Err(GitError::OperationFailed("空命令".to_string()));
    }

    // Only allow git commands for safety
    let program = parts[0];
    if program != "git" {
        return Err(GitError::OperationFailed(
            "Inline terminal 目前只支援 git 命令。請使用外部終端機執行其他命令。".to_string(),
        ));
    }

    let output = Command::new(program)
        .args(&parts[1..])
        .current_dir(&cwd)
        .output()
        .map_err(|e| GitError::OperationFailed(format!("無法執行命令: {e}")))?;

    Ok(ShellOutput {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        exit_code: output.status.code().unwrap_or(-1),
    })
}
