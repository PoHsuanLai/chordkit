//! The layers a keymap is made of, lowest priority first.

use crate::action::Action;
use crate::chord::DefaultChord;

/// Where a binding comes from. A higher layer replaces a lower one for the same action, and wins
/// a chord both want.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum Layer {
    /// The platform's conventions.
    Convention,
    /// An app's declared defaults.
    App,
    /// The system's own settings.
    System,
    /// The person's changes.
    Person,
}

/// One action's chords in one layer. No chords means the layer unbinds the action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Entry {
    pub(crate) action: Action,
    pub(crate) layer: Layer,
    pub(crate) chords: Vec<DefaultChord>,
}

/// Puts `entry` in, replacing the entry for the same action in the same layer.
pub(crate) fn upsert(entries: &mut Vec<Entry>, entry: Entry) {
    match entries
        .iter_mut()
        .find(|e| e.action == entry.action && e.layer == entry.layer)
    {
        Some(existing) => *existing = entry,
        None => entries.push(entry),
    }
}
