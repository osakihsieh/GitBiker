
use std::process::Command;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GitHubItem {
    pub number: i32,
    pub title: String,
    pub state: String,
    pub url: String,
    pub author: Option<GitHubAuthor>,
    pub updatedAt: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GitHubAuthor {
    pub login: String,
}

#[tauri::command]
pub async fn get_github_prs(repo_path: String) -> Result<Vec<GitHubItem>, String> {
    let output = Command::new("gh")
        .arg("pr")
        .arg("list")
        .arg("--limit").arg("20")
        .arg("--json").arg("number,title,state,url,author,updatedAt")
        .current_dir(repo_path)
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        let json_str = String::from_utf8_lossy(&output.stdout).to_string();
        let prs: Vec<GitHubItem> = serde_json::from_str(&json_str).map_err(|e| e.to_string())?;
        Ok(prs)
    } else {
        let err_msg = String::from_utf8_lossy(&output.stderr).to_string();
        Err(err_msg)
    }
}

#[tauri::command]
pub async fn get_github_issues(repo_path: String) -> Result<Vec<GitHubItem>, String> {
    let output = Command::new("gh")
        .arg("issue")
        .arg("list")
        .arg("--limit").arg("20")
        .arg("--json").arg("number,title,state,url,author,updatedAt")
        .current_dir(repo_path)
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        let json_str = String::from_utf8_lossy(&output.stdout).to_string();
        let issues: Vec<GitHubItem> = serde_json::from_str(&json_str).map_err(|e| e.to_string())?;
        Ok(issues)
    } else {
        let err_msg = String::from_utf8_lossy(&output.stderr).to_string();
        Err(err_msg)
    }
}
