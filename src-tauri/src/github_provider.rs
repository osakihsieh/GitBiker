
use std::process::Command;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct GitHubItem {
    pub id: i64,
    pub number: i32,
    pub title: String,
    pub state: String,
    pub url: String,
    pub author: String,
}

#[tauri::command]
pub async fn get_github_prs() -> Result<Vec<GitHubItem>, String> {
    let output = Command::new("gh")
        .arg("pr")
        .arg("list")
        .arg("--limit").arg("10")
        .arg("--json").arg("number,title,state,url,author")
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let json_str = String::from_utf8_lossy(&out.stdout).to_string();
            // 這裡簡化處理，實際建議用 serde_json 解析
            Ok(vec![]) 
        },
        _ => Err("無法獲取 PR 列表，請檢查 gh 登入狀態。".to_string()),
    }
}
