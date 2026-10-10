//! chordkit source for KDE Plasma: the `[Shortcuts]` group of `kdeglobals`, which holds the
//! user's KStandardShortcut choices (Copy, Undo, Find, and so on).
//!
//! The parser is pure over text; `KdeSource` adds the one file read, with the path injected.

mod names;
mod parse;
mod source;

pub use parse::parse_kdeglobals;
pub use source::KdeSource;
