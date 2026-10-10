//! Actions: what a key press means.

use std::str::FromStr;

use thiserror::Error;

use crate::standard_action::StandardAction;

/// Why an id could not be read.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ActionIdError {
    /// Not a valid app name.
    #[error("\"{0}\" is not a valid app name: use lowercase letters, digits, - and _")]
    BadApp(String),
    /// Not a valid app action id.
    #[error(
        "\"{0}\" is not a valid action id: write app.action using lowercase letters, digits, - and _"
    )]
    BadAction(String),
    /// No standard action has this name.
    #[error("\"{0}\" is not a standard action")]
    UnknownStandard(String),
}

fn valid_segment(text: &str) -> bool {
    !text.is_empty()
        && text
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
}

/// An app's name, the first part of its action ids: "mail".
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AppId(String);

impl AppId {
    /// An app name: lowercase letters, digits, `-` and `_`.
    pub fn new(name: &str) -> Result<AppId, ActionIdError> {
        if valid_segment(name) {
            Ok(AppId(name.to_owned()))
        } else {
            Err(ActionIdError::BadApp(name.to_owned()))
        }
    }

    /// The name.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for AppId {
    type Err = ActionIdError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        AppId::new(text)
    }
}

/// An action an app declares, named "app.action": "mail.compose".
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AppAction(String);

impl AppAction {
    /// An app action id: two or more dot-separated parts of lowercase letters, digits, `-`, `_`.
    pub fn new(id: &str) -> Result<AppAction, ActionIdError> {
        let parts = id.split('.').count();
        if parts >= 2 && id.split('.').all(valid_segment) {
            Ok(AppAction(id.to_owned()))
        } else {
            Err(ActionIdError::BadAction(id.to_owned()))
        }
    }

    /// The whole id.
    pub fn id(&self) -> &str {
        &self.0
    }

    /// The app it belongs to: the part before the first dot.
    pub fn app(&self) -> AppId {
        let first = self.0.split('.').next().unwrap_or_default();
        AppId(first.to_owned())
    }
}

impl FromStr for AppAction {
    type Err = ActionIdError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        AppAction::new(text)
    }
}

/// What a key press means: a shared standard action or one an app declared.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Action {
    /// A standard action.
    Standard(StandardAction),
    /// An app's own action.
    App(AppAction),
}

impl Action {
    /// The stable id: a standard slug ("copy") or an app id ("mail.compose").
    pub fn id(&self) -> String {
        match self {
            Action::Standard(action) => action.slug(),
            Action::App(action) => action.id().to_owned(),
        }
    }
}

impl FromStr for Action {
    type Err = ActionIdError;

    fn from_str(id: &str) -> Result<Self, Self::Err> {
        if id.contains('.') {
            return AppAction::new(id).map(Action::App);
        }
        StandardAction::from_slug(id)
            .map(Action::Standard)
            .ok_or_else(|| ActionIdError::UnknownStandard(id.to_owned()))
    }
}

impl From<StandardAction> for Action {
    fn from(action: StandardAction) -> Action {
        Action::Standard(action)
    }
}

impl From<AppAction> for Action {
    fn from(action: AppAction) -> Action {
        Action::App(action)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_parse_and_name_their_app() {
        let compose = AppAction::new("mail.compose").unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(compose.app().as_str(), "mail");
        assert_eq!(
            "copy".parse::<Action>(),
            Ok(Action::Standard(StandardAction::Copy))
        );
        assert_eq!("mail.compose".parse::<Action>(), Ok(Action::App(compose)));
    }

    #[test]
    fn bad_ids_are_refused_in_words() {
        assert!(AppAction::new("compose").is_err());
        assert!(AppAction::new("Mail.compose").is_err());
        assert!(AppAction::new("mail.").is_err());
        assert!(AppId::new("").is_err());
        assert_eq!(
            "teleport".parse::<Action>(),
            Err(ActionIdError::UnknownStandard("teleport".into()))
        );
    }
}
