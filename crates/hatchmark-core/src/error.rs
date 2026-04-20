use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialisation error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("no such {kind} with id {id}")]
    NotFound { kind: &'static str, id: i64 },
    #[error("invalid state: {0}")]
    Invalid(String),
}

pub type Result<T> = std::result::Result<T, CoreError>;
