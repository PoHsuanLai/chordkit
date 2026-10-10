//! The common Ctrl-based conventions (IBM CUA lineage): the base of Windows and Linux.

use super::{
    Row::{Key, Unbound},
    Spaces, TableRow,
};
use crate::standard_action::StandardAction as S;

pub(super) static SPACES: Spaces =
    Spaces::None("There is no common key for a numbered virtual desktop on this platform.");

pub(super) static ROWS: &[TableRow] = &[
    (
        S::Launcher,
        Unbound("Each desktop has its own launcher key and there is no common one."),
    ),
    (S::NextInputSource, Key("Super+Space")),
    (S::EmojiAndSymbols, Key("Ctrl+.")),
    (S::NextApp, Key("Alt+Tab")),
    (S::PreviousApp, Key("Alt+Shift+Tab")),
    (S::NextWindow, Key("Alt+`")),
    (S::Settings, Key("Primary+,")),
    (S::Cancel, Key("Escape")),
    (S::Help, Key("F1")),
    (S::SelectAll, Key("Primary+A")),
    (S::Copy, Key("Primary+C")),
    (S::Cut, Key("Primary+X")),
    (S::Paste, Key("Primary+V")),
    (S::PasteAndMatchStyle, Key("Primary+Shift+V")),
    (S::Undo, Key("Primary+Z")),
    (S::Redo, Key("Primary+Shift+Z")),
    (S::Find, Key("Primary+F")),
    (S::FindNext, Key("F3")),
    (S::FindPrevious, Key("Shift+F3")),
    (
        S::UseSelectionForFind,
        Unbound("There is no common key for using the selection as the search."),
    ),
    (
        S::JumpToSelection,
        Unbound("There is no common key for jumping to the selection."),
    ),
    (S::Bold, Key("Primary+B")),
    (S::Italic, Key("Primary+I")),
    (S::Underline, Key("Primary+U")),
    (
        S::ShowFonts,
        Unbound("Fonts are chosen in the app's own controls here."),
    ),
    (
        S::ShowColors,
        Unbound("Colours are chosen in the app's own controls here."),
    ),
    (S::Bigger, Key("Primary++")),
    (S::Smaller, Key("Primary+-")),
    (S::Hide, Unbound("Apps are minimised here, not hidden.")),
    (
        S::HideOthers,
        Unbound("Apps are minimised here, not hidden."),
    ),
    (S::Minimize, Key("Alt+F9")),
    (
        S::MinimizeAll,
        Unbound("There is no common key for minimising all of an app's windows."),
    ),
    (S::New, Key("Primary+N")),
    (S::Open, Key("Primary+O")),
    (S::Save, Key("Primary+S")),
    (S::SaveAs, Key("Primary+Shift+S")),
    (S::Print, Key("Primary+P")),
    (
        S::PageSetup,
        Unbound("There is no common key for page setup."),
    ),
    (S::Close, Key("Primary+W")),
    (S::CloseAll, Key("Primary+Shift+W")),
    (S::Quit, Key("Primary+Q")),
    (
        S::ToggleToolbar,
        Unbound("There is no common key for the toolbar."),
    ),
    (S::ToggleSidebar, Key("F9")),
    (S::FullScreen, Key("F11")),
    (S::ToggleDock, Unbound("There is no dock to show or hide.")),
    (
        S::QuickLook,
        Unbound("There is no common key for previewing an item."),
    ),
    (
        S::Reveal,
        Unbound("There is no common key for showing an item in the file manager."),
    ),
    (S::ScreenshotScreen, Key("PrintScreen")),
    (S::ScreenshotSelection, Key("Shift+PrintScreen")),
    (
        S::ScreenshotTools,
        Unbound("The screenshot keys above already cover this."),
    ),
    (S::ForceQuit, Key("Ctrl+Alt+Escape")),
    (S::LockScreen, Key("Super+L")),
    (S::LogOut, Key("Ctrl+Alt+Delete")),
    (
        S::MissionControl,
        Unbound("There is no common key for the window overview."),
    ),
    (
        S::AppWindows,
        Unbound("There is no common key for an app's windows overview."),
    ),
    (S::SpaceLeft, Key("Ctrl+Alt+Left")),
    (S::SpaceRight, Key("Ctrl+Alt+Right")),
];
