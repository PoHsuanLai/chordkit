//! Why an app's shortcut was refused, in plain words.

use thiserror::Error;

use crate::standard_action::StandardAction;

/// A refused registration. Nothing is registered when any row is refused.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum Conflict {
    /// The platform or desktop keeps the chord for itself.
    #[error("{shown} can't be used for {wanted_by}: {reason}")]
    Reserved {
        /// The chord as this platform shows it.
        shown: String,
        /// The action that asked for it.
        wanted_by: String,
        /// Why it is kept, in plain words.
        reason: &'static str,
    },
    /// A standard action already has the chord, in every app.
    #[error("{shown} already means \"{meaning}\" in every app, so {wanted_by} can't use it")]
    Standard {
        /// The chord as this platform shows it.
        shown: String,
        /// The standard action that has it.
        action: StandardAction,
        /// That action in plain words.
        meaning: String,
        /// The action that asked for it.
        wanted_by: String,
    },
    /// Another registered action already has the chord.
    #[error("{shown} is already used by {held_by}, so {wanted_by} can't use it")]
    Taken {
        /// The chord as this platform shows it.
        shown: String,
        /// The action that has it.
        held_by: String,
        /// The action that asked for it.
        wanted_by: String,
    },
    /// An action id that does not start with the registering app's name.
    #[error("{action} doesn't belong to {app}: an app's action ids start with its name")]
    ForeignAction {
        /// The action id.
        action: String,
        /// The registering app.
        app: String,
    },
}
