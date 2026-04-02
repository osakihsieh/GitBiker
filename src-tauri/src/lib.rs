mod commands;
mod git;
mod watcher;

use commands::git_commands::GitState;
use git::LocalGit;
use watcher::WatcherState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "gitbiker=info".into()),
        )
        .init();

    tracing::info!("GitBiker 啟動中...");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .manage(GitState {
            git: LocalGit::new(),
        })
        .manage(WatcherState::new())
        .invoke_handler(tauri::generate_handler![
            commands::git_status,
            commands::git_log,
            commands::git_diff,
            commands::git_stage,
            commands::git_unstage,
            commands::git_commit,
            commands::git_push,
            commands::git_pull,
            commands::git_branches,
            commands::git_switch_branch,
            commands::git_create_branch,
            commands::git_delete_branch,
            commands::git_clone,
            commands::check_git_version,
            commands::start_watching,
            commands::stop_watching,
            commands::open_in_folder,
            commands::open_in_editor,
            commands::open_in_terminal,
        ])
        .run(tauri::generate_context!())
        .expect("啟動 GitBiker 失敗");
}
