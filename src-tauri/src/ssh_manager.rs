
use std::process::Command;
use std::fs;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct SshKey {
    pub name: String,
    pub path: String,
    pub key_type: String,
    pub public_key: String,
}

#[tauri::command]
pub async fn get_ssh_keys() -> Result<Vec<SshKey>, String> {
    let ssh_dir = dirs::home_dir()
        .ok_or("Could not find home directory")?
        .join(".ssh");
    
    if !ssh_dir.exists() {
        return Ok(vec![]);
    }

    let mut keys = Vec::new();
    if let Ok(entries) = fs::read_dir(ssh_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "pub" {
                    let name = path.file_stem().unwrap().to_string_lossy().to_string();
                    let public_key = fs::read_to_string(&path).unwrap_or_default();
                    let key_type = if public_key.contains("ed25519") { "ED25519" } 
                                  else if public_key.contains("rsa") { "RSA" } 
                                  else { "Unknown" };
                    
                    keys.push(SshKey {
                        name,
                        path: path.to_string_lossy().to_string(),
                        key_type: key_type.to_string(),
                        public_key,
                    });
                }
            }
        }
    }
    Ok(keys)
}

#[tauri::command]
pub async fn generate_ssh_key(name: String, email: String) -> Result<String, String> {
    let ssh_dir = dirs::home_dir()
        .ok_or("Could not find home directory")?
        .join(".ssh");
    
    if !ssh_dir.exists() {
        fs::create_dir_all(&ssh_dir).map_err(|e| e.to_string())?;
    }

    let key_path = ssh_dir.join(&name);
    if key_path.exists() {
        return Err("Key already exists".to_string());
    }

    let output = Command::new("ssh-keygen")
        .arg("-t").arg("ed25519")
        .arg("-C").arg(email)
        .arg("-f").arg(&key_path)
        .arg("-N").arg("") // No passphrase for speed
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(format!("Key {} generated successfully", name))
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[tauri::command]
pub async fn test_ssh_connection() -> Result<String, String> {
    let output = Command::new("ssh")
        .arg("-T")
        .arg("-o").arg("ConnectTimeout=5")
        .arg("git@github.com")
        .output();

    // Note: ssh -T returns exit code 1 for successful auth on GitHub
    match output {
        Ok(out) => {
            let msg = String::from_utf8_lossy(&out.stderr).to_string();
            if msg.contains("successfully authenticated") {
                Ok(msg)
            } else {
                Err(msg)
            }
        },
        Err(e) => Err(e.to_string())
    }
}
