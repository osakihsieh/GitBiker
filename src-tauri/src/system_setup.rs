
use std::process::Command;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ToolStatus {
    pub name: String,
    pub installed: bool,
    pub version: Option<String>,
}

#[tauri::command]
pub async fn check_gh_status() -> Result<ToolStatus, String> {
    let output = Command::new("gh")
        .arg("--version")
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let version = String::from_utf8_lossy(&out.stdout).to_string();
            Ok(ToolStatus {
                name: "gh".to_string(),
                installed: true,
                version: Some(version),
            })
        },
        _ => Ok(ToolStatus {
            name: "gh".to_string(),
            installed: false,
            version: None,
        }),
    }
}

#[tauri::command]
pub async fn install_gh() -> Result<String, String> {
    let os = std::env::consts::OS;
    
    match os {
        "macos" => {
            // Check for brew
            if Command::new("brew").arg("--version").output().is_err() {
                return Err("MacOS 建議先安裝 Homebrew 才能自動安裝 gh。".to_string());
            }
            let output = Command::new("brew").arg("install").arg("gh").output().map_err(|e| e.to_string())?;
            if output.status.success() { Ok("GitHub CLI 已透過 Homebrew 安裝成功！".to_string()) }
            else { Err(String::from_utf8_lossy(&output.stderr).to_string()) }
        },
        "windows" => {
            // Try winget (Windows 10/11 default)
            let output = Command::new("winget")
                .arg("install")
                .arg("--id").arg("Microsoft.GitHubCLI")
                .arg("--source").arg("winget")
                .arg("--silent")
                .arg("--accept-package-agreements")
                .output()
                .map_err(|e| e.to_string())?;
            
            if output.status.success() { Ok("GitHub CLI 已透過 winget 安裝成功！".to_string()) }
            else { Err("winget 安裝失敗，請嘗試手動安裝。".to_string()) }
        },
        "linux" => {
            // Linux is complex, try to detect dnf or apt
            if Command::new("dnf").arg("--version").output().is_ok() {
                // Fedora/RHEL
                return Ok("偵測到 Fedora/RHEL，請手動執行: sudo dnf install gh".to_string());
            } else if Command::new("apt").arg("--version").output().is_ok() {
                // Ubuntu/Debian
                return Ok("偵測到 Ubuntu/Debian，請參考官方說明手動安裝 (需新增 repo)。".to_string());
            }
            Err("Linux 環境多樣，請依照您的發行版手動安裝 GitHub CLI。".to_string())
        },
        _ => Err(format!("目前不支援在 {} 環境下自動安裝。", os)),
    }
}
