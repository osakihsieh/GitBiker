import { invoke } from '@tauri-apps/api/core';
import type {
  FileStatus,
  Commit,
  DiffResult,
  Branch,
  PushResult,
  PullResult
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

export async function gitDeleteBranch(path: string, name: string): Promise<void> {
  return invoke('git_delete_branch', { path, name });
}

export async function startWatching(path: string): Promise<void> {
  return invoke('start_watching', { path });
}

export async function stopWatching(): Promise<void> {
  return invoke('stop_watching');
}

// ── External Tools ────────────────────────────────────

export async function openInFolder(path: string): Promise<void> {
  return invoke('open_in_folder', { path });
}

export async function openInEditor(path: string): Promise<void> {
  return invoke('open_in_editor', { path });
}

export async function openInTerminal(path: string): Promise<void> {
  return invoke('open_in_terminal', { path });
}
