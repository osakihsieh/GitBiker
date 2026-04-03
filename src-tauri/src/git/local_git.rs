use std::path::{Path, PathBuf};
use std::process::Command;

use git2::{DiffOptions, Repository, StatusOptions};
use sha2::{Digest, Sha256};

use crate::git::error::GitError;
use crate::git::operations::GitOperations;
use crate::git::types::*;

const MAX_DIFF_BYTES: usize = 10 * 1024 * 1024; // 10MB
const MAX_CONFLICT_FILE_BYTES: usize = 1024 * 1024; // 1MB

pub struct LocalGit;

impl LocalGit {
    pub fn new() -> Self {
        Self
    }

    fn open_repo(path: &Path) -> Result<Repository, GitError> {
        if !path.exists() {
            return Err(GitError::PathNotFound(path.display().to_string()));
        }
        Repository::open(path).map_err(|_| GitError::NotARepo(path.display().to_string()))
    }

    pub(crate) fn check_index_lock(path: &Path) -> Result<(), GitError> {
        let lock_path = path.join(".git/index.lock");
        if lock_path.exists() {
            return Err(GitError::IndexLocked);
        }
        Ok(())
    }

    pub(crate) fn run_git(path: &Path, args: &[&str]) -> Result<String, GitError> {
        let output = Command::new("git")
            .args(args)
            .current_dir(path)
            .output()
            .map_err(|e| GitError::OperationFailed(format!("無法執行 git: {e}")))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            if stderr.contains("Authentication") || stderr.contains("could not read Username") {
                Err(GitError::AuthError(stderr))
            } else if stderr.contains("timeout") || stderr.contains("timed out") {
                Err(GitError::NetworkTimeout(stderr))
            } else {
                Err(GitError::OperationFailed(stderr))
            }
        }
    }

    fn map_status_kind(status: git2::Status) -> (FileStatusKind, StagingState) {
        if status.is_index_new() {
            (FileStatusKind::Added, StagingState::Staged)
        } else if status.is_index_modified() {
            (FileStatusKind::Modified, StagingState::Staged)
        } else if status.is_index_deleted() {
            (FileStatusKind::Deleted, StagingState::Staged)
        } else if status.is_index_renamed() {
            (FileStatusKind::Renamed, StagingState::Staged)
        } else if status.is_wt_new() {
            (FileStatusKind::Untracked, StagingState::Unstaged)
        } else if status.is_wt_modified() {
            (FileStatusKind::Modified, StagingState::Unstaged)
        } else if status.is_wt_deleted() {
            (FileStatusKind::Deleted, StagingState::Unstaged)
        } else if status.is_wt_renamed() {
            (FileStatusKind::Renamed, StagingState::Unstaged)
        } else if status.is_conflicted() {
            (FileStatusKind::Conflicted, StagingState::Unstaged)
        } else if status.is_ignored() {
            (FileStatusKind::Ignored, StagingState::Unstaged)
        } else {
            (FileStatusKind::Unknown, StagingState::Unstaged)
        }
    }
}

// ── Branch management (CLI-based, not on trait) ──────────

impl LocalGit {
    pub fn force_delete_branch(&self, path: &Path, name: &str) -> Result<(), GitError> {
        Self::check_index_lock(path)?;
        Self::run_git(path, &["branch", "-D", name])?;
        Ok(())
    }

    pub fn rename_branch(&self, path: &Path, old_name: &str, new_name: &str) -> Result<(), GitError> {
        Self::check_index_lock(path)?;
        Self::run_git(path, &["branch", "-m", old_name, new_name])?;
        Ok(())
    }

    pub fn checkout_remote_branch(&self, path: &Path, remote_branch: &str) -> Result<String, GitError> {
        Self::check_index_lock(path)?;
        // remote_branch is like "origin/feature-x", extract local name
        let local_name = remote_branch
            .split('/')
            .skip(1)
            .collect::<Vec<_>>()
            .join("/");
        if local_name.is_empty() {
            return Err(GitError::OperationFailed(
                format!("無效的 remote branch 名稱: {remote_branch}"),
            ));
        }
        Self::run_git(path, &["checkout", "-b", &local_name, "--track", remote_branch])?;
        Ok(local_name)
    }

