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

export type LogFilter =
  | { type: 'Head' }
  | { type: 'All' }
  | { type: 'Branch'; value: string };

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

export interface CherryPickResult {
  commit_id: string;
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

export interface BranchCompareResult {
  base: string;
  compare: string;
  commits: Commit[];
  files: FileStatus[];
  ahead: number;
  behind: number;
}

export interface GitErrorResponse {
  code: string;
  message: string;
}

// ── Conflict Resolution Types ─────────────────────────

export type ConflictType = 'Content' | 'DeleteModify' | 'AddAdd';

export interface ConflictFile {
  path: string;
  conflict_type: ConflictType;
  is_binary: boolean;
}

export interface ConflictHunk {
  index: number;
  ours: string;
  theirs: string;
  base: string | null;
  start_line: number;
}

export type ConflictSegment =
  | { type: 'Context'; value: string }
  | { type: 'Hunk'; value: ConflictHunk };

export interface ConflictContent {
  path: string;
  segments: ConflictSegment[];
  hunk_count: number;
  content_hash: string;
  parse_error: string | null;
}

export interface MergeDryRunResult {
  has_conflicts: boolean;
  conflict_files: string[];
  method: string;
}

export interface MergeCompleteResult {
  commit_hash: string;
}

export type ResolveChoice = 'Ours' | 'Theirs';
