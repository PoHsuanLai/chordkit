//! chordkit source for GNOME, from a dconf keyfile dump (`dconf dump /` output).
//!
//! GNOME has no user-editable keys for the editing shortcuts (Copy, Paste and so on are fixed in
//! GTK), so those come from chordkit's conventions. What a person can change are the window
//! manager, shell and media keys, which dconf stores under `org/gnome/...`; this crate maps
//! those that match standard actions. Reading the dump needs no GLib: the dump is text.

mod accelerator;
mod keys;
mod parse;
mod source;

pub use accelerator::parse_accelerator;
pub use parse::parse_dconf_dump;
pub use source::GnomeSource;