    pub fn merge_branch(&self, path: &Path, branch_name: &str) -> Result<MergeResult, GitError> {
        Self::check_index_lock(path)?;
        match Self::run_git(path, &["merge", "--no-edit", branch_name]) {
            Ok(output) => Ok(MergeResult {
                branch: branch_name.to_string(),
                success: true,
                message: output,
                conflicts: Vec::new(),
            }),
            Err(GitError::OperationFailed(stderr)) => {
                // Check if this is a merge conflict
                if stderr.contains("CONFLICT") || stderr.contains("Automatic merge failed") {
                    let conflicts: Vec<String> = stderr
                        .lines()
                        .filter(|l| l.contains("CONFLICT"))
                        .map(|l| l.to_string())
                        .collect();
                    Ok(MergeResult {
                        branch: branch_name.to_string(),
                        success: false,
                        message: stderr,
                        conflicts,
                    })
                } else {
                    Err(GitError::OperationFailed(stderr))
                }
            }
            Err(e) => Err(e),
        }
    }

    pub fn merge_abort(&self, path: &Path) -> Result<(), GitError> {
        Self::run_git(path, &["merge", "--abort"])?;
        Ok(())
    }

    pub fn stash_list(&self, path: &Path) -> Result<Vec<StashEntry>, GitError> {
        let output = Self::run_git(path, &["stash", "list", "--format=%gs"])?;
        let entries: Vec<StashEntry> = output
            .lines()
            .enumerate()
            .filter(|(_, l)| !l.is_empty())
            .map(|(i, l)| StashEntry {
                index: i,
                message: l.to_string(),
            })
            .collect();
        Ok(entries)
    }

    pub fn stash_push(&self, path: &Path, message: Option<&str>) -> Result<String, GitError> {
        let mut args = vec!["stash", "push"];
        if let Some(msg) = message {
            args.push("-m");
            args.push(msg);
        }
        Self::run_git(path, &args)
    }

    pub fn stash_pop(&self, path: &Path, index: usize) -> Result<String, GitError> {
        let stash_ref = format!("stash@{{{index}}}");
        Self::run_git(path, &["stash", "pop", &stash_ref])
    }

    pub fn stash_apply(&self, path: &Path, index: usize) -> Result<String, GitError> {
        let stash_ref = format!("stash@{{{index}}}");
        Self::run_git(path, &["stash", "apply", &stash_ref])
    }

    pub fn stash_drop(&self, path: &Path, index: usize) -> Result<String, GitError> {
        let stash_ref = format!("stash@{{{index}}}");
        Self::run_git(path, &["stash", "drop", &stash_ref])
    }

    pub fn branch_merge_status(&self, path: &Path, branch_name: &str, base: &str) -> Result<BranchMergeStatus, GitError> {
        let range = format!("{base}..{branch_name}");
        let output = Self::run_git(path, &["rev-list", "--count", &range])?;
        let count: usize = output.trim().parse().unwrap_or(0);
        Ok(BranchMergeStatus {
            merged: count == 0,
            unmerged_count: count,
        })
    }
}

// ── Conflict Resolution ──────────────────────────────

