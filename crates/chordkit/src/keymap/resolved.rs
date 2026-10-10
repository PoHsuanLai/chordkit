//! The live bindings of a keymap for each context, indexed for lookup.

use std::collections::HashMap;

use super::effective::{Pair, effective};
use super::layers::Entry;
use crate::action::Action;
use crate::chord::Chord;
use crate::platform::{Context, Platform};

/// One context's live bindings.
#[derive(Debug, Clone, Default)]
pub(crate) struct Bound {
    pairs: Vec<Pair>,
    by_chord: HashMap<Chord, usize>,
}

impl Bound {
    fn new(pairs: Vec<Pair>) -> Bound {
        let by_chord = pairs
            .iter()
            .enumerate()
            .map(|(index, (_, chord))| (*chord, index))
            .collect();
        Bound { pairs, by_chord }
    }

    pub(crate) fn action_for(&self, chord: Chord) -> Option<&Action> {
        self.by_chord.get(&chord).map(|index| &self.pairs[*index].0)
    }

    pub(crate) fn pairs(&self) -> &[Pair] {
        &self.pairs
    }
}

/// The three contexts' bindings.
#[derive(Debug, Clone, Default)]
pub(crate) struct Resolved {
    normal: Bound,
    text_entry: Bound,
    terminal: Bound,
}

impl Resolved {
    pub(crate) fn build(platform: Platform, entries: &[Entry]) -> Resolved {
        let of = |context| Bound::new(effective(platform, entries, context));
        Resolved {
            normal: of(Context::Normal),
            text_entry: of(Context::TextEntry),
            terminal: of(Context::Terminal),
        }
    }

    pub(crate) fn in_context(&self, context: Context) -> &Bound {
        match context {
            Context::Normal => &self.normal,
            Context::TextEntry => &self.text_entry,
            Context::Terminal => &self.terminal,
        }
    }
}
