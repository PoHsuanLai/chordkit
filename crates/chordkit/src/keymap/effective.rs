//! From layers to the chords that are live in a context.

use std::cmp::Reverse;
use std::collections::HashSet;

use super::layers::{Entry, Layer};
use crate::action::Action;
use crate::chord::Chord;
use crate::platform::{Context, Platform};

/// An action and one chord that triggers it.
pub(crate) type Pair = (Action, Chord);

/// A person's entry for an app action nobody registered: kept, but not live.
fn dormant(entries: &[Entry], entry: &Entry) -> bool {
    entry.layer == Layer::Person
        && matches!(entry.action, Action::App(_))
        && !entries
            .iter()
            .any(|e| e.layer == Layer::App && e.action == entry.action)
}

/// The entries that decide their action: the highest live layer for each.
fn winners(entries: &[Entry]) -> Vec<&Entry> {
    entries
        .iter()
        .filter(|entry| !dormant(entries, entry))
        .filter(|entry| {
            !entries.iter().any(|other| {
                other.action == entry.action
                    && other.layer > entry.layer
                    && !dormant(entries, other)
            })
        })
        .collect()
}

/// Whether `chord` may fire in `context`: a text field keeps typing keys, and a terminal also
/// keeps plain Ctrl for the program.
fn allowed(context: Context, chord: Chord) -> bool {
    match context {
        Context::Normal => true,
        Context::TextEntry => !chord.belongs_to_text_field(),
        Context::Terminal => !chord.belongs_to_text_field() && !chord.is_plain_ctrl(),
    }
}

/// Every live (action, chord) in `context`. Where two actions want one chord, the higher layer
/// has it, and within a layer the first.
pub(crate) fn effective(platform: Platform, entries: &[Entry], context: Context) -> Vec<Pair> {
    let mut wanted: Vec<(Layer, &Action, Chord)> = Vec::new();
    for entry in winners(entries) {
        for default in &entry.chords {
            if let Some(chord) = default
                .resolve(platform, context)
                .filter(|c| allowed(context, *c))
            {
                wanted.push((entry.layer, &entry.action, chord));
            }
        }
    }
    wanted.sort_by_key(|(layer, _, _)| Reverse(*layer));
    let mut taken: HashSet<Chord> = HashSet::new();
    wanted
        .into_iter()
        .filter(|(_, _, chord)| taken.insert(*chord))
        .map(|(_, action, chord)| (action.clone(), chord))
        .collect()
}
