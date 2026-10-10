//! How far a standard action reaches: every app, or only the apps that offer the feature.
//!
//! The split follows the Mac menus. A universal action is part of being a desktop app or of the
//! desktop itself (clipboard, undo, windows, quit, find, the desktop's own switchers), so every
//! app honours it and none may give up its chord. An action that is only meaningful where the
//! app has the feature (fonts, text styles, printing, documents, toolbars) is offered by
//! those apps alone; an app without the feature may [forgo](crate::Registration::forgo) it and
//! take the chord for something of its own.

use crate::standard_action::StandardAction;

/// Whether every app honours a standard action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Reach {
    /// Every app honours it; no app may take its chord.
    Universal,
    /// Only apps that offer the feature honour it; others may forgo it.
    IfOffered,
}

impl StandardAction {
    /// Whether every app honours this action, or only apps that offer the feature.
    ///
    /// | Reach | Actions |
    /// |---|---|
    /// | Universal | launcher, next input source, emoji and symbols, next/previous app, next window, settings, cancel, help, select all, copy, cut, paste, undo, redo, find, find next, find previous, hide, hide others, minimise, minimise all, close, close all, quit, full screen, toggle dock, app windows, mission control, force quit, lock screen, log out, the screenshots, the Space switches |
    /// | IfOffered | paste and match style, use selection for find, jump to selection, bold, italic, underline, show fonts, show colours, bigger, smaller, new, open, save, save as, print, page setup, toggle toolbar, toggle sidebar, quick look, reveal |
    pub fn reach(self) -> Reach {
        use StandardAction as S;
        match self {
            S::PasteAndMatchStyle
            | S::UseSelectionForFind
            | S::JumpToSelection
            | S::Bold
            | S::Italic
            | S::Underline
            | S::ShowFonts
            | S::ShowColors
            | S::Bigger
            | S::Smaller
            | S::New
            | S::Open
            | S::Save
            | S::SaveAs
            | S::Print
            | S::PageSetup
            | S::ToggleToolbar
            | S::ToggleSidebar
            | S::QuickLook
            | S::Reveal => Reach::IfOffered,
            S::Launcher
            | S::NextInputSource
            | S::EmojiAndSymbols
            | S::NextApp
            | S::PreviousApp
            | S::NextWindow
            | S::Settings
            | S::Cancel
            | S::Help
            | S::SelectAll
            | S::Copy
            | S::Cut
            | S::Paste
            | S::Undo
            | S::Redo
            | S::Find
            | S::FindNext
            | S::FindPrevious
            | S::Hide
            | S::HideOthers
            | S::Minimize
            | S::MinimizeAll
            | S::Close
            | S::CloseAll
            | S::Quit
            | S::FullScreen
            | S::ToggleDock
            | S::ScreenshotScreen
            | S::ScreenshotSelection
            | S::ScreenshotTools
            | S::ForceQuit
            | S::LockScreen
            | S::LogOut
            | S::MissionControl
            | S::AppWindows
            | S::SpaceLeft
            | S::SpaceRight
            | S::SwitchToSpace(_) => Reach::Universal,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::standard_action::SpaceNumber;

    #[test]
    fn reach_table() {
        use StandardAction as S;
        type Case = (StandardAction, Reach);
        let cases: &[Case] = &[
            (S::Copy, Reach::Universal),
            (S::Quit, Reach::Universal),
            (S::Find, Reach::Universal),
            (
                S::SwitchToSpace(SpaceNumber::new(3).unwrap_or_else(|| panic!("3"))),
                Reach::Universal,
            ),
            (S::ShowFonts, Reach::IfOffered),
            (S::Bold, Reach::IfOffered),
            (S::Print, Reach::IfOffered),
            (S::PageSetup, Reach::IfOffered),
        ];
        for (action, expected) in cases {
            assert_eq!(action.reach(), *expected, "{action:?}");
        }
    }

    #[test]
    fn some_actions_are_offered_only() {
        let offered = StandardAction::all()
            .filter(|a| a.reach() == Reach::IfOffered)
            .count();
        assert_eq!(offered, 20);
    }
}
