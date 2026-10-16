//! `KdeSource`: kdeglobals from an injected path, over KDE's conventions.

use std::path::PathBuf;

use chordkit::{Keymap, KeymapSource, Platform, SourceError};
use chordkit_file::{ReadError, SETTINGS_FILE_LIMIT, read_text};

use crate::parse::parse_kdeglobals;

/// Reads the user's KDE shortcut choices from one kdeglobals file. A missing file means the
/// person changed nothing, so the keymap is KDE's conventions as they are.
#[derive(Debug, Clone)]
pub struct KdeSource {
    path: PathBuf,
}

impl KdeSource {
    /// A source reading `path` (usually `<config dir>/kdeglobals`).
    pub fn new(path: impl Into<PathBuf>) -> KdeSource {
        KdeSource { path: path.into() }
    }
}

impl KeymapSource for KdeSource {
    fn load(&self, platform: Platform) -> Result<Keymap, SourceError> {
        let text = match read_text(&self.path, SETTINGS_FILE_LIMIT) {
            Ok(text) => text,
            Err(ReadError::Missing) => String::new(),
            Err(error) => {
                return Err(SourceError::Read {
                    place: self.path.display().to_string(),
                    reason: error.to_string(),
                });
            }
        };
        Ok(Keymap::conventional(platform).with_system(parse_kdeglobals(&text).bindings))
    }
}
