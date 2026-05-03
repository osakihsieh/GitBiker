use std::path::{Path, PathBuf};

use crate::git::error::GitError;
use crate::git::types::*;

pub trait GitOperations: Send + Sync {
    fn status(&self, path: &Path) -> Result<Vec<FileStatus>, GitError>;
    fn log(
        &self,
        path: &Path,
        limit: usize,
        filter: Option<LogFilter>,
    ) -> Result<Vec<Commit>, GitError>;
    fn diff(&self, path: &Path, file: &Path) -> Result<DiffResult, GitError>;
    fn stage(&self, path: &Path, files: &[PathBuf]) -> Result<(), GitError>;
    fn unstage(&self, path: &Path, files: &[PathBuf]) -> Result<(), GitError>;
    fn commit(&self, path: &Path, message: &str) -> Result<String, GitError>;
    fn push(&self, path: &Path, remote: &str, branch: &str) -> Result<PushResult, GitError>;
    fn pull(&self, path: &Path, remote: &str, branch: &str) -> Result<PullResult, GitError>;
    fn branches(&self, path: &Path) -> Result<Vec<Branch>, GitError>;
    fn switch_branch(&self, path: &Path, name: &str) -> Result<(), GitError>;
    fn create_branch(&self, path: &Path, name: &str) -> Result<(), GitError>;
    fn delete_branch(&self, path: &Path, name: &str) -> Result<(), GitError>;
    fn branch_compare(
        &self,
        path: &Path,
        base: &str,
        compare: &str,
    ) -> Result<BranchCompareResult, GitError>;
    fn rebase(&self, path: &Path, branch: &str, onto: &str) -> Result<RebaseResult, GitError>;
    fn rebase_interactive(
        &self,
        path: &Path,
        onto: &str,
        commits: Vec<RebaseCommit>,
    ) -> Result<RebaseResult, GitError>;
    fn lfs_status(&self, path: &Path) -> Result<GitLfsStatus, GitError>;
    fn lfs_track(&self, path: &Path, pattern: &str) -> Result<(), GitError>;
    fn lfs_untrack(&self, path: &Path, pattern: &str) -> Result<(), GitError>;
    fn cherry_pick(&self, path: &Path, commit_id: &str) -> Result<CherryPickResult, GitError>;
    fn get_submodules(&self, path: &Path) -> Result<Vec<SubmoduleInfo>, GitError>;
    fn update_submodule(
        &self,
        path: &Path,
        name: &str,
        init: bool,
        recursive: bool,
    ) -> Result<(), GitError>;
    fn add_submodule(&self, path: &Path, url: &str, submodule_path: &Path) -> Result<(), GitError>;
}
