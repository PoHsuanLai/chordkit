//! Registering an app's actions: refused when the chord is not the app's to take.

use super::Keymap;
use super::conflict::Conflict;
use super::effective::effective;
use super::layers::{Entry, Layer, upsert};
use crate::action::{Action, AppAction, AppId};
use crate::chord::{Chord, DefaultChord};
use crate::platform::Context;
use crate::reserved::reserved_reason;

/// Contexts whose Primary differs; a chord must be free in each.
const CHECKED: [Context; 2] = [Context::Normal, Context::Terminal];

impl Keymap {
    /// Registers `app`'s actions with their portable default chords. All or nothing: the first
    /// chord the platform reserves, a standard action owns, or another action holds is refused.
    /// A person's own changes never refuse a registration; they simply win later.
    pub fn register(
        &mut self,
        app: &AppId,
        actions: &[(AppAction, DefaultChord)],
    ) -> Result<(), Conflict> {
        let mut staged: Vec<Entry> = Vec::new();
        for (action, chord) in actions {
            if action.app() != *app {
                return Err(Conflict::ForeignAction {
                    action: action.id().to_owned(),
                    app: app.as_str().to_owned(),
                });
            }
            self.check(action, *chord, &staged)?;
            stage(&mut staged, action, *chord);
        }
        for entry in staged {
            upsert(&mut self.entries, entry);
        }
        self.rebuild();
        Ok(())
    }

    fn check(
        &self,
        action: &AppAction,
        chord: DefaultChord,
        staged: &[Entry],
    ) -> Result<(), Conflict> {
        let wanting = Action::App(action.clone());
        let live: Vec<Entry> = self
            .entries
            .iter()
            .filter(|entry| entry.layer != Layer::Person)
            .chain(staged)
            .cloned()
            .collect();
        for context in CHECKED {
            let Some(concrete) = chord.resolve(self.platform, context) else {
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

fn stage(staged: &mut Vec<Entry>, action: &AppAction, chord: DefaultChord) {
    let wanted = Action::App(action.clone());
    match staged.iter_mut().find(|entry| entry.action == wanted) {
        Some(entry) => entry.chords.push(chord),
        None => staged.push(Entry {
            action: wanted,
            layer: Layer::App,
            chords: vec![chord],
        }),
    }
}
