//! chordkit pieces for macOS that need no platform call: the value syntax of
//! `NSUserKeyEquivalents` (System Settings > Keyboard > App Shortcuts) and a table from menu
//! titles to standard actions.
//!
//! Reading the plist files themselves (`~/Library/Preferences/.GlobalPreferences.plist` and each
//! app's domain) is left to the program that owns file access; it hands the title and value
//! strings in here.

mod equivalent;
mod titles;

pub use equivalent::parse_key_equivalent;
pub use titles::{ENGLISH, MenuTitles, import_user_key_equivalents};
