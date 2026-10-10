//! KStandardShortcut names and the standard action each one means.

use chordkit::StandardAction;

type Name = (&'static str, StandardAction);

/// The names kdeglobals uses in `[Shortcuts]`. Names not listed are left alone.
pub(crate) const NAMES: &[Name] = &[
    ("Open", StandardAction::Open),
    ("New", StandardAction::New),
    ("Close", StandardAction::Close),
    ("Save", StandardAction::Save),
    ("SaveAs", StandardAction::SaveAs),
    ("Print", StandardAction::Print),
    ("Quit", StandardAction::Quit),
    ("Undo", StandardAction::Undo),
    ("Redo", StandardAction::Redo),
    ("Cut", StandardAction::Cut),
    ("Copy", StandardAction::Copy),
    ("Paste", StandardAction::Paste),
    ("SelectAll", StandardAction::SelectAll),
    ("Find", StandardAction::Find),
    ("FindNext", StandardAction::FindNext),
    ("FindPrev", StandardAction::FindPrevious),
    ("Help", StandardAction::Help),
    ("ZoomIn", StandardAction::Bigger),
    ("ZoomOut", StandardAction::Smaller),
    ("FullScreen", StandardAction::FullScreen),
    ("Preferences", StandardAction::Settings),
];

pub(crate) fn action_named(name: &str) -> Option<StandardAction> {
    NAMES
        .iter()
        .find(|(known, _)| *known == name)
        .map(|(_, action)| *action)
}