impl LocalGit {
    /// Dry-run merge using `git merge-tree` (Git >= 2.38).
    /// Falls back to skipping if Git is too old.
    pub fn merge_dry_run(&self, path: &Path, branch_name: &str) -> Result<MergeDryRunResult, GitError> {
        // Check git version for merge-tree --write-tree support
        let version_output = Self::run_git(path, &["--version"])?;
        let supports_merge_tree = Self::git_version_at_least(&version_output, 2, 38);

        if !supports_merge_tree {
            return Ok(MergeDryRunResult {
                has_conflicts: false,
                conflict_files: Vec::new(),
                method: "skipped".to_string(),
            });
        }

        // git merge-tree --write-tree HEAD branch_name
        match Self::run_git(path, &["merge-tree", "--write-tree", "HEAD", branch_name]) {
            Ok(_) => Ok(MergeDryRunResult {
                has_conflicts: false,
                conflict_files: Vec::new(),
                method: "merge-tree".to_string(),
            }),
            Err(GitError::OperationFailed(stderr)) => {
                // merge-tree outputs conflicted files on stderr/stdout
                let conflicts: Vec<String> = stderr
                    .lines()
                    .filter(|l| l.contains("CONFLICT") || l.ends_with('\t'))
                    .filter_map(|l| {
                        // Extract file path from "CONFLICT (content): Merge conflict in <path>"
                        if l.contains("Merge conflict in ") {
                            l.split("Merge conflict in ").nth(1).map(|s| s.trim().to_string())
                        } else if l.contains("CONFLICT (modify/delete)") {
                            l.split(": ").nth(1).map(|s| s.split(" deleted").next().unwrap_or(s).trim().to_string())
                        } else {
                            None
                        }
                    })
                    .collect();
                Ok(MergeDryRunResult {
                    has_conflicts: !conflicts.is_empty(),
                    conflict_files: conflicts,
                    method: "merge-tree".to_string(),
                })
            }
            Err(e) => Err(e),
        }
    }

    fn git_version_at_least(version_str: &str, major: u32, minor: u32) -> bool {
        // "git version 2.43.0.windows.1" → extract 2.43
        let parts: Vec<&str> = version_str.trim().split_whitespace().collect();
        if let Some(version) = parts.get(2) {
            let nums: Vec<u32> = version.split('.').filter_map(|s| s.parse().ok()).collect();
            if nums.len() >= 2 {
                return nums[0] > major || (nums[0] == major && nums[1] >= minor);
            }
        }
        false
    }

    /// Get list of conflicted files from git status.
    pub fn get_conflict_files(&self, path: &Path) -> Result<Vec<ConflictFile>, GitError> {
        // Check MERGE_HEAD exists
        let merge_head = path.join(".git/MERGE_HEAD");
        if !merge_head.exists() {
            return Err(GitError::OperationFailed("不在 merge 狀態中（MERGE_HEAD 不存在）".to_string()));
        }

        let output = Self::run_git(path, &["status", "--porcelain=v2"])?;
        let mut files = Vec::new();

        for line in output.lines() {
            if line.starts_with("u ") {
                // Unmerged entry: u <xy> <sub> <m1> <m2> <m3> <mW> <h1> <h2> <h3> <path>
                let parts: Vec<&str> = line.splitn(11, ' ').collect();
                if parts.len() >= 11 {
                    let xy = parts[1];
                    let file_path = parts[10].to_string();
                    let conflict_type = Self::classify_conflict(xy);
                    let is_binary = Self::is_binary_file(path, &file_path);
                    files.push(ConflictFile {
                        path: file_path,
                        conflict_type,
                        is_binary,
                    });
                }
            }
        }

        Ok(files)
    }

    fn classify_conflict(xy: &str) -> ConflictType {
        // xy codes: DD=both deleted, AU=added by us, UD=deleted by them,
        // UA=added by them, DU=deleted by us, AA=both added, UU=both modified
        match xy {
            "AA" => ConflictType::AddAdd,
            "DU" | "UD" => ConflictType::DeleteModify,
            _ => ConflictType::Content,
        }
    }

    fn is_binary_file(repo_path: &Path, file_path: &str) -> bool {
        let full_path = repo_path.join(file_path);
        if let Ok(bytes) = std::fs::read(&full_path) {
            // Check first 8KB for null bytes (common binary detection)
            let check_len = bytes.len().min(8192);
            bytes[..check_len].contains(&0)
        } else {
            false
        }
    }

