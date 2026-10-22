//! The standard actions: what the platform's shortcuts mean, independent of which key does it.

use crate::action::ActionIdError;

/// A Space (virtual desktop) number, 1 to 9.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SpaceNumber(u8);

impl SpaceNumber {
    /// Space `n`, when `n` is 1 to 9.
    pub fn new(n: u8) -> Option<SpaceNumber> {
        (1..=9).contains(&n).then_some(SpaceNumber(n))
    }

    /// The number, 1 to 9.
    pub fn get(self) -> u8 {
        self.0
    }
}

/// An action every app and the desktop share the meaning of. Which key does it is decided by
/// the platform's conventions, never here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum StandardAction {
    /// Open the launcher, the place to search for and start apps.
    Launcher,
    /// Switch to the next keyboard input language.
    NextInputSource,
    /// Open the emoji and symbols picker.
    EmojiAndSymbols,
    /// Switch to the next app.
    NextApp,
    /// Switch to the previous app.
    PreviousApp,
    /// Switch to the app's next window.
    NextWindow,
    /// Open the app's settings.
    Settings,
    /// Cancel the operation in progress.
    Cancel,
    /// Open the app's help.
    Help,
    /// Select everything.
    SelectAll,
    /// Copy the selection.
    Copy,
    /// Cut the selection.
    Cut,
    /// Paste what was copied.
    Paste,
    /// Paste, taking on the style of the place it lands.
    PasteAndMatchStyle,
    /// Undo the last change.
    Undo,
    /// Redo the change just undone.
    Redo,
    /// Find text.
    Find,
    /// Go to the next match.
    FindNext,
    /// Go to the previous match.
    FindPrevious,
    /// Use the selection as the thing to find.
    UseSelectionForFind,
    /// Scroll to the selection.
    JumpToSelection,
    /// Make the selection bold.
    Bold,
    /// Make the selection italic.
    Italic,
    /// Underline the selection.
    Underline,
    /// Show the font chooser.
    ShowFonts,
    /// Show the colour chooser.
    ShowColors,
    /// Make things bigger.
    Bigger,
    /// Make things smaller.
    Smaller,
    /// Back to the actual size, after bigger or smaller.
    ActualSize,
    /// Hide the app.
    Hide,
    /// Hide every other app.
    HideOthers,
    /// Minimise the window.
    Minimize,
    /// Minimise all of the app's windows.
    MinimizeAll,
    /// Make something new.
    New,
    /// Open something.
    Open,
    /// Save.
    Save,
    /// Save under a new name or as a copy.
    SaveAs,
    /// Print.
    Print,
    /// Choose paper and page settings.
    PageSetup,
    /// Close the window.
    Close,
    /// Close all of the app's windows.
    CloseAll,
    /// Quit the app.
    Quit,
    /// Show or hide the toolbar.
    ToggleToolbar,
    /// Show or hide the sidebar.
    ToggleSidebar,
    /// Go full screen, or leave it.
    FullScreen,
    /// Show or hide the dock.
    ToggleDock,
    /// Preview the selected item.
    QuickLook,
    /// Show the item in the file manager.
    Reveal,
    /// Move the selected items to the Trash.
    MoveToTrash,
    /// Make a new folder where the person is.
    NewFolder,
    /// Empty the Trash.
    EmptyTrash,
    /// Capture the whole screen.
    ScreenshotScreen,
    /// Capture a part of the screen.
    ScreenshotSelection,
    /// Open the screenshot tools.
    ScreenshotTools,
    /// Force an unresponsive app to quit.
    ForceQuit,
    /// Lock the screen.
    LockScreen,
    /// Log out.
    LogOut,
    /// Show every window and Space at once.
    MissionControl,
    /// Show all of the app's windows.
    AppWindows,
    /// Go to the Space on the left.
    SpaceLeft,
    /// Go to the Space on the right.
    SpaceRight,
    /// Tile the window on the left half of the screen.
    TileLeft,
    /// Tile the window on the right half of the screen.
    TileRight,
    /// Tile the window on the top half of the screen.
    TileTop,
    /// Tile the window on the bottom half of the screen.
    TileBottom,
    /// Tile the window in the top left quarter of the screen.
    TileTopLeft,
    /// Tile the window in the top right quarter of the screen.
    TileTopRight,
    /// Tile the window in the bottom left quarter of the screen.
    TileBottomLeft,
    /// Tile the window in the bottom right quarter of the screen.
    TileBottomRight,
    /// Make the window fill the screen without going full screen.
    Fill,
    /// Move the window to the middle of the screen, keeping its size.
    Center,
    /// Give the window the size and place it had before it was tiled, filled or centred.
    ReturnToPreviousSize,
    /// Go to Space `n`.
    SwitchToSpace(SpaceNumber),
}

