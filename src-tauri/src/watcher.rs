use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;

use notify::RecursiveMode;
use notify_debouncer_full::{new_debouncer, DebouncedEvent, Debouncer, FileIdMap};
use tauri::{AppHandle, Emitter};

use crate::git::GitError;

type FileWatcher = Debouncer<notify::RecommendedWatcher, FileIdMap>;

pub struct WatcherState {
    inner: Mutex<Option<FileWatcher>>,
}

impl WatcherState {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(None),
        }
    }

    pub fn start(&self, repo_path: &str, app_handle: AppHandle) -> Result<(), GitError> {
        let mut guard = self.inner.lock().map_err(|e| {
            GitError::OperationFailed(format!("watcher lock poisoned: {e}"))
        })?;

        // 先停止舊的 watcher
        *guard = None;

        let git_dir = PathBuf::from(repo_path).join(".git");
        if !git_dir.exists() {
            return Err(GitError::NotARepo(repo_path.to_string()));
        }

        let handle = app_handle.clone();
        let mut debouncer = new_debouncer(
            Duration::from_millis(500),
            None,
            move |result: Result<Vec<DebouncedEvent>, Vec<notify::Error>>| {
                let events = match result {
                    Ok(evts) => evts,
                    Err(errs) => {
                        tracing::warn!("fs watcher errors: {:?}", errs);
                        return;
                    }
                };

                let change_kind = classify_events(&events);
                if let Some(kind) = change_kind {
                    tracing::debug!("git change detected: {kind}");
                    let _ = handle.emit("git-changed", kind);
                }
            },
        )
        .map_err(|e| GitError::OperationFailed(format!("無法建立 watcher: {e}")))?;

        debouncer
            .watch(git_dir.as_path(), RecursiveMode::Recursive)
            .map_err(|e| GitError::OperationFailed(format!("無法監聽 .git 目錄: {e}")))?;

        *guard = Some(debouncer);
        tracing::info!("開始監聽: {}", repo_path);
        Ok(())
    }

    pub fn stop(&self) -> Result<(), GitError> {
        let mut guard = self.inner.lock().map_err(|e| {
            GitError::OperationFailed(format!("watcher lock poisoned: {e}"))
        })?;
        *guard = None;
        tracing::info!("停止監聽");
        Ok(())
    }
}

/// 根據變更的檔案路徑分類事件類型
fn classify_events(events: &[DebouncedEvent]) -> Option<&'static str> {
    let mut has_refs = false;
    let mut has_index = false;
    let mut has_head = false;

    for event in events {
        for path in &event.paths {
            let path_str = path.to_string_lossy();

            // 忽略 lock 檔案（操作進行中的暫時文件）
            if path_str.ends_with(".lock") {
                continue;
            }

            if path_str.contains("refs") || path_str.contains("FETCH_HEAD") {
                has_refs = true;
            } else if path_str.contains("HEAD") {
                has_head = true;
            } else if path_str.contains("index") {
                has_index = true;
            }
        }
    }

    if has_refs || has_head {
        Some("refs")
    } else if has_index {
        Some("index")
    } else {
        None
    }
}
