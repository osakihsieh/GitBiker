use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum GitError {
    #[error("路徑不存在: {0}")]
    PathNotFound(String),

    #[error("不是 Git 倉庫: {0}")]
    NotARepo(String),

    #[error("Git 操作失敗: {0}")]
    OperationFailed(String),

    #[error("認證失敗: {0}")]
    AuthError(String),

    #[error("網路逾時: {0}")]
    NetworkTimeout(String),

    #[error("index.lock 已存在，另一個 Git 操作可能正在進行")]
    IndexLocked,

    #[error("git2 錯誤: {0}")]
    Git2(#[from] git2::Error),

    #[error("IO 錯誤: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Serialize)]
pub struct SerializableError {
    pub code: String,
    pub message: String,
}

impl From<&GitError> for SerializableError {
    fn from(err: &GitError) -> Self {
        let code = match err {
            GitError::PathNotFound(_) => "PATH_NOT_FOUND",
            GitError::NotARepo(_) => "NOT_A_REPO",
            GitError::OperationFailed(_) => "OPERATION_FAILED",
            GitError::AuthError(_) => "AUTH_ERROR",
            GitError::NetworkTimeout(_) => "NETWORK_TIMEOUT",
            GitError::IndexLocked => "INDEX_LOCKED",
            GitError::Git2(_) => "GIT2_ERROR",
            GitError::Io(_) => "IO_ERROR",
        };
        Self {
            code: code.to_string(),
            message: err.to_string(),
        }
    }
}

impl Serialize for GitError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        SerializableError::from(self).serialize(serializer)
    }
}
