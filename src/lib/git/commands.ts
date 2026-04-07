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
  CherryPickResult,
  StashEntry,
  ConflictFile,
  ConflictContent,
  MergeDryRunResult,
  MergeCompleteResult,
  ResolveChoice,
} from './types';

export async function gitStatus(path: string): Promise<FileStatus[]> {
  return invoke('git_status', { path });
}

export async function gitLog(path: string, limit?: number, filter?: LogFilter): Promise<Commit[]> {
  return invoke('git_log', { path, limit, filter });
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

// ── Hunk-level staging ──────────────────────────────

export async function gitStageHunk(path: string, patch: string): Promise<void> {
  return invoke('git_stage_hunk', { path, patch });
}

export async function gitUnstageHunk(path: string, patch: string): Promise<void> {
  return invoke('git_unstage_hunk', { path, patch });
}

export async function gitStashHunk(path: string, patch: string, message?: string): Promise<string> {
  return invoke('git_stash_hunk', { path, patch, message: message ?? null });
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

// ── Conflict Resolution ─────────────────────────────

export async function gitMergeDryRun(path: string, branchName: string): Promise<MergeDryRunResult> {
  return invoke('git_merge_dry_run', { path, branchName });
}

export async function gitGetConflictFiles(path: string): Promise<ConflictFile[]> {
  return invoke('git_get_conflict_files', { path });
}

export async function gitGetConflictContent(path: string, filePath: string): Promise<ConflictContent> {
  return invoke('git_get_conflict_content', { path, filePath });
}

export async function gitResolveConflictContent(
  path: string,
  filePath: string,
  resolvedContent: string,
  contentHash: string,
): Promise<void> {
  return invoke('git_resolve_conflict_content', { path, filePath, resolvedContent, contentHash });
}

export async function gitResolveConflictChoice(
  path: string,
  filePath: string,
  choice: ResolveChoice,
): Promise<void> {
  return invoke('git_resolve_conflict_choice', { path, filePath, choice });
}

export async function gitCompleteMerge(path: string, message?: string): Promise<MergeCompleteResult> {
  return invoke('git_complete_merge', { path, message });
}

// ── Stash ────────────────────────────────────────────

export async function gitStashList(path: string): Promise<StashEntry[]> {
  return invoke('git_stash_list', { path });
}

export async function gitStashPush(path: string, message?: string): Promise<string> {
  return invoke('git_stash_push', { path, message: message ?? null });
}

export async function gitStashPushFiles(path: string, files: string[], message?: string): Promise<string> {
  return invoke('git_stash_push_files', { path, files, message: message ?? null });
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

// ── Init / Revert / Reset / File History ─────────────

export async function gitInit(path: string): Promise<void> {
  return invoke('git_init', { path });
}

export async function gitRevert(path: string, commitId: string, isMerge: boolean): Promise<string> {
  return invoke('git_revert', { path, commitId, isMerge });
}

// ── Cherry-pick ─────────────────────────────────────

export async function gitCherryPick(path: string, commitId: string): Promise<CherryPickResult> {
  return invoke('git_cherry_pick', { path, commitId });
}

export async function gitCherryPickAbort(path: string): Promise<void> {
  return invoke('git_cherry_pick_abort', { path });
}

export async function gitCherryPickContinue(path: string): Promise<string> {
  return invoke('git_cherry_pick_continue', { path });
}

export async function gitResetSoft(path: string, target: string): Promise<void> {
  return invoke('git_reset_soft', { path, target });
}

export async function gitResetHard(path: string, target: string): Promise<void> {
  return invoke('git_reset_hard', { path, target });
}

export async function gitFileLog(path: string, file: string, limit?: number): Promise<Commit[]> {
  return invoke('git_file_log', { path, file, limit: limit ?? 200 });
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

// ── Multi-repo ──────────────────────────────────────

export async function scanGitRepos(path: string): Promise<string[]> {
  return invoke('scan_git_repos', { path });
}

export async function gitBranchCompare(path: string, base: string, compare: string): Promise<BranchCompareResult> {
  return invoke('git_branch_compare', { path, base, compare });
}
