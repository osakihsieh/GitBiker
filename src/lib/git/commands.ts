import { invoke } from '@tauri-apps/api/core';
import type {
  FileStatus,
  Commit,
  DiffResult,
  Branch,
  PushResult,
  PullResult,
  RemoteInfo,
  BranchMergeStatus,
  MergeResult,
  StashEntry,
} from './types';

export async function gitStatus(path: string): Promise<FileStatus[]> {
  return invoke('git_status', { path });
}

export async function gitLog(path: string, limit?: number): Promise<Commit[]> {
  return invoke('git_log', { path, limit: limit ?? 100 });
}

export async function gitDiff(path: string, file: string): Promise<DiffResult> {
  return invoke('git_diff', { path, file });
}

export async function gitStage(path: string, files: string[]): Promise<void> {
  return invoke('git_stage', { path, files });
}

export async function gitUnstage(path: string, files: string[]): Promise<void> {
  return invoke('git_unstage', { path, files });
}

export async function gitCommit(path: string, message: string): Promise<string> {
  return invoke('git_commit', { path, message });
}

export async function gitPush(
  path: string,
  remote?: string,
  branch?: string
): Promise<PushResult> {
  return invoke('git_push', { path, remote, branch });
}

export async function gitPull(
  path: string,
  remote?: string,
  branch?: string
): Promise<PullResult> {
  return invoke('git_pull', { path, remote, branch });
}

export async function gitBranches(path: string): Promise<Branch[]> {
  return invoke('git_branches', { path });
}

export async function gitSwitchBranch(path: string, name: string): Promise<void> {
  return invoke('git_switch_branch', { path, name });
}

export async function gitCreateBranch(path: string, name: string): Promise<void> {
  return invoke('git_create_branch', { path, name });
}

export async function gitDeleteBranch(path: string, name: string, force = false): Promise<void> {
  return invoke('git_delete_branch', { path, name, force });
}

export async function gitRenameBranch(path: string, oldName: string, newName: string): Promise<void> {
  return invoke('git_rename_branch', { path, oldName, newName });
}

export async function gitCheckoutRemoteBranch(path: string, remoteBranch: string): Promise<string> {
  return invoke('git_checkout_remote_branch', { path, remoteBranch });
}

export async function gitBranchMergeStatus(path: string, branchName: string, base?: string): Promise<BranchMergeStatus> {
  return invoke('git_branch_merge_status', { path, branchName, base });
}

// ── Merge ────────────────────────────────────────────

export async function gitMergeBranch(path: string, branchName: string): Promise<MergeResult> {
  return invoke('git_merge_branch', { path, branchName });
}

export async function gitMergeAbort(path: string): Promise<void> {
  return invoke('git_merge_abort', { path });
}

// ── Stash ────────────────────────────────────────────

export async function gitStashList(path: string): Promise<StashEntry[]> {
  return invoke('git_stash_list', { path });
}

export async function gitStashPush(path: string, message?: string): Promise<string> {
  return invoke('git_stash_push', { path, message });
}

export async function gitStashPop(path: string, index?: number): Promise<string> {
  return invoke('git_stash_pop', { path, index });
}

export async function gitStashApply(path: string, index?: number): Promise<string> {
  return invoke('git_stash_apply', { path, index });
}

export async function gitStashDrop(path: string, index: number): Promise<string> {
  return invoke('git_stash_drop', { path, index });
}

export async function startWatching(path: string): Promise<void> {
  return invoke('start_watching', { path });
}

export async function stopWatching(): Promise<void> {
  return invoke('stop_watching');
}

// ── Commit Detail + Search ────────────────────────────

export async function gitShowFileDiff(path: string, commitId: string, file: string): Promise<DiffResult> {
  return invoke('git_show_file_diff', { path, commitId, file });
}

export async function gitShowFiles(path: string, commitId: string): Promise<FileStatus[]> {
  return invoke('git_show_files', { path, commitId });
}

export async function gitLogSearch(
  path: string,
  query: string,
  searchType: string,
  limit?: number,
): Promise<Commit[]> {
  return invoke('git_log_search', { path, query, searchType, limit });
}

// ── Remote Management ────────────────────────────────

export async function gitRemoteList(path: string): Promise<RemoteInfo[]> {
  return invoke('git_remote_list', { path });
}

export async function gitRemoteAdd(path: string, name: string, url: string): Promise<void> {
  return invoke('git_remote_add', { path, name, url });
}

export async function gitRemoteRemove(path: string, name: string): Promise<void> {
  return invoke('git_remote_remove', { path, name });
}

export async function gitRemoteRename(path: string, oldName: string, newName: string): Promise<void> {
  return invoke('git_remote_rename', { path, oldName, newName });
}

// ── Tag + Fetch ──────────────────────────────────────

export async function gitTagCreate(path: string, name: string, commitId?: string): Promise<void> {
  return invoke('git_tag_create', { path, name, commitId });
}

export async function gitFetch(path: string, remote?: string): Promise<string> {
  return invoke('git_fetch', { path, remote });
}

// ── File Operations ───────────────────────────────────

export async function gitIgnore(path: string, pattern: string): Promise<void> {
  return invoke('git_ignore', { path, pattern });
}

export async function gitCheckoutFile(
  path: string,
  file: string,
  staging: string,
  kind: string,
): Promise<void> {
  return invoke('git_checkout_file', { path, file, staging, kind });
}

// ── Editor Detection ─────────────────────────────────

export interface EditorInfo {
  id: string;
  name: string;
  command: string;
}

export async function detectEditors(): Promise<EditorInfo[]> {
  return invoke('detect_editors');
}

// ── External Tools ────────────────────────────────────

export async function openInFolder(path: string): Promise<void> {
  return invoke('open_in_folder', { path });
}

export async function openInEditor(path: string, editor?: string): Promise<void> {
  return invoke('open_in_editor', { path, editor: editor ?? null });
}

export async function openInTerminal(path: string): Promise<void> {
  return invoke('open_in_terminal', { path });
}
