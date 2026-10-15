//! Menu titles to standard actions. NSUserKeyEquivalents is keyed by the menu item's title,
//! which is localised, so a table is per language.

use chordkit::{StandardAction, SystemImport};

use crate::equivalent::parse_key_equivalent;

type Title = (&'static str, StandardAction);

/// One language's standard menu titles.
#[derive(Debug, Clone, Copy)]
pub struct MenuTitles {
    titles: &'static [Title],
}

/// The English titles of the standard menus.
pub const ENGLISH: MenuTitles = MenuTitles {
    titles: &[
        ("Undo", StandardAction::Undo),
        ("Redo", StandardAction::Redo),
        ("Cut", StandardAction::Cut),
        ("Copy", StandardAction::Copy),
        ("Paste", StandardAction::Paste),
        ("Paste and Match Style", StandardAction::PasteAndMatchStyle),
        ("Select All", StandardAction::SelectAll),
        ("Find", StandardAction::Find),
        ("Find Next", StandardAction::FindNext),
        ("Find Previous", StandardAction::FindPrevious),
        (
            "Use Selection for Find",
            StandardAction::UseSelectionForFind,
        ),
        ("Jump to Selection", StandardAction::JumpToSelection),
        ("Bold", StandardAction::Bold),
        ("Italic", StandardAction::Italic),
        ("Underline", StandardAction::Underline),
        ("Show Fonts", StandardAction::ShowFonts),
        ("Show Colors", StandardAction::ShowColors),
        ("Bigger", StandardAction::Bigger),
        ("Smaller", StandardAction::Smaller),
        ("Hide Others", StandardAction::HideOthers),
        ("Minimize", StandardAction::Minimize),
        ("Minimize All", StandardAction::MinimizeAll),
        ("New", StandardAction::New),
        ("Open", StandardAction::Open),
        ("Save", StandardAction::Save),
        ("Save As", StandardAction::SaveAs),
        ("Print", StandardAction::Print),
        ("Page Setup", StandardAction::PageSetup),
        ("Close", StandardAction::Close),
        ("Close All", StandardAction::CloseAll),
        ("Settings", StandardAction::Settings),
        ("Preferences", StandardAction::Settings),
        ("Show Toolbar", StandardAction::ToggleToolbar),
        ("Hide Toolbar", StandardAction::ToggleToolbar),
        ("Show Sidebar", StandardAction::ToggleSidebar),
        ("Hide Sidebar", StandardAction::ToggleSidebar),
        ("Enter Full Screen", StandardAction::FullScreen),
        ("Exit Full Screen", StandardAction::FullScreen),
        ("Top Left", StandardAction::TileTopLeft),
        ("Top Right", StandardAction::TileTopRight),
        ("Bottom Left", StandardAction::TileBottomLeft),
        ("Bottom Right", StandardAction::TileBottomRight),
        (
            "Return to Previous Size",
            StandardAction::ReturnToPreviousSize,
        ),
    ],
};

impl MenuTitles {
    /// A language's titles.
    pub const fn new(titles: &'static [Title]) -> MenuTitles {
        MenuTitles { titles }
    }

    fn action_for(&self, title: &str) -> Option<StandardAction> {
        let plain = title.trim_end_matches(['\u{2026}', '.']).trim();
        self.titles
            .iter()
            .find(|(known, _)| *known == plain)
            .map(|(_, action)| *action)
    }
}

/// The system bindings in `pairs`, the (menu title, key equivalent) entries of an
/// NSUserKeyEquivalents dictionary. Titles that are not standard are ignored; a value that
/// cannot be read is reported in `problems`. The person's shortcut replaces the convention.
pub fn import_user_key_equivalents<'a>(
    titles: &MenuTitles,
    pairs: impl IntoIterator<Item = (&'a str, &'a str)>,
) -> SystemImport {
    let mut import = SystemImport::default();
    for (title, value) in pairs {
        let Some(action) = titles.action_for(title) else {
            continue;
        };
        match parse_key_equivalent(value) {
            Ok(chord) => import.add(action, vec![chord]),
            Err(why) => import
                .problems
                .push(format!("the \"{title}\" shortcut: {why}")),
        }
    }
    import
}

#[cfg(test)]
mod tests {
    use super::*;

    fn canonical(import: &SystemImport, action: StandardAction) -> Vec<String> {
        import
            .bindings
            .iter()
            .filter(|b| b.action == action)
            .flat_map(|b| b.chords.iter().map(|c| c.canonical()))
            .collect()
    }

    #[test]
    fn titles_map_to_actions_and_ellipses_are_ignored() {
        let pairs = [
            ("Copy", "@~c"),
            ("Save As\u{2026}", "@$s"),
            ("Mystery Item", "@m"),
            ("Return to Previous Size", "^~r"),
            ("Cut", "@$"),
        ];
        let import = import_user_key_equivalents(&ENGLISH, pairs);
        assert_eq!(canonical(&import, StandardAction::Copy), ["Alt+Super+C"]);
        assert_eq!(
            canonical(&import, StandardAction::SaveAs),
            ["Shift+Super+S"]
        );
        assert_eq!(
            canonical(&import, StandardAction::ReturnToPreviousSize),
            ["Ctrl+Alt+R"]
        );
        assert_eq!(import.bindings.len(), 3);
        assert_eq!(import.problems.len(), 1);
        assert!(import.problems[0].contains("Cut"));
    }
}
