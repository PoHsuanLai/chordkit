//! A private folder per test, removed on drop.

use std::path::{Path, PathBuf};

pub struct Scratch(PathBuf);

impl Scratch {
    /// A fresh empty folder named for the test, under the system temporary folder.
    pub fn new(test: &str) -> Scratch {
        let path =
            std::env::temp_dir().join(format!("chordkit-file-{}-{test}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        Scratch(path)
    }

    pub fn path(&self) -> &Path {
        &self.0
    }

    /// Writes `bytes` to `name` in the folder and returns its path.
    pub fn file(&self, name: &str, bytes: &[u8]) -> PathBuf {
        let path = self.0.join(name);
        std::fs::write(&path, bytes).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        path
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
