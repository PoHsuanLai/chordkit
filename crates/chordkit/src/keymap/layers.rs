//! The layers a keymap is made of, lowest priority first.

use crate::action::Action;
use crate::chord::{Chord, DefaultChord};
use crate::platform::{Context, Platform};

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
    /// Standard actions an app has given up because it does not offer them. Above everything:
    /// the app does not have the feature, whoever bound it.
    Forgone,
}

/// One chord an action is bound to: a default and, optionally, a fallback used in a context
/// where the default has no live form.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Binding {
    first: DefaultChord,
    fallback: Option<DefaultChord>,
}

impl Binding {
    pub(crate) fn with_fallback(first: DefaultChord, fallback: DefaultChord) -> Binding {
        Binding {
            first,
            fallback: Some(fallback),
        }
    }

    /// The fallback default, if the row has one.
    pub(crate) fn fallback(self) -> Option<DefaultChord> {
        self.fallback
    }

    /// The default's own chord on `platform` in `context`, if it has a form that can fire.
    pub(crate) fn first_live(self, platform: Platform, context: Context) -> Option<Chord> {
        self.first
            .resolve(platform, context)
            .filter(|c| super::effective::allowed(context, *c))
    }

    /// The chord that is live on `platform` in `context`: the default's, else the fallback's. A
    /// form that cannot fire in `context` (a plain-Ctrl chord in a terminal, which the program
    /// keeps) counts as no form, so the fallback is tried.
    pub(crate) fn live(self, platform: Platform, context: Context) -> Option<Chord> {
        let fires = |chord: &Chord| super::effective::allowed(context, *chord);
        self.first
            .resolve(platform, context)
            .filter(fires)
            .or_else(|| self.fallback?.resolve(platform, context).filter(fires))
    }
}

impl From<DefaultChord> for Binding {
    fn from(first: DefaultChord) -> Binding {
        Binding {
            first,
            fallback: None,
        }
    }
}

/// One action's chords in one layer. No chords means the layer unbinds the action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Entry {
    pub(crate) action: Action,
    pub(crate) layer: Layer,
    pub(crate) chords: Vec<Binding>,
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
