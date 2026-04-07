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
            commands::git_rename_branch,
            commands::git_checkout_remote_branch,
            commands::git_branch_merge_status,
            commands::git_merge_branch,
            commands::git_merge_abort,
            commands::git_branch_compare,
            commands::git_merge_dry_run,
            commands::git_get_conflict_files,
            commands::git_get_conflict_content,
            commands::git_resolve_conflict_content,
            commands::git_resolve_conflict_choice,
            commands::git_complete_merge,
            commands::git_stash_list,
            commands::git_stash_push,
            commands::git_stash_push_files,
            commands::git_stash_pop,
            commands::git_stash_apply,
            commands::git_stash_drop,
            commands::git_init,
            commands::git_revert,
            commands::scan_git_repos,
            commands::git_stage_hunk,
            commands::git_unstage_hunk,
            commands::git_stash_hunk,
            commands::git_cherry_pick,
            commands::git_cherry_pick_abort,
            commands::git_cherry_pick_continue,
            commands::git_reset_soft,
            commands::git_reset_hard,
            commands::git_file_log,
            commands::git_clone,
            commands::check_git_version,
            commands::start_watching,
            commands::stop_watching,
            commands::git_ignore,
            commands::git_checkout_file,
            commands::git_show_file_diff,
            commands::git_show_files,
            commands::git_log_search,
            commands::git_remote_list,
            commands::git_remote_add,
            commands::git_remote_remove,
            commands::git_remote_rename,
            commands::git_tag_create,
            commands::git_fetch,
            commands::open_in_folder,
            commands::open_in_editor,
            commands::open_in_terminal,
            commands::detect_editors,
            commands::run_shell_command,
        ])
        .run(tauri::generate_context!())
        .expect("啟動 GitBiker 失敗");
}
