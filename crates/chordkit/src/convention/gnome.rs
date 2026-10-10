//! GNOME: the CUA base with GNOME's own defaults.

use super::{
    Row::{Key, Unbound},
    Spaces, TableRow,
};
use crate::standard_action::StandardAction as S;

pub(super) static SPACES: Spaces =
    Spaces::None("In GNOME, Super plus a number starts the dock's apps, not a desktop.");

pub(super) static ROWS: &[TableRow] = &[
    (S::Launcher, Key("Super+A")),
    (S::EmojiAndSymbols, Key("Primary+.")),
    (S::NextApp, Key("Super+Tab")),
    (S::PreviousApp, Key("Super+Shift+Tab")),
    (S::NextWindow, Key("Super+`")),
    (S::FindNext, Key("Primary+G")),
    (S::FindPrevious, Key("Primary+Shift+G")),
    (S::Minimize, Key("Super+H")),
    (
        S::ScreenshotScreen,
        Unbound("GNOME has one screenshot tool, on the Print key."),
    ),
    (
        S::ScreenshotSelection,
        Unbound("GNOME has one screenshot tool, on the Print key."),
    ),
    (S::ScreenshotTools, Key("PrintScreen")),
    (
        S::ForceQuit,
        Unbound("GNOME has no key to force an app to quit."),
    ),
    (S::MissionControl, Key("Super")),
    (S::SpaceLeft, Key("Super+PageUp")),
    (S::SpaceRight, Key("Super+PageDown")),
];
