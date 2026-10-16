//! Why a file could not be read.

use thiserror::Error;

use crate::limit::ByteLimit;

/// Why a bounded read gave nothing. The text is plain words: it goes into a log line, and
/// the caller adds which file it was.
#[derive(Debug, Error)]
pub enum ReadError {
    /// There is no file at the path (or a link there points nowhere).
    #[error("there is no such file")]
    Missing,
    /// The path is not a plain file: it is a folder, a pipe, a device or a socket.
    #[error("it is not a plain file")]
    NotRegular,
    /// The file is bigger than the limit.
    #[error("it is larger than {} bytes", .0.get())]
    TooLarge(ByteLimit),
    /// The file is not text.
    #[error("it is not valid text")]
    NotText,
    /// The system refused to open or read it.
    #[error("{0}")]
    Io(#[from] std::io::Error),
}

impl ReadError {
    /// Whether the file was simply absent, which callers usually treat as "nothing set".
    pub fn is_missing(&self) -> bool {
        matches!(self, ReadError::Missing)
    }
}
