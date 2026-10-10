//! chordkit: a portable keymap core.
//!
//! Apps declare their own actions with a portable default chord and handle actions; they never
//! read modifiers or decide Cmd versus Ctrl. A [`Keymap`] holds, for one [`Platform`], the
//! platform's conventions, the system's own shortcut settings, the person's changes and the apps'
//! registered actions, and turns a toolkit's [`KeyInput`] into an [`Action`].
//!
//! The crate is pure: no I/O, no async. Sources that read a system's settings live in other
//! crates and implement [`KeymapSource`].

mod action;
mod chord;
mod chord_text;
mod convention;
mod detect;
mod key;
mod key_input;
pub mod keyfile;
mod keymap;
mod modifier;
mod overrides;
mod platform;
mod reserved;
#[cfg(feature = "serde")]
mod serde_support;
mod source;
mod standard_action;

pub use action::{Action, ActionIdError, AppAction, AppId};
pub use chord::{Chord, DefaultChord, PrimaryUse};
pub use chord_text::ChordError;
pub use convention::{Convention, ConventionTable};
pub use detect::{HostOs, OVERRIDE_VAR, detect, detect_process};
pub use key::{Key, NamedKey};
pub use key_input::{AltGr, KeyInput};
pub use keymap::{Conflict, Keymap};
pub use modifier::{Modifier, Modifiers};
pub use overrides::{OverrideEntry, OverrideProblem, Overrides};
pub use platform::{Context, Desktop, Platform, PlatformError};
pub use reserved::reserved_reason;
pub use source::{
    ChangeCallback, ConventionSource, KeymapSource, SourceError, SystemBinding, SystemImport, Watch,
};
pub use standard_action::{SpaceNumber, StandardAction};
