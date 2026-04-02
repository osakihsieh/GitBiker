use std::path::{Path, PathBuf};
use std::process::Command;

use git2::{DiffOptions, Repository, StatusOptions};

use crate::git::error::GitError;
use crate::git::operations::GitOperations;
use crate::git::types::*;

const MAX_DIFF_BYTES: usize = 10 * 1024 * 1024; // 10MB

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
