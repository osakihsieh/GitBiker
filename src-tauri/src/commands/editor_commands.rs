use std::path::PathBuf;
use std::process::Command;

#[derive(serde::Serialize, Clone)]
pub struct EditorInfo {
    pub id: String,
    pub name: String,
    pub command: String,
}

struct EditorCandidate {
    id: &'static str,
    name: &'static str,
    commands_win: &'static [&'static str],
    commands_unix: &'static [&'static str],
}

const KNOWN_EDITORS: &[EditorCandidate] = &[
    EditorCandidate {
        id: "vscode",
        name: "VS Code",
        commands_win: &["code.cmd", "code"],
        commands_unix: &["code"],
    },
    EditorCandidate {
        id: "cursor",
        name: "Cursor",
        commands_win: &["cursor.cmd", "cursor"],
        commands_unix: &["cursor"],
    },
    EditorCandidate {
        id: "zed",
        name: "Zed",
        commands_win: &["zed"],
        commands_unix: &["zed"],
    },
    EditorCandidate {
        id: "sublime",
        name: "Sublime Text",
        commands_win: &["subl.exe", "subl"],
        commands_unix: &["subl"],
    },
    EditorCandidate {
        id: "antigravity",
        name: "Antigravity",
        commands_win: &["antigravity.cmd", "antigravity"],
        commands_unix: &["antigravity"],
    },
];

fn find_editor_command(candidate: &EditorCandidate) -> Option<String> {
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

    #[cfg(target_os = "windows")]
    {
        if let Ok(local_app) = std::env::var("LOCALAPPDATA") {
            let extra_paths: Vec<PathBuf> = match candidate.id {
                "vscode" => vec![
                    PathBuf::from(&local_app).join("Programs/Microsoft VS Code/bin/code.cmd"),
                ],
                "cursor" => vec![
                    PathBuf::from(&local_app).join("Programs/cursor/resources/app/bin/cursor.cmd"),
                ],
                _ => vec![],
            };
            for p in &extra_paths {
                if p.exists() {
                    return Some(p.to_string_lossy().to_string());
                }
            }
        }
    }

    None
}

#[tauri::command]
pub fn detect_editors() -> Vec<EditorInfo> {
    KNOWN_EDITORS
        .iter()
        .filter_map(|c| {
            find_editor_command(c).map(|command| EditorInfo {
                id: c.id.to_string(),
                name: c.name.to_string(),
                command,
            })
        })
        .collect()
}

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
pub fn open_in_editor(path: String, editor: Option<String>) -> Result<(), String> {
    let path_ref = std::path::Path::new(&path);
    if !path_ref.exists() {
        return Err(format!("路徑不存在: {path}"));
    }

    // 1. User's preferred editor (from Settings UI — highest priority)
    if let Some(ref preferred) = editor {
        if !preferred.is_empty() && Command::new(preferred).arg(&path).spawn().is_ok() {
            return Ok(());
        }
    }

    // 2. VISUAL / EDITOR environment variables
    if let Ok(env_editor) = std::env::var("VISUAL").or_else(|_| std::env::var("EDITOR")) {
        if !env_editor.is_empty() && Command::new(&env_editor).arg(&path).spawn().is_ok() {
            return Ok(());
        }
    }

    // 3. Fallback: try common editors in order of preference
    let candidates = if cfg!(target_os = "windows") {
        vec!["code.cmd", "code", "cursor.cmd", "cursor", "zed", "subl"]
    } else {
        vec!["code", "cursor", "zed", "subl"]
    };

    for candidate in &candidates {
        if Command::new(candidate).arg(&path).spawn().is_ok() {
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