    /// Read and parse conflict markers from a file.
    pub fn get_conflict_content(&self, path: &Path, file_path: &str) -> Result<ConflictContent, GitError> {
        // Validate path is within repo
        let full_path = path.join(file_path);
        let canonical_repo = path.canonicalize().map_err(|e| GitError::Io(e))?;
        let canonical_file = full_path.canonicalize().map_err(|e| GitError::Io(e))?;
        if !canonical_file.starts_with(&canonical_repo) {
            return Err(GitError::OperationFailed("路徑超出 repo 範圍".to_string()));
        }

        let content = std::fs::read_to_string(&full_path)
            .map_err(|e| GitError::OperationFailed(format!("無法讀取檔案 {file_path}: {e}")))?;

        // Check file size
        if content.len() > MAX_CONFLICT_FILE_BYTES {
            let hash = Self::compute_hash(&content);
            return Ok(ConflictContent {
                path: file_path.to_string(),
                segments: Vec::new(),
                hunk_count: 0,
                content_hash: hash,
                parse_error: Some("檔案過大（>1MB），建議在編輯器中開啟".to_string()),
            });
        }

        let hash = Self::compute_hash(&content);
        Self::parse_conflict_markers(file_path, &content, hash)
    }

    fn compute_hash(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Parse conflict markers from file content.
    ///
    /// Handles standard markers:
    ///   <<<<<<< HEAD
    ///   (ours content)
    ///   ||||||| base    (optional, diff3 style)
    ///   (base content)
    ///   =======
    ///   (theirs content)
    ///   >>>>>>> branch
    fn parse_conflict_markers(file_path: &str, content: &str, content_hash: String) -> Result<ConflictContent, GitError> {
        let lines: Vec<&str> = content.lines().collect();
        let mut segments: Vec<ConflictSegment> = Vec::new();
        let mut hunk_count: usize = 0;
        let mut context_buf: Vec<&str> = Vec::new();
        let mut i = 0;

        while i < lines.len() {
            if lines[i].starts_with("<<<<<<<") {
                // Flush context buffer
                if !context_buf.is_empty() {
                    segments.push(ConflictSegment::Context(context_buf.join("\n")));
                    context_buf.clear();
                }

                let start_line = i + 1;
                let mut ours_lines: Vec<&str> = Vec::new();
                let mut base_lines: Vec<&str> = Vec::new();
                let mut theirs_lines: Vec<&str> = Vec::new();
                let mut in_base = false;
                let mut in_theirs = false;
                let mut found_end = false;

                i += 1;
                while i < lines.len() {
                    if lines[i].starts_with("|||||||") {
                        in_base = true;
                    } else if lines[i].starts_with("=======") {
                        in_theirs = true;
                        in_base = false;
                    } else if lines[i].starts_with(">>>>>>>") {
                        found_end = true;
                        i += 1;
                        break;
                    } else if in_theirs {
                        theirs_lines.push(lines[i]);
                    } else if in_base {
                        base_lines.push(lines[i]);
                    } else {
                        ours_lines.push(lines[i]);
                    }
                    i += 1;
                }

                if !found_end {
                    // Malformed: no closing marker
                    return Ok(ConflictContent {
                        path: file_path.to_string(),
                        segments: Vec::new(),
                        hunk_count: 0,
                        content_hash,
                        parse_error: Some("Conflict markers 格式異常（缺少 >>>>>>>）".to_string()),
                    });
                }

                let base = if base_lines.is_empty() {
                    None
                } else {
                    Some(base_lines.join("\n"))
                };

                segments.push(ConflictSegment::Hunk(ConflictHunk {
                    index: hunk_count,
                    ours: ours_lines.join("\n"),
                    theirs: theirs_lines.join("\n"),
                    base,
                    start_line,
                }));
                hunk_count += 1;
            } else {
                context_buf.push(lines[i]);
                i += 1;
            }
        }

        // Flush remaining context
        if !context_buf.is_empty() {
            segments.push(ConflictSegment::Context(context_buf.join("\n")));
        }

        Ok(ConflictContent {
            path: file_path.to_string(),
            segments,
            hunk_count,
            content_hash,
            parse_error: None,
        })
    }

    /// Resolve a conflict file by writing new content (for content conflicts).
    pub fn resolve_conflict_content(
        &self,
        path: &Path,
        file_path: &str,
        resolved_content: &str,
        expected_hash: &str,
    ) -> Result<(), GitError> {
        let full_path = path.join(file_path);
        // Validate path
        let canonical_repo = path.canonicalize().map_err(|e| GitError::Io(e))?;
        let canonical_file = full_path.canonicalize().map_err(|e| GitError::Io(e))?;
        if !canonical_file.starts_with(&canonical_repo) {
            return Err(GitError::OperationFailed("路徑超出 repo 範圍".to_string()));
        }

        // Hash check: read current file and compare hash
        let current_content = std::fs::read_to_string(&full_path)
            .map_err(|e| GitError::OperationFailed(format!("無法讀取檔案: {e}")))?;
        let current_hash = Self::compute_hash(&current_content);
        if current_hash != expected_hash {
            return Err(GitError::OperationFailed(
                "檔案已被外部修改，請重新載入衝突內容".to_string(),
            ));
        }

        // Atomic write using tempfile
        use std::io::Write;
        let dir = full_path.parent().ok_or_else(|| {
            GitError::OperationFailed("無法取得檔案目錄".to_string())
        })?;
        let mut tmp = tempfile::NamedTempFile::new_in(dir)
            .map_err(|e| GitError::OperationFailed(format!("無法建立暫存檔: {e}")))?;
        tmp.write_all(resolved_content.as_bytes())
            .map_err(|e| GitError::OperationFailed(format!("寫入失敗: {e}")))?;
        tmp.persist(&full_path)
            .map_err(|e| GitError::OperationFailed(format!("原子寫入失敗: {e}")))?;

        Ok(())
    }

    /// Resolve a conflict file by choosing ours or theirs (for binary/delete-modify).
    pub fn resolve_conflict_choice(
        &self,
        path: &Path,
        file_path: &str,
        choice: &ResolveChoice,
    ) -> Result<(), GitError> {
        let flag = match choice {
            ResolveChoice::Ours => "--ours",
            ResolveChoice::Theirs => "--theirs",
        };
        Self::run_git(path, &["checkout", flag, "--", file_path])?;
        Ok(())
    }

    /// Complete the merge by committing.
    pub fn complete_merge(&self, path: &Path, message: &str) -> Result<MergeCompleteResult, GitError> {
        Self::check_index_lock(path)?;

        // Check no unresolved conflicts remain
        let merge_head = path.join(".git/MERGE_HEAD");
        if !merge_head.exists() {
            return Err(GitError::OperationFailed("不在 merge 狀態中".to_string()));
        }

        // Check for remaining conflicts
        let status_output = Self::run_git(path, &["status", "--porcelain=v2"])?;
        let unresolved: Vec<&str> = status_output
            .lines()
            .filter(|l| l.starts_with("u "))
            .collect();
        if !unresolved.is_empty() {
            return Err(GitError::OperationFailed(
                format!("還有 {} 個未解決的衝突", unresolved.len()),
            ));
        }

        let msg = if message.trim().is_empty() {
            // Use default merge message
            let merge_msg_path = path.join(".git/MERGE_MSG");
            std::fs::read_to_string(&merge_msg_path).unwrap_or_else(|_| "Merge commit".to_string())
        } else {
            message.to_string()
        };

        let output = Self::run_git(path, &["commit", "-m", &msg])?;
        let hash = output
            .lines()
            .next()
            .and_then(|line| line.split_whitespace().nth(1))
            .unwrap_or("unknown")
            .trim_matches(|c| c == '[' || c == ']')
            .to_string();

        Ok(MergeCompleteResult { commit_hash: hash })
    }
}

impl GitOperations for LocalGit {
    fn status(&self, path: &Path) -> Result<Vec<FileStatus>, GitError> {
        let repo = Self::open_repo(path)?;
        let mut opts = StatusOptions::new();
        opts.include_untracked(true)
            .recurse_untracked_dirs(true)
            .include_ignored(false);

        let statuses = repo.statuses(Some(&mut opts))?;
        let mut result = Vec::new();

        for entry in statuses.iter() {
            let file_path = entry.path().unwrap_or("");
            let git_status = entry.status();

            // 同一個檔案可能同時有 staged 和 unstaged 的變更
            if git_status.intersects(
                git2::Status::INDEX_NEW
                    | git2::Status::INDEX_MODIFIED
                    | git2::Status::INDEX_DELETED
                    | git2::Status::INDEX_RENAMED,
            ) {
                let (kind, _) = Self::map_status_kind(
                    git_status
                        & (git2::Status::INDEX_NEW
                            | git2::Status::INDEX_MODIFIED
                            | git2::Status::INDEX_DELETED
                            | git2::Status::INDEX_RENAMED),
                );
                result.push(FileStatus {
                    path: PathBuf::from(file_path),
                    kind,
                    staging: StagingState::Staged,
                });
            }

            if git_status.intersects(
                git2::Status::WT_NEW
                    | git2::Status::WT_MODIFIED
                    | git2::Status::WT_DELETED
                    | git2::Status::WT_RENAMED
                    | git2::Status::CONFLICTED,
            ) {
                let (kind, _) = Self::map_status_kind(
                    git_status
                        & (git2::Status::WT_NEW
                            | git2::Status::WT_MODIFIED
                            | git2::Status::WT_DELETED
                            | git2::Status::WT_RENAMED
                            | git2::Status::CONFLICTED),
                );
                result.push(FileStatus {
                    path: PathBuf::from(file_path),
                    kind,
                    staging: StagingState::Unstaged,
                });
            }
        }

        Ok(result)
    }

