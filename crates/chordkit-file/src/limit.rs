//! How much of a file may be read.

/// A size limit in bytes. A file of exactly this size is read; one byte more is refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ByteLimit(u64);

impl ByteLimit {
    /// A limit of `bytes`.
    pub const fn new(bytes: u64) -> ByteLimit {
        ByteLimit(bytes)
    }

    /// The limit in bytes.
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// The limit for a desktop's shortcut settings file: 4 MiB, far above any real one (kdeglobals
/// and a dconf dump are tens of KiB) and small enough to hold in memory at once.
pub const SETTINGS_FILE_LIMIT: ByteLimit = ByteLimit::new(4 * 1024 * 1024);
