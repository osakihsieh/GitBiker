use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ── Conflict Resolution Types ─────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConflictType {
    Content,
    DeleteModify,
    AddAdd,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictFile {
    pub path: String,
    pub conflict_type: ConflictType,
    pub is_binary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictHunk {
    pub index: usize,
    pub ours: String,
    pub theirs: String,
    pub base: Option<String>,
    pub start_line: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum ConflictSegment {
    Context(String),
    Hunk(ConflictHunk),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictContent {
    pub path: String,
    pub segments: Vec<ConflictSegment>,
    pub hunk_count: usize,
    pub content_hash: String,
    pub parse_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeDryRunResult {
    pub has_conflicts: bool,
    pub conflict_files: Vec<String>,
    pub method: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeCompleteResult {
    pub commit_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ResolveChoice {
    Ours,
    Theirs,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "value")]
pub enum LogFilter {
    Head,
    All,
    Branch(String),
}

// ── Existing Types ────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FileStatusKind {
    Modified,
    Added,
    Deleted,
    Renamed,
    Copied,
    Untracked,
    Ignored,
    Conflicted,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StagingState {
    Staged,
    Unstaged,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileStatus {
    pub path: PathBuf,
    pub kind: FileStatusKind,
    pub staging: StagingState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitRef {
    pub name: String,
    pub kind: RefKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RefKind {
    Local,
    Remote,
    Tag,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commit {
    pub id: String,
    pub message: String,
    pub author: String,
    pub email: String,
    pub timestamp: i64,
    pub parents: Vec<String>,
    pub refs: Vec<CommitRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteInfo {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffHunk {
    pub header: String,
    pub lines: Vec<DiffLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffLine {
    pub kind: DiffLineKind,
    pub content: String,
    pub old_lineno: Option<u32>,
    pub new_lineno: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DiffLineKind {
    Context,
    Addition,
    Deletion,
    Header,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffResult {
    pub file_path: PathBuf,
    pub hunks: Vec<DiffHunk>,
    pub stats: DiffStats,
    pub is_binary: bool,
    pub is_truncated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffStats {
    pub additions: usize,
    pub deletions: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branch {
    pub name: String,
    pub is_current: bool,
    pub is_remote: bool,
    pub upstream: Option<String>,
    pub commit_id: Option<String>,
    pub ahead: Option<u32>,
    pub behind: Option<u32>,
    pub last_commit_timestamp: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchMergeStatus {
    pub merged: bool,
    pub unmerged_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushResult {
    pub remote: String,
    pub branch: String,
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeResult {
    pub branch: String,
    pub success: bool,
    pub message: String,
    pub conflicts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StashEntry {
    pub index: usize,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullResult {
    pub remote: String,
    pub branch: String,
    pub success: bool,
    pub message: String,
    pub conflicts: Vec<String>,
}