    fn log(&self, path: &Path, limit: usize) -> Result<Vec<Commit>, GitError> {
        let repo = Self::open_repo(path)?;

        // Build commit_id → refs map from all references
        let mut refs_map: std::collections::HashMap<String, Vec<CommitRef>> =
            std::collections::HashMap::new();

        if let Ok(refs) = repo.references() {
            for ref_result in refs {
                if let Ok(reference) = ref_result {
                    let name = match reference.shorthand() {
                        Some(n) => n.to_string(),
                        None => continue,
                    };
                    let target_oid = match reference.resolve() {
                        Ok(resolved) => match resolved.target() {
                            Some(oid) => oid,
                            None => continue,
                        },
                        Err(_) => continue,
                    };

                    let kind = if reference.is_tag()
                        || name.starts_with("tags/")
                        || reference
                            .name()
                            .map_or(false, |n| n.starts_with("refs/tags/"))
                    {
                        RefKind::Tag
                    } else if reference.is_remote()
                        || reference
                            .name()
                            .map_or(false, |n| n.starts_with("refs/remotes/"))
                    {
                        RefKind::Remote
                    } else {
                        RefKind::Local
                    };

                    refs_map
                        .entry(target_oid.to_string())
                        .or_default()
                        .push(CommitRef {
                            name,
                            kind,
                        });
                }
            }
        }

        let mut revwalk = repo.revwalk()?;
        revwalk.push_head()?;
        revwalk.set_sorting(git2::Sort::TIME)?;

        let mut commits = Vec::new();
        for oid in revwalk.take(limit) {
            let oid = oid?;
            let commit = repo.find_commit(oid)?;
            let commit_refs = refs_map.remove(&oid.to_string()).unwrap_or_default();
            commits.push(Commit {
                id: oid.to_string(),
                message: commit.message().unwrap_or("").to_string(),
                author: commit.author().name().unwrap_or("").to_string(),
                email: commit.author().email().unwrap_or("").to_string(),
                timestamp: commit.time().seconds(),
                parents: commit.parent_ids().map(|id| id.to_string()).collect(),
                refs: commit_refs,
            });
        }

        Ok(commits)
    }

