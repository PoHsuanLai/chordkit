//! A keymap: the live action <-> chord bindings for one platform.
//!
//! Layers, lowest first: the platform's conventions, apps' registered defaults, the system's own
//! settings, the person's changes. See `layers` for how they combine.

mod conflict;
mod effective;
mod layers;
mod register;
mod registration;
mod resolved;

use crate::action::{Action, AppAction};
use crate::chord::{Chord, DefaultChord};
use crate::convention::{Convention, ConventionTable};
use crate::key_input::KeyInput;
use crate::overrides::{OverrideProblem, Overrides};
use crate::platform::{Context, Platform};
use crate::reserved::reserved_reason;
use crate::source::SystemBinding;
use crate::standard_action::StandardAction;

pub use conflict::Conflict;
use layers::{Binding, Entry, Layer, upsert};
pub use registration::Registration;
use resolved::Resolved;

/// The bindings of one platform.
#[derive(Debug, Clone)]
pub struct Keymap {
    platform: Platform,
    entries: Vec<Entry>,
    resolved: Resolved,
}

impl Keymap {
    /// The platform's conventions and nothing else.
    pub fn conventional(platform: Platform) -> Keymap {
        let entries = ConventionTable::for_platform(platform)
            .iter()
            .filter_map(|(action, convention)| match convention {
                Convention::Bound(chord) => Some(Entry {
                    action: Action::Standard(action),
                    layer: Layer::Convention,
                    chords: vec![chord.into()],
                }),
                Convention::Unbound(_) => None,
            })
            .collect();
        Keymap {
            platform,
            entries,
            resolved: Resolved::default(),
        }
        .rebuilt()
    }

    /// This keymap with the system's own settings laid over the conventions: each action the
    /// system names takes the system's chords (none, when the system unbinds it).
    pub fn with_system(mut self, bindings: Vec<SystemBinding>) -> Keymap {
        for binding in bindings {
            upsert(
                &mut self.entries,
                Entry {
                    action: Action::Standard(binding.action),
                    layer: Layer::System,
                    chords: binding
                        .chords
                        .into_iter()
                        .map(|chord| Binding::from(DefaultChord::from(chord)))
                        .collect(),
                },
            );
        }
        self.rebuilt()
    }

    /// This keymap with the person's changes laid on top. Chords the platform reserves are
    /// refused and reported; the rest take effect. A change to an app action whose registration
    /// was refused binds too: the person's choice wins, over standard actions as well. Changes to
    /// app actions nobody has registered wait (see `dormant_overrides`).
    pub fn with_overrides(mut self, overrides: &Overrides) -> (Keymap, Vec<OverrideProblem>) {
        let mut problems = Vec::new();
        for change in overrides.entries() {
            let (allowed, refused): (Vec<DefaultChord>, Vec<DefaultChord>) = change
                .chords
                .iter()
                .copied()
                .partition(|chord| self.is_free(*chord));
            problems.extend(
                refused
                    .iter()
                    .map(|chord| self.refusal(change.line, *chord)),
            );
            if allowed.is_empty() && !refused.is_empty() {
                continue;
            }
            upsert(
                &mut self.entries,
                Entry {
                    action: change.action.clone(),
                    layer: Layer::Person,
                    chords: allowed.into_iter().map(Binding::from).collect(),
                },
            );
        }
        (self.rebuilt(), problems)
    }

    /// The platform this keymap is for.
    pub fn platform(&self) -> Platform {
        self.platform
    }

    /// The action `input` triggers in `context`, if any. The input is normalised for this
    /// platform and context first (so on our desktop a terminal's Ctrl+Shift+D is Super+D), and
    /// a press with AltGr held types text and triggers nothing.
    pub fn resolve(&self, input: &KeyInput, context: Context) -> Option<Action> {
        if input.types_text() {
            return None;
        }
        let chord = input.normalise_in(self.platform, context);
        let resolved = self.resolved.in_context(context);
        resolved
            .action_for(chord)
            .or_else(|| resolved.action_for(chord.plus_for_equals()?))
            .cloned()
    }

