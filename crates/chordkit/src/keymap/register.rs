//! Registering an app's actions: refused when the chord is not the app's to take.

use super::Keymap;
use super::conflict::Conflict;
use super::effective::effective;
use super::layers::{Binding, Entry, Layer, upsert};
use super::registration::{Registration, Row};
use crate::action::{Action, AppAction, AppId};
use crate::chord::{Chord, DefaultChord};
use crate::platform::Context;
use crate::reach::Reach;
use crate::reserved::reserved_reason;
use crate::standard_action::StandardAction;

/// Contexts whose Primary differs; a chord must be free in each.
const CHECKED: [Context; 2] = [Context::Normal, Context::Terminal];

impl Keymap {
    /// Registers `app`'s actions with their portable default chords. All or nothing: the first
    /// chord the platform reserves, a standard action owns, or another action holds is refused.
    /// A person's own changes never refuse a registration; they simply win later. This is
    /// `register_with` for an app that forgoes nothing and has no fallbacks.
    pub fn register(
        &mut self,
        app: &AppId,
        actions: &[(AppAction, DefaultChord)],
    ) -> Result<(), Conflict> {
        let registration = actions
            .iter()
            .fold(Registration::new(app), |registration, (action, chord)| {
                registration.action(action.clone(), *chord)
            });
        self.register_with(&registration)
    }

    /// Registers what `registration` declares, all or nothing, as `register` does. Standard
    /// actions it forgoes stop holding their chords in this keymap (a universal one is refused).
    /// A refused app action is remembered, so a person's own chord for it still binds.
    pub fn register_with(&mut self, registration: &Registration) -> Result<(), Conflict> {
        let app = registration.app();
        if let Some(foreign) = registration.rows().iter().find(|r| r.action.app() != *app) {
            return Err(Conflict::ForeignAction {
                action: foreign.action.id().to_owned(),
                app: app.as_str().to_owned(),
            });
        }
        let mut staged = forgone_entries(app, registration.forgone())?;
        let mut refused: Vec<AppAction> = Vec::new();
        let mut first: Option<Conflict> = None;
        for row in registration.rows() {
            match self.settle(row, &staged) {
                Ok(binding) => stage(&mut staged, row, binding),
                Err(conflict) => {
                    refused.push(row.action.clone());
                    first.get_or_insert(conflict);
                }
            }
        }
        if let Some(conflict) = first {
            self.remember_refused(refused);
            return Err(conflict);
        }
        for entry in staged {
            upsert(&mut self.entries, entry);
        }
        self.rebuild();
        Ok(())
    }

    /// Marks each refused action as registered with no chords, unless an app already registered
    /// it, so the person's chord for it is live.
    fn remember_refused(&mut self, refused: Vec<AppAction>) {
        for action in refused {
            let wanted = Action::App(action);
            let known = self
                .entries
                .iter()
                .any(|e| e.layer == Layer::App && e.action == wanted);
            if !known {
                self.entries.push(Entry {
                    action: wanted,
                    layer: Layer::App,
                    chords: Vec::new(),
                });
            }
        }
        self.rebuild();
    }

    /// The binding `row` takes: its own when its live chords are free, else, when it has a
    /// fallback, the fallback alone if that is free. Both taken refuses with the default's
    /// conflict.
    fn settle(&self, row: &Row, staged: &[Entry]) -> Result<Binding, Conflict> {
        match self.check(row, row.binding, staged) {
            Ok(()) => Ok(row.binding),
            Err(conflict) => match row.binding.fallback() {
                Some(fallback) => {
                    let alone = Binding::from(fallback);
                    self.check(row, alone, staged)
                        .map(|()| alone)
                        .map_err(|_| conflict)
                }
                None => Err(conflict),
            },
        }
    }

    fn check(&self, row: &Row, binding: Binding, staged: &[Entry]) -> Result<(), Conflict> {
        let wanting = Action::App(row.action.clone());
        let live: Vec<Entry> = self
            .entries
            .iter()
            .filter(|entry| entry.layer != Layer::Person)
            .chain(staged)
            .cloned()
            .collect();
        for context in CHECKED {
            let Some(concrete) = binding.live(self.platform, context) else {
                continue;
            };
            let shown = concrete.display(self.platform);
            if let Some(reason) = reserved_reason(self.platform, concrete) {
                return Err(Conflict::Reserved {
                    shown,
                    wanted_by: wanting.id(),
                    reason,
                });
            }
            let holder = effective(self.platform, &live, context).into_iter().find(
                |(held, held_chord): &(Action, Chord)| *held_chord == concrete && *held != wanting,
            );
            if let Some((held, _)) = holder {
                return Err(clash(shown, held, &wanting));
            }
        }
        Ok(())
    }
}

/// The entries that unbind the standard actions an app does not offer; a universal action is
/// refused.
fn forgone_entries(app: &AppId, forgone: &[StandardAction]) -> Result<Vec<Entry>, Conflict> {
    forgone
        .iter()
        .map(|action| match action.reach() {
            Reach::IfOffered => Ok(Entry {
                action: Action::Standard(*action),
                layer: Layer::Forgone,
                chords: Vec::new(),
            }),
            Reach::Universal => Err(Conflict::Universal {
                action: *action,
                meaning: action.label(),
                app: app.as_str().to_owned(),
            }),
        })
        .collect()
}

fn clash(shown: String, held: Action, wanting: &Action) -> Conflict {
    match held {
        Action::Standard(action) => Conflict::Standard {
            shown,
            action,
            meaning: action.label(),
            wanted_by: wanting.id(),
        },
        other => Conflict::Taken {
            shown,
            held_by: other.id(),
            wanted_by: wanting.id(),
        },
    }
}

fn stage(staged: &mut Vec<Entry>, row: &Row, binding: Binding) {
    let wanted = Action::App(row.action.clone());
    match staged.iter_mut().find(|entry| entry.action == wanted) {
        Some(entry) => entry.chords.push(binding),
        None => staged.push(Entry {
            action: wanted,
            layer: Layer::App,
            chords: vec![binding],
        }),
    }
}