    fn diff(&self, path: &Path, file: &Path) -> Result<DiffResult, GitError> {
        let repo = Self::open_repo(path)?;
        let mut diff_opts = DiffOptions::new();
        diff_opts.pathspec(file);

        let diff = repo.diff_index_to_workdir(None, Some(&mut diff_opts))?;
        let stats = diff.stats()?;

        let mut hunks: Vec<DiffHunk> = Vec::new();
        let mut is_binary = false;
        let mut total_bytes: usize = 0;
        let mut truncated = false;

        // 用 print 代替 foreach 避免多重 mutable borrow
        diff.print(git2::DiffFormat::Patch, |delta, hunk, line| {
            is_binary = delta.flags().is_binary();

            if let Some(hunk) = hunk {
                let header = String::from_utf8_lossy(hunk.header()).to_string();
                // 新 hunk 開始時檢查是否需要建立新的 DiffHunk
                if hunks.is_empty()
                    || hunks.last().map_or(true, |h| h.header != header)
                {
                    hunks.push(DiffHunk {
                        header,
                        lines: Vec::new(),
                    });
                }
            }

            let content = String::from_utf8_lossy(line.content()).to_string();
            total_bytes += content.len();

            if total_bytes > MAX_DIFF_BYTES {
                truncated = true;
                return false;
            }

            let (kind, old_lineno, new_lineno) = match line.origin() {
                '+' => (DiffLineKind::Addition, None, line.new_lineno()),
                '-' => (DiffLineKind::Deletion, line.old_lineno(), None),
                ' ' => (DiffLineKind::Context, line.old_lineno(), line.new_lineno()),
                'H' => (DiffLineKind::Header, None, None),
                _ => return true,
            };

            if let Some(last_hunk) = hunks.last_mut() {
                last_hunk.lines.push(DiffLine {
                    kind,
                    content,
                    old_lineno,
                    new_lineno,
                });
            }
            true
        })?;

        Ok(DiffResult {
            file_path: file.to_path_buf(),
            hunks,
            stats: DiffStats {
                additions: stats.insertions(),
                deletions: stats.deletions(),
            },
            is_binary,
            is_truncated: truncated,
        })
    }

