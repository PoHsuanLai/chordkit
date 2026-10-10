//! Where a keymap comes from: the system's own shortcut settings.
//!
//! A source turns what a platform stores into a `Keymap`. The trait is runtime-agnostic: loading
//! is a plain call, and change notification is a callback. Sources live in their own crates
//! (`chordkit-kde`, `chordkit-gnome`) or in the program that owns the settings (keycap's daemon
//! implements this trait for our desktop on top of chordkit).

use thiserror::Error;

use crate::chord::Chord;
use crate::keymap::Keymap;
use crate::platform::Platform;
use crate::standard_action::StandardAction;

/// Why a source could not give a keymap.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SourceError {
    /// A settings file or service could not be read.
    #[error("couldn't read the shortcut settings at {place}: {reason}")]
    Read {
        /// The file or service.
        place: String,
        /// The system's reason.
        reason: String,
    },
    /// The source cannot watch for changes or serve this platform.
    #[error("{0}")]
    Unsupported(String),
}

/// The system's chords for one standard action. No chords means the system unbinds it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemBinding {
    /// The action.
    pub action: StandardAction,
    /// Its chords, best first.
    pub chords: Vec<Chord>,
}

/// What reading a system's settings produced: the bindings found and what could not be read.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SystemImport {
    /// One binding per action, in the order first seen.
    pub bindings: Vec<SystemBinding>,
    /// Entries that were recognised but could not be read, in plain words.
    pub problems: Vec<String>,
}

impl SystemImport {
    /// Adds `chords` to `action`'s binding (creating it), so two settings that mean one
    /// action both count.
    pub fn add(&mut self, action: StandardAction, chords: Vec<Chord>) {
        match self.bindings.iter_mut().find(|b| b.action == action) {
            Some(binding) => binding.chords.extend(chords),
            None => self.bindings.push(SystemBinding { action, chords }),
        }
    }
}

/// Called (from any thread) when the source's settings changed; reload to see them.
pub type ChangeCallback = Box<dyn Fn() + Send + Sync + 'static>;

/// Keeps a change subscription alive; dropping it ends the subscription.
pub struct Watch {
    _guard: Box<dyn Send>,
}

impl Watch {
    /// A watch that never fires, for sources whose settings do not change while running.
    pub fn inert() -> Watch {
        Watch {
            _guard: Box::new(()),
        }
    }

    /// A watch that holds `guard` until dropped (a file watcher, a bus subscription).
    pub fn holding(guard: impl Send + 'static) -> Watch {
        Watch {
            _guard: Box::new(guard),
        }
    }
}

impl std::fmt::Debug for Watch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Watch")
    }
}

/// Something that can give the keymap a platform really has.
pub trait KeymapSource {
    /// The platform's keymap: its conventions with this source's settings laid over them.
    fn load(&self, platform: Platform) -> Result<Keymap, SourceError>;

    /// Calls `on_change` whenever the settings change, until the returned `Watch` is dropped.
    /// The default never fires; a source with a notification channel overrides it.
    fn watch(&self, on_change: ChangeCallback) -> Result<Watch, SourceError> {
        drop(on_change);
        Ok(Watch::inert())
    }
}

/// The source with no settings of its own: the platform's conventions. It is what macOS and
/// Windows use until they have a source that reads their settings.
#[derive(Debug, Clone, Copy, Default)]
pub struct ConventionSource;

impl KeymapSource for ConventionSource {
    fn load(&self, platform: Platform) -> Result<Keymap, SourceError> {
        Ok(Keymap::conventional(platform))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::Action;
    use crate::platform::Context;

    #[test]
    fn convention_source_loads_conventions_and_never_fires() {
        let keymap = ConventionSource.load(Platform::Windows);
        let keymap = keymap.unwrap_or_else(|e| panic!("{e}"));
        let copy = Action::Standard(StandardAction::Copy);
        assert_eq!(keymap.chords_in(&copy, Context::Normal).len(), 1);
        assert!(ConventionSource.watch(Box::new(|| {})).is_ok());
    }

    #[test]
    fn import_merges_chords_of_one_action() {
        let mut import = SystemImport::default();
        let a: Chord = "Ctrl+A".parse().unwrap_or_else(|e| panic!("{e}"));
        let b: Chord = "Ctrl+B".parse().unwrap_or_else(|e| panic!("{e}"));
        import.add(StandardAction::Copy, vec![a]);
        import.add(StandardAction::Copy, vec![b]);
        assert_eq!(
            import.bindings,
            [SystemBinding {
                action: StandardAction::Copy,
                chords: vec![a, b]
            }]
        );
    }
}
