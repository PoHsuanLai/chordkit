//! `GnomeSource`: a dconf dump from an injected path, over GNOME's conventions.

use std::path::PathBuf;

use chordkit::{Keymap, KeymapSource, Platform, SourceError};
use chordkit_file::{ReadError, SETTINGS_FILE_LIMIT, read_text};

use crate::parse::parse_dconf_dump;

/// Reads the person's GNOME shortcut choices from a dconf keyfile dump (the output of
/// `dconf dump /`, saved by the caller). A missing file means nothing was changed, so the keymap
/// is GNOME's conventions as they are.
#[derive(Debug, Clone)]
pub struct GnomeSource {
    path: PathBuf,
}

impl GnomeSource {
    /// A source reading the dump at `path`.
    pub fn new(path: impl Into<PathBuf>) -> GnomeSource {
        GnomeSource { path: path.into() }
    }
}

impl KeymapSource for GnomeSource {
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
        Ok(Keymap::conventional(platform).with_system(parse_dconf_dump(&text).bindings))
    }
}
