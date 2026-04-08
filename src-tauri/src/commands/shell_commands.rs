use std::path::PathBuf;
use std::process::Command;

use crate::git::GitError;

#[derive(serde::Serialize)]
pub struct ShellOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

#[derive(serde::Serialize, Clone)]
pub struct ShellInfo {
    pub id: String,
    pub name: String,
    pub command: String,
}

struct ShellCandidate {
    id: &'static str,
    name: &'static str,
    commands_win: &'static [&'static str],
    commands_unix: &'static [&'static str],
}

const KNOWN_SHELLS: &[ShellCandidate] = &[
    ShellCandidate {
        id: "git-bash",
        name: "Git Bash",
        commands_win: &["bash.exe"],
        commands_unix: &[],
    },
    ShellCandidate {
        id: "powershell-core",
        name: "PowerShell",
        commands_win: &["pwsh.exe", "pwsh"],
        commands_unix: &["pwsh"],
    },
    ShellCandidate {
        id: "powershell",
        name: "Windows PowerShell",
        commands_win: &["powershell.exe"],
        commands_unix: &[],
    },
    ShellCandidate {
        id: "cmd",
        name: "CMD",
        commands_win: &["cmd.exe"],
        commands_unix: &[],
    },
    ShellCandidate {
        id: "bash",
        name: "Bash",
        commands_win: &[],
        commands_unix: &["bash"],
    },
    ShellCandidate {
        id: "zsh",
        name: "Zsh",
        commands_win: &[],
        commands_unix: &["zsh"],
    },
    ShellCandidate {
        id: "fish",
        name: "Fish",
        commands_win: &[],
        commands_unix: &["fish"],
    },
];

fn find_shell_command(candidate: &ShellCandidate) -> Option<String> {
    let commands = if cfg!(target_os = "windows") {
        candidate.commands_win
    } else {
        candidate.commands_unix
    };

    for cmd in commands {
        if which::which(cmd).is_ok() {
            return Some(cmd.to_string());
        }
    }

    // Windows: check common install paths for Git Bash
    #[cfg(target_os = "windows")]
    if candidate.id == "git-bash" {
        let extra_paths = [
            "C:\\Program Files\\Git\\bin\\bash.exe",
            "C:\\Program Files (x86)\\Git\\bin\\bash.exe",
        ];
        for p in &extra_paths {
            if std::path::Path::new(p).exists() {
                return Some(p.to_string());
            }
        }
    }

    None
}

#[tauri::command]
pub fn detect_shells() -> Vec<ShellInfo> {
    KNOWN_SHELLS
        .iter()
        .filter_map(|c| {
            find_shell_command(c).map(|command| ShellInfo {
                id: c.id.to_string(),
                name: c.name.to_string(),
                command,
            })
        })
        .collect()
}

/// Build a Command that runs `user_cmd` through the given shell.
fn build_shell_command(shell_id: &str, shell_cmd: &str, user_cmd: &str, cwd: &PathBuf) -> Command {
    let mut cmd = match shell_id {
        "cmd" => {
            let mut c = Command::new(shell_cmd);
            c.args(["/C", user_cmd]);
            c
        }
        "powershell" | "powershell-core" => {
            let mut c = Command::new(shell_cmd);
            c.args(["-NoProfile", "-Command", user_cmd]);
            c
        }
        // bash, git-bash, zsh, fish, etc.
        _ => {
            let mut c = Command::new(shell_cmd);
            c.args(["-c", user_cmd]);
            c
        }
    };

    cmd.current_dir(cwd);

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }

    cmd
}

#[tauri::command]
pub fn run_shell_command(
    path: String,
    command: String,
    shell: Option<String>,
) -> Result<ShellOutput, GitError> {
    let cwd = PathBuf::from(&path);
    if !cwd.exists() {
        return Err(GitError::PathNotFound(path));
    }

    let cmd_trimmed = command.trim();
    if cmd_trimmed.is_empty() {
        return Err(GitError::OperationFailed("空命令".to_string()));
    }

    // If a shell is specified, use it; otherwise fall back to git-only mode
    let output = if let Some(ref shell_id) = shell {
        // Find the shell command from detected shells
        let shell_info = detect_shells()
            .into_iter()
            .find(|s| s.id == *shell_id)
            .ok_or_else(|| {
                GitError::OperationFailed(format!("找不到指定的 Shell: {shell_id}"))
            })?;

        build_shell_command(shell_id, &shell_info.command, cmd_trimmed, &cwd)
            .output()
            .map_err(|e| GitError::OperationFailed(format!("無法執行命令: {e}")))?
    } else {
        // Legacy git-only mode (no shell selected)
        let parts: Vec<&str> = cmd_trimmed.split_whitespace().collect();
        if parts.is_empty() {
            return Err(GitError::OperationFailed("空命令".to_string()));
        }

        let program = parts[0];
        if program != "git" {
            return Err(GitError::OperationFailed(
                "請先在設定中選擇終端機 Shell，以支援非 Git 命令。".to_string(),
            ));
        }

        crate::git::LocalGit::git_command()
            .args(&parts[1..])
            .current_dir(&cwd)
            .output()
            .map_err(|e| GitError::OperationFailed(format!("無法執行命令: {e}")))?
    };

    Ok(ShellOutput {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        exit_code: output.status.code().unwrap_or(-1),
    })
}
