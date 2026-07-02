use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;
use notify::RecursiveMode;
use notify_debouncer_full::new_debouncer;
use std::time::Duration;
use sysinfo::System;
use tauri::{AppHandle, Emitter, State};
use std::sync::{Arc, Mutex};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentStatus {
    pub profile: String,
    pub pid: Option<u32>,
    pub last_action: String,
    pub status: String, 
    pub last_update: u64,
    pub history: Vec<String>,
    pub worktree: Option<String>,
}

pub struct AgentMonitor {
    pub statuses: Arc<Mutex<Vec<AgentStatus>>>,
    app_handle: AppHandle,
}

pub struct AgentMonitorState(pub Arc<Mutex<Vec<AgentStatus>>>);

#[tauri::command]
pub fn get_agent_statuses(state: State<AgentMonitorState>) -> Vec<AgentStatus> {
    state.0.lock().unwrap().clone()
}

impl AgentMonitor {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            statuses: Arc::new(Mutex::new(Vec::new())),
            app_handle,
        }
    }

    pub fn start_monitoring(&self) {
        let statuses = self.statuses.clone();
        let app_handle = self.app_handle.clone();
        
        std::thread::spawn(move || {
            let hermes_path = PathBuf::from("/Users/peterhsieh/.hermes/profiles");
            if !hermes_path.exists() {
                return;
            }

            let (tx, rx) = std::sync::mpsc::channel();
            let mut debouncer = new_debouncer(Duration::from_millis(500), None, tx).unwrap();

            debouncer
                .watch(&hermes_path, RecursiveMode::Recursive)
                .unwrap();

            for res in rx {
                match res {
                    Ok(events) => {
                        for event in events {
                            if let Some(path) = event.paths.first() {
                                if path.extension().map_or(false, |ext| ext == "jsonl") {
                                    if let Some(mut status) = parse_jsonl(path) {
                                        update_status(&statuses, &app_handle, &mut status);
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => println!("watch error: {:?}", e),
                }
            }
        });

        let statuses_proc = self.statuses.clone();
        let app_handle_proc = self.app_handle.clone();
        std::thread::spawn(move || {
            let mut sys = System::new_all();
            loop {
                sys.refresh_all();
                {
                    let mut current_statuses = statuses_proc.lock().unwrap();
                    for status in current_statuses.iter_mut() {
                        let agent_proc = sys.processes().values().find(|p| {
                            let name = p.name().to_string_lossy().to_lowercase();
                            let cmd_line = p.cmd().iter().map(|arg| arg.to_string_lossy().to_lowercase()).collect::<Vec<_>>().join(" ");
                            
                            // Check for hermes, opencode, or claude-code
                            let is_agent_binary = name.contains("hermes") || name.contains("opencode") || name.contains("claude");
                            let is_profile_match = cmd_line.contains(&status.profile.to_lowercase());
                            
                            is_agent_binary && is_profile_match
                        });

                        if let Some(p) = agent_proc {
                            status.pid = Some(p.pid().as_u32());
                            if let Some(cwd) = p.cwd() {
                                status.worktree = Some(cwd.to_string_lossy().to_string());
                            }
                        } else {
                            status.pid = None;
                        }
                    }
                }
                let _ = app_handle_proc.emit("agent_statuses_update", statuses_proc.lock().unwrap().clone());
                std::thread::sleep(Duration::from_secs(5));
            }
        });
    }
}

fn parse_jsonl(path: &Path) -> Option<AgentStatus> {
    let profile = path.parent()?.parent()?.file_name()?.to_str()?.to_string();
    let content = fs::read_to_string(path).ok()?;
    let last_line = content.lines().last()?;
    let json: serde_json::Value = serde_json::from_str(last_line).ok()?;

    let mut status = "idle".to_string();
    let mut last_action = "Waiting for input...".to_string();

    if let Some(role) = json.get("role") {
        if role == "assistant" {
            if let Some(content_text) = json.get("content").and_then(|c| c.as_str()) {
                last_action = content_text.chars().take(100).collect::<String>();
                status = "thinking".to_string();
            }
            if let Some(tool_calls) = json.get("tool_calls").and_then(|t| t.as_array()) {
                if let Some(first_call) = tool_calls.first() {
                    let function_name = first_call.get("function").and_then(|f| f.get("name")).and_then(|n| n.as_str()).unwrap_or("");
                    status = match function_name {
                        "search_files" | "read_file" | "web_search" | "session_search" => "searching",
                        "patch" | "write_file" | "execute_code" => "patching",
                        "delegate_task" => "orchestrating",
                        _ => "working",
                    }.to_string();
                    last_action = format!("Executing {}", function_name);
                }
            }
        }
    }

    Some(AgentStatus {
        profile,
        pid: None, 
        last_action,
        status,
        last_update: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
        history: Vec::new(),
        worktree: None,
    })
}

fn update_status(statuses: &Arc<Mutex<Vec<AgentStatus>>>, app_handle: &AppHandle, new_status: &mut AgentStatus) {
    let mut current_statuses = statuses.lock().unwrap();
    if let Some(existing) = current_statuses.iter_mut().find(|s| s.profile == new_status.profile) {
        if existing.last_action != new_status.last_action {
            existing.history.push(existing.last_action.clone());
            if existing.history.len() > 10 {
                existing.history.remove(0);
            }
        }
        existing.last_action = new_status.last_action.clone();
        existing.status = new_status.status.clone();
        existing.last_update = new_status.last_update;
        // Don't overwrite PID here, let the process thread handle it
    } else {
        current_statuses.push(new_status.clone());
    }
    let _ = app_handle.emit("agent_statuses_update", current_statuses.clone());
}
