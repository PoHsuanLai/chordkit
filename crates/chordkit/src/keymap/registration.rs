//! What an app registers: its actions, their defaults and fallbacks, and the standard actions
//! it does not offer.

use super::layers::Binding;
use crate::action::{AppAction, AppId};
use crate::chord::DefaultChord;
use crate::standard_action::StandardAction;

/// One app action and the chord it asks for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Row {
    pub(crate) action: AppAction,
    pub(crate) binding: Binding,
}

/// An app's registration, built step by step and given to `Keymap::register_with`.
///
/// ```
/// use chordkit::{AppAction, AppId, Registration, StandardAction};
///
/// let app = AppId::new("temor").unwrap();
/// let fonts = AppAction::new("temor.fonts").unwrap();
/// let registration = Registration::new(&app)
///     .forgo(StandardAction::ShowFonts)
///     .action(fonts, "Primary+T".parse().unwrap());
/// # let _ = registration;
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Registration {
    app: AppId,
    forgone: Vec<StandardAction>,
    rows: Vec<Row>,
}

impl Registration {
    /// An empty registration for `app`.
    pub fn new(app: &AppId) -> Registration {
        Registration {
            app: app.clone(),
            forgone: Vec::new(),
            rows: Vec::new(),
        }
    }

    /// The app does not offer `action`, so the action no longer holds its chord in this app.
    /// Only actions with [`Reach::IfOffered`](crate::Reach::IfOffered) can be forgone; the
    /// registration is refused for a universal one.
    pub fn forgo(mut self, action: StandardAction) -> Registration {
        self.forgone.push(action);
        self
    }

    /// Forgoes each of `actions`.
    pub fn forgo_all(mut self, actions: impl IntoIterator<Item = StandardAction>) -> Registration {
        self.forgone.extend(actions);
        self
    }

    /// Declares `action` with its portable default chord. A repeated action gets both chords.
    pub fn action(mut self, action: AppAction, chord: DefaultChord) -> Registration {
        self.rows.push(Row {
            action,
            binding: Binding::from(chord),
        });
        self
    }

    /// Declares `action` with a default and a fallback. The fallback serves each context where the
    /// default has no live form (a Primary chord with Shift and Alt in a terminal, say). It also
    /// replaces the default everywhere when the default is reserved or taken, provided it is live
    /// wherever the default was and free there; otherwise the row is refused. Defaults settle
    /// before fallbacks, so declaration order only matters between two fallbacks wanting one
    /// chord: the first declared wins and the other is refused.
    pub fn action_with_fallback(
        mut self,
        action: AppAction,
        chord: DefaultChord,
        fallback: DefaultChord,
    ) -> Registration {
        self.rows.push(Row {
            action,
            binding: Binding::with_fallback(chord, fallback),
        });
        self
    }

    pub(crate) fn app(&self) -> &AppId {
        &self.app
    }

    pub(crate) fn forgone(&self) -> &[StandardAction] {
        &self.forgone
    }

    pub(crate) fn rows(&self) -> &[Row] {
        &self.rows
    }
}