    fn stage(&self, path: &Path, files: &[PathBuf]) -> Result<(), GitError> {
        Self::check_index_lock(path)?;
        let mut args = vec!["add", "--"];
        let file_strs: Vec<String> = files.iter().map(|f| f.display().to_string()).collect();
        for f in &file_strs {
            args.push(f);
        }
        Self::run_git(path, &args)?;
        Ok(())
    }

    fn unstage(&self, path: &Path, files: &[PathBuf]) -> Result<(), GitError> {
        Self::check_index_lock(path)?;
        let mut args = vec!["restore", "--staged", "--"];
        let file_strs: Vec<String> = files.iter().map(|f| f.display().to_string()).collect();
        for f in &file_strs {
            args.push(f);
        }
        Self::run_git(path, &args)?;
        Ok(())
    }

    fn commit(&self, path: &Path, message: &str) -> Result<String, GitError> {
        Self::check_index_lock(path)?;
        if message.trim().is_empty() {
            return Err(GitError::OperationFailed(
                "Commit message 不能為空".to_string(),
            ));
        }
        let output = Self::run_git(path, &["commit", "-m", message])?;
        // 從輸出中提取 commit hash
        let hash = output
            .lines()
            .next()
            .and_then(|line| line.split_whitespace().nth(1))
            .unwrap_or("unknown")
            .trim_matches(|c| c == '[' || c == ']')
            .to_string();
        Ok(hash)
    }

    fn push(&self, path: &Path, remote: &str, branch: &str) -> Result<PushResult, GitError> {
        match Self::run_git(path, &["push", remote, branch]) {
            Ok(output) => Ok(PushResult {
                remote: remote.to_string(),
                branch: branch.to_string(),
                success: true,
                message: output,
            }),
            Err(e) => Ok(PushResult {
                remote: remote.to_string(),
                branch: branch.to_string(),
                success: false,
                message: e.to_string(),
            }),
        }
    }

