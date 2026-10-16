//! The bounded read itself.

use std::fs::{File, OpenOptions};
use std::io::{ErrorKind, Read};
use std::path::Path;

use crate::error::ReadError;
use crate::limit::ByteLimit;

/// The text of the regular file at `path`, at most `limit` bytes of UTF-8.
pub fn read_text(path: &Path, limit: ByteLimit) -> Result<String, ReadError> {
    let bytes = read_bytes(path, limit)?;
    String::from_utf8(bytes).map_err(|_| ReadError::NotText)
}

/// The bytes of the regular file at `path`, at most `limit` of them.
pub fn read_bytes(path: &Path, limit: ByteLimit) -> Result<Vec<u8>, ReadError> {
    let file = open(path)?;
    let meta = file.metadata()?;
    if !meta.is_file() {
        return Err(ReadError::NotRegular);
    }
    if meta.len() > limit.get() {
        return Err(ReadError::TooLarge(limit));
    }
    // The file may have grown since the check: read one byte past the limit to notice.
    let mut bytes = Vec::new();
    file.take(limit.get().saturating_add(1))
        .read_to_end(&mut bytes)?;
    match u64::try_from(bytes.len()) {
        Ok(read) if read <= limit.get() => Ok(bytes),
        _ => Err(ReadError::TooLarge(limit)),
    }
}

/// Opens for reading without ever blocking: a pipe with no writer opens at once and is then
/// refused by the file-type check. Links are followed, so the check sees the target.
fn open(path: &Path) -> Result<File, ReadError> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NONBLOCK);
    }
    options.open(path).map_err(|error| match error.kind() {
        ErrorKind::NotFound => ReadError::Missing,
        _ => ReadError::Io(error),
    })
}
