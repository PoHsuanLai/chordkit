//! KDE Plasma: the CUA base with Plasma's own defaults.

use super::{
    Row::{Key, Unbound},
    Spaces, TableRow,
};
use crate::standard_action::StandardAction as S;

pub(super) static SPACES: Spaces = Spaces::Keys([
    "Ctrl+F1", "Ctrl+F2", "Ctrl+F3", "Ctrl+F4", "Ctrl+F5", "Ctrl+F6", "Ctrl+F7", "Ctrl+F8",
    "Ctrl+F9",
]);

pub(super) static ROWS: &[TableRow] = &[
    (S::Launcher, Key("Alt+Space")),
    (S::NextInputSource, Key("Super+Alt+K")),
    (S::EmojiAndSymbols, Key("Super+.")),
    (S::Minimize, Key("Super+PageDown")),
    (S::ScreenshotScreen, Key("Shift+PrintScreen")),
    (S::ScreenshotSelection, Key("Super+Shift+PrintScreen")),
    (S::ScreenshotTools, Key("PrintScreen")),
    (S::LockScreen, Key("Ctrl+Alt+L")),
    (S::MissionControl, Key("Super+W")),
    (
        S::AppWindows,
        Unbound("Plasma has no key for an app's windows overview."),
    ),
    (S::SpaceLeft, Key("Ctrl+Super+Left")),
    (S::SpaceRight, Key("Ctrl+Super+Right")),
];
