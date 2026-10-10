//! Windows: the CUA base with the rows Windows does differently.

use super::{
    Row::{Key, Unbound},
    Spaces, TableRow,
};
use crate::standard_action::StandardAction as S;

pub(super) static SPACES: Spaces =
    Spaces::None("Windows has no key for a numbered virtual desktop.");

pub(super) static ROWS: &[TableRow] = &[
    (S::Launcher, Key("Super+S")),
    (S::EmojiAndSymbols, Key("Super+.")),
    (
        S::NextWindow,
        Unbound("Windows has no key for another window of the same app."),
    ),
    (S::Redo, Key("Primary+Y")),
    (S::Quit, Key("Alt+F4")),
    (S::Minimize, Key("Super+Down")),
    (S::MinimizeAll, Key("Super+M")),
    (S::ScreenshotSelection, Key("Super+Shift+S")),
    (S::ForceQuit, Key("Ctrl+Shift+Escape")),
    (
        S::LogOut,
        Unbound("Ctrl+Alt+Delete belongs to the system and apps never see it."),
    ),
    (S::MissionControl, Key("Super+Tab")),
    (S::SpaceLeft, Key("Ctrl+Super+Left")),
    (S::SpaceRight, Key("Ctrl+Super+Right")),
];