    /// Whether `input` is in the Primary form for this platform in `context`, bound or not: it
    /// holds the Primary modifiers (Ctrl+Shift for a terminal off the Mac style; Super or, on
    /// our desktop in a terminal, Ctrl+Shift too) and its key is a letter, digit, punctuation
    /// or symbol. A terminal drops such a chord when `resolve` finds nothing, rather than send
    /// a control byte to the program. A press with AltGr held is text, so never in this form.
    pub fn is_primary_form(&self, input: &KeyInput, context: Context) -> bool {
        let chord = input.normalise_in(self.platform, context);
        !input.types_text()
            && chord.key().is_printable()
            && chord
                .modifiers()
                .contains_all(self.platform.primary(context))
    }

    /// The chords that trigger `action` in ordinary window content, best first.
    pub fn chords_of(&self, action: &Action) -> Vec<Chord> {
        self.chords_in(action, Context::Normal)
    }

    /// The chords that trigger `action` in `context`.
    pub fn chords_in(&self, action: &Action, context: Context) -> Vec<Chord> {
        self.resolved
            .in_context(context)
            .pairs()
            .iter()
            .filter(|(bound, _)| bound == action)
            .map(|(_, chord)| *chord)
            .collect()
    }

    /// Every live (action, chord) in `context`, for menus and settings screens.
    pub fn bindings(&self, context: Context) -> &[(Action, Chord)] {
        self.resolved.in_context(context).pairs()
    }

    /// App actions the person has changed that no app has registered (yet). An action an app
    /// registered but had refused is not dormant: the person's change binds.
    pub fn dormant_overrides(&self) -> Vec<AppAction> {
        let registered = |action: &Action| {
            self.entries
                .iter()
                .any(|e| e.layer == Layer::App && e.action == *action)
        };
        self.entries
            .iter()
            .filter(|e| e.layer == Layer::Person && !registered(&e.action))
            .filter_map(|e| match &e.action {
                Action::App(action) => Some(action.clone()),
                Action::Standard(_) => None,
            })
            .collect()
    }

    /// The standard action's chords as the conventions and system give them, ignoring the
    /// person's changes: what "reset to default" would restore.
    pub fn default_chords_of(&self, action: StandardAction) -> Vec<Chord> {
        let without_person: Vec<Entry> = self
            .entries
            .iter()
            .filter(|e| e.layer != Layer::Person)
            .cloned()
            .collect();
        effective::effective(self.platform, &without_person, Context::Normal)
            .into_iter()
            .filter(|(bound, _)| *bound == Action::Standard(action))
            .map(|(_, chord)| chord)
            .collect()
    }

    fn is_free(&self, chord: DefaultChord) -> bool {
        chord
            .resolve(self.platform, Context::Normal)
            .is_none_or(|concrete| reserved_reason(self.platform, concrete).is_none())
    }

    fn refusal(&self, line: usize, chord: DefaultChord) -> OverrideProblem {
        let shown = chord.resolve(self.platform, Context::Normal).map_or_else(
            || chord.canonical(),
            |concrete| concrete.display(self.platform),
        );
        let reason = chord
            .resolve(self.platform, Context::Normal)
            .and_then(|concrete| reserved_reason(self.platform, concrete))
            .unwrap_or("it is kept for the system");
        OverrideProblem {
            line,
            what: format!("{shown} can't be used: {reason}"),
        }
    }

    fn rebuilt(mut self) -> Keymap {
        self.rebuild();
        self
    }

    fn rebuild(&mut self) {
        self.resolved = Resolved::build(self.platform, &self.entries);
    }
}

#[cfg(test)]
mod forgo_tests;
#[cfg(test)]
mod terminal_tests;
#[cfg(test)]
mod tests;
