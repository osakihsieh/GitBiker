export type FileStatusKind =
  | 'Modified'
  | 'Added'
  | 'Deleted'
  | 'Renamed'
  | 'Copied'
  | 'Untracked'
  | 'Ignored'
  | 'Conflicted'
  | 'Unknown';

export type StagingState = 'Staged' | 'Unstaged';

export interface FileStatus {
  path: string;
  kind: FileStatusKind;
  staging: StagingState;
}

export type RefKind = 'Local' | 'Remote' | 'Tag';

export interface CommitRef {
  name: string;
  kind: RefKind;
}

export interface Commit {
  id: string;
  message: string;
  author: string;
  email: string;
  timestamp: number;
  parents: string[];
  refs: CommitRef[];
}

export interface RemoteInfo {
  name: string;
  url: string;
}

export type DiffLineKind = 'Context' | 'Addition' | 'Deletion' | 'Header';

export interface DiffLine {
  kind: DiffLineKind;
  content: string;
  old_lineno: number | null;
  new_lineno: number | null;
}

export interface DiffHunk {
  header: string;
  lines: DiffLine[];
}

export interface DiffStats {
  additions: number;
  deletions: number;
}

export interface DiffResult {
  file_path: string;
  hunks: DiffHunk[];
  stats: DiffStats;
  is_binary: boolean;
  is_truncated: boolean;
}

export interface Branch {
  name: string;
  is_current: boolean;
  is_remote: boolean;
  upstream: string | null;
  commit_id: string | null;
  ahead: number | null;
  behind: number | null;
  last_commit_timestamp: number | null;
}

export interface BranchMergeStatus {
  merged: boolean;
  unmerged_count: number;
}

export interface MergeResult {
  branch: string;
  success: boolean;
  message: string;
  conflicts: string[];
}

export interface StashEntry {
  index: number;
  message: string;
}

export interface PushResult {
  remote: string;
  branch: string;
  success: boolean;
  message: string;
}

export interface PullResult {
  remote: string;
  branch: string;
  success: boolean;
  message: string;
  conflicts: string[];
}

export interface GitErrorResponse {
  code: string;
  message: string;
}
