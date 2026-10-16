//! One bounded reader for settings files that belong to someone else (a desktop's shortcut
//! files, a keymap in the person's home).
//!
//! A source that reads such a file must never hang, run out of memory or panic on it, whatever
//! has been put at that path. So the file is opened without blocking (a FIFO with no writer
//! would otherwise stop `open` for good), the opened file itself is checked to be a regular
//! file (symlinks are followed first, so the target is what is checked), its size is held to a
//! named limit, and its text must be UTF-8. Any failure is a typed [`ReadError`]; the caller
//! skips that source and logs the error.

mod error;
mod limit;
mod read;

pub use error::ReadError;
pub use limit::{ByteLimit, SETTINGS_FILE_LIMIT};
pub use read::{read_bytes, read_text};
