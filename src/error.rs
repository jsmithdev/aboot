/// Errors surfaced by aboot commands.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Msg(String),
    #[error("git failed: {0}")]
    Git(String),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

impl Error {
    pub fn msg<S: Into<String>>(s: S) -> Self {
        Error::Msg(s.into())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