type Named = (StandardAction, &'static str);

/// Every action except the Space switches, with its slug.
const NAMED: &[Named] = &[
    (StandardAction::Launcher, "launcher"),
    (StandardAction::NextInputSource, "next-input-source"),
    (StandardAction::EmojiAndSymbols, "emoji-and-symbols"),
    (StandardAction::NextApp, "next-app"),
    (StandardAction::PreviousApp, "previous-app"),
    (StandardAction::NextWindow, "next-window"),
    (StandardAction::Settings, "settings"),
    (StandardAction::Cancel, "cancel"),
    (StandardAction::Help, "help"),
    (StandardAction::SelectAll, "select-all"),
    (StandardAction::Copy, "copy"),
    (StandardAction::Cut, "cut"),
    (StandardAction::Paste, "paste"),
    (StandardAction::PasteAndMatchStyle, "paste-and-match-style"),
    (StandardAction::Undo, "undo"),
    (StandardAction::Redo, "redo"),
    (StandardAction::Find, "find"),
    (StandardAction::FindNext, "find-next"),
    (StandardAction::FindPrevious, "find-previous"),
    (
        StandardAction::UseSelectionForFind,
        "use-selection-for-find",
    ),
    (StandardAction::JumpToSelection, "jump-to-selection"),
    (StandardAction::Bold, "bold"),
    (StandardAction::Italic, "italic"),
    (StandardAction::Underline, "underline"),
    (StandardAction::ShowFonts, "show-fonts"),
    (StandardAction::ShowColors, "show-colors"),
    (StandardAction::Bigger, "bigger"),
    (StandardAction::Smaller, "smaller"),
    (StandardAction::ActualSize, "actual-size"),
    (StandardAction::Hide, "hide"),
    (StandardAction::HideOthers, "hide-others"),
    (StandardAction::Minimize, "minimize"),
    (StandardAction::MinimizeAll, "minimize-all"),
    (StandardAction::New, "new"),
    (StandardAction::Open, "open"),
    (StandardAction::Save, "save"),
    (StandardAction::SaveAs, "save-as"),
    (StandardAction::Print, "print"),
    (StandardAction::PageSetup, "page-setup"),
    (StandardAction::Close, "close"),
    (StandardAction::CloseAll, "close-all"),
    (StandardAction::Quit, "quit"),
    (StandardAction::ToggleToolbar, "toggle-toolbar"),
    (StandardAction::ToggleSidebar, "toggle-sidebar"),
    (StandardAction::FullScreen, "full-screen"),
    (StandardAction::ToggleDock, "toggle-dock"),
    (StandardAction::QuickLook, "quick-look"),
    (StandardAction::Reveal, "reveal"),
    (StandardAction::MoveToTrash, "move-to-trash"),
    (StandardAction::NewFolder, "new-folder"),
    (StandardAction::EmptyTrash, "empty-trash"),
    (StandardAction::ScreenshotScreen, "screenshot-screen"),
    (StandardAction::ScreenshotSelection, "screenshot-selection"),
    (StandardAction::ScreenshotTools, "screenshot-tools"),
    (StandardAction::ForceQuit, "force-quit"),
    (StandardAction::LockScreen, "lock-screen"),
    (StandardAction::LogOut, "log-out"),
    (StandardAction::MissionControl, "mission-control"),
    (StandardAction::AppWindows, "app-windows"),
    (StandardAction::SpaceLeft, "space-left"),
    (StandardAction::SpaceRight, "space-right"),
    (StandardAction::TileLeft, "tile-left"),
    (StandardAction::TileRight, "tile-right"),
    (StandardAction::TileTop, "tile-top"),
    (StandardAction::TileBottom, "tile-bottom"),
    (StandardAction::TileTopLeft, "tile-top-left"),
    (StandardAction::TileTopRight, "tile-top-right"),
    (StandardAction::TileBottomLeft, "tile-bottom-left"),
    (StandardAction::TileBottomRight, "tile-bottom-right"),
    (StandardAction::Fill, "fill"),
    (StandardAction::Center, "center"),
    (
        StandardAction::ReturnToPreviousSize,
        "return-to-previous-size",
    ),
];

const SPACE_SLUG: &str = "switch-to-space-";

impl StandardAction {
    /// Every standard action, the nine Space switches included.
    pub fn all() -> impl Iterator<Item = StandardAction> {
        NAMED.iter().map(|(action, _)| *action).chain(
            (1..=9)
                .filter_map(SpaceNumber::new)
                .map(StandardAction::SwitchToSpace),
        )
    }

    /// The stable id used in override files: "copy", "switch-to-space-3".
    pub fn slug(self) -> String {
        match self {
            StandardAction::SwitchToSpace(n) => format!("{SPACE_SLUG}{}", n.get()),
            other => NAMED
                .iter()
                .find(|(action, _)| *action == other)
                .map(|(_, slug)| (*slug).to_owned())
                .unwrap_or_default(),
        }
    }

    /// The action with this slug.
    pub fn from_slug(slug: &str) -> Option<StandardAction> {
        if let Some(digits) = slug.strip_prefix(SPACE_SLUG) {
            return digits
                .parse::<u8>()
                .ok()
                .and_then(SpaceNumber::new)
                .map(StandardAction::SwitchToSpace);
        }
        NAMED
            .iter()
            .find(|(_, name)| *name == slug)
            .map(|(action, _)| *action)
    }

    /// Plain words for the action ("select all"), for messages.
    pub fn label(self) -> String {
        self.slug().replace('-', " ")
    }
}

impl std::str::FromStr for StandardAction {
    type Err = ActionIdError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        StandardAction::from_slug(text)
            .ok_or_else(|| ActionIdError::UnknownStandard(text.to_owned()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn there_are_eighty_one_and_slugs_are_unique_and_round_trip() {
        let all: Vec<StandardAction> = StandardAction::all().collect();
        assert_eq!(all.len(), 81);
        let mut slugs: Vec<String> = all.iter().map(|a| a.slug()).collect();
        for (action, slug) in all.iter().zip(&slugs) {
            assert!(!slug.is_empty(), "{action:?} has no slug");
            assert_eq!(StandardAction::from_slug(slug), Some(*action));
        }
        slugs.sort();
        slugs.dedup();
        assert_eq!(slugs.len(), 81);
    }

    #[test]
    fn labels_are_plain_words_and_bad_slugs_are_refused() {
        assert_eq!(StandardAction::SelectAll.label(), "select all");
        assert_eq!(StandardAction::TileTopLeft.label(), "tile top left");
        assert_eq!(
            StandardAction::ReturnToPreviousSize.label(),
            "return to previous size"
        );
        assert_eq!(
            StandardAction::from_slug("center"),
            Some(StandardAction::Center)
        );
        assert_eq!(StandardAction::from_slug("switch-to-space-0"), None);
        assert_eq!(StandardAction::from_slug("switch-to-space-10"), None);
        assert_eq!(StandardAction::from_slug("teleport"), None);
    }
}
