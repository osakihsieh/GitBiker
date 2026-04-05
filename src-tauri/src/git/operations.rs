use std::path::{Path, PathBuf};

use crate::git::error::GitError;
use crate::git::types::*;

pub trait GitOperations: Send + Sync {
    fn status(&self, path: &Path) -> Result<Vec<FileStatus>, GitError>;
    fn log(&self, path: &Path, limit: usize, filter: Option<LogFilter>) -> Result<Vec<Commit>, GitError>;
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
    fn branch_compare(&self, path: &Path, base: &str, compare: &str) -> Result<BranchCompareResult, GitError>;
}