    fn pull(&self, path: &Path, remote: &str, branch: &str) -> Result<PullResult, GitError> {
        match Self::run_git(path, &["pull", remote, branch]) {
            Ok(output) => {
                let conflicts: Vec<String> = if output.contains("CONFLICT") {
                    output
                        .lines()
                        .filter(|l| l.contains("CONFLICT"))
                        .map(|l| l.to_string())
                        .collect()
                } else {
                    Vec::new()
                };
                Ok(PullResult {
                    remote: remote.to_string(),
                    branch: branch.to_string(),
                    success: conflicts.is_empty(),
                    message: output,
                    conflicts,
                })
            }
            Err(e) => Ok(PullResult {
                remote: remote.to_string(),
                branch: branch.to_string(),
                success: false,
                message: e.to_string(),
                conflicts: Vec::new(),
            }),
        }
    }

    fn branches(&self, path: &Path) -> Result<Vec<Branch>, GitError> {
        let repo = Self::open_repo(path)?;
        let mut branches = Vec::new();

        for branch_result in repo.branches(Some(git2::BranchType::Local))? {
            let (branch, _) = branch_result?;
            let name = branch.name()?.unwrap_or("").to_string();
            let is_current = branch.is_head();

            let upstream_name = branch
                .upstream()
                .ok()
                .and_then(|u| u.name().ok().flatten().map(|s| s.to_string()));

            let local_commit = branch.get().peel_to_commit().ok();
            let commit_id = local_commit.as_ref().map(|c| c.id().to_string());
            let last_commit_timestamp = local_commit.as_ref().map(|c| c.time().seconds());

            // Compute ahead/behind using git2 if upstream exists
            let (ahead, behind) = if let Some(ref upstream_ref) = upstream_name {
                if let Ok(upstream_branch) = repo.find_branch(upstream_ref, git2::BranchType::Remote) {
                    if let (Some(local_oid), Ok(upstream_commit)) = (
                        local_commit.as_ref().map(|c| c.id()),
                        upstream_branch.get().peel_to_commit(),
                    ) {
                        repo.graph_ahead_behind(local_oid, upstream_commit.id())
                            .map(|(a, b)| (Some(a as u32), Some(b as u32)))
                            .unwrap_or((None, None))
                    } else {
                        (None, None)
                    }
                } else {
                    (None, None)
                }
            } else {
                (None, None)
            };

            branches.push(Branch {
                name,
                is_current,
                is_remote: false,
                upstream: upstream_name,
                commit_id,
                ahead,
                behind,
                last_commit_timestamp,
            });
        }

        for branch_result in repo.branches(Some(git2::BranchType::Remote))? {
            let (branch, _) = branch_result?;
            let name = branch.name()?.unwrap_or("").to_string();
            let commit = branch.get().peel_to_commit().ok();
            let commit_id = commit.as_ref().map(|c| c.id().to_string());
            let last_commit_timestamp = commit.as_ref().map(|c| c.time().seconds());

            branches.push(Branch {
                name,
                is_current: false,
                is_remote: true,
                upstream: None,
                commit_id,
                ahead: None,
                behind: None,
                last_commit_timestamp,
            });
        }

        Ok(branches)
    }

    fn switch_branch(&self, path: &Path, name: &str) -> Result<(), GitError> {
        Self::check_index_lock(path)?;
        Self::run_git(path, &["checkout", name])?;
        Ok(())
    }

    fn create_branch(&self, path: &Path, name: &str) -> Result<(), GitError> {
        Self::check_index_lock(path)?;
        Self::run_git(path, &["checkout", "-b", name])?;
        Ok(())
    }

    fn delete_branch(&self, path: &Path, name: &str) -> Result<(), GitError> {
        Self::check_index_lock(path)?;
        Self::run_git(path, &["branch", "-d", name])?;
        Ok(())
    }
}
