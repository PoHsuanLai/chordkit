use std::path::PathBuf;

use chordkit::{
    Action, Chord, Context, Desktop, KeyInput, Keymap, KeymapSource, Platform, StandardAction,
};
use chordkit_kde::{KdeSource, parse_kdeglobals};

const KDE: Platform = Platform::Linux {
    desktop: Desktop::Kde,
};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn press(keymap: &Keymap, text: &str) -> Option<Action> {
    let chord: Chord = text.parse().unwrap_or_else(|e| panic!("{text}: {e}"));
    keymap.resolve(
        &KeyInput::new(chord.modifiers(), chord.key()),
        Context::Normal,
    )
}

fn chords(import: &chordkit::SystemImport, action: StandardAction) -> Vec<String> {
    import
        .bindings
        .iter()
        .filter(|binding| binding.action == action)
        .flat_map(|binding| binding.chords.iter().map(|c| c.canonical()))
        .collect()
}

#[test]
fn shortcuts_group_maps_names_alternatives_and_none() {
    let text = std::fs::read_to_string(fixture("kdeglobals_changed")).unwrap_or_default();
    let import = parse_kdeglobals(&text);
    assert_eq!(
        chords(&import, StandardAction::Copy),
        ["Ctrl+Ins", "Ctrl+C"]
    );
    assert_eq!(chords(&import, StandardAction::Redo), ["Ctrl+Y"]);
    assert_eq!(chords(&import, StandardAction::Quit), ["Ctrl+Alt+Q"]);
    assert!(chords(&import, StandardAction::Undo).is_empty());
    assert!(
        import
            .bindings
            .iter()
            .any(|b| b.action == StandardAction::Undo)
    );
    assert_eq!(import.problems.len(), 1);
    assert!(
        import.problems[0].contains("Find"),
        "{}",
        import.problems[0]
    );
}

#[test]
fn load_lays_the_file_over_kdes_conventions() {
    let keymap = KdeSource::new(fixture("kdeglobals_changed"))
        .load(KDE)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        press(&keymap, "Ctrl+Insert"),
        Some(Action::Standard(StandardAction::Copy))
    );
    assert_eq!(
        press(&keymap, "Ctrl+Y"),
        Some(Action::Standard(StandardAction::Redo))
    );
    assert_eq!(press(&keymap, "Ctrl+Shift+Z"), None);
    assert_eq!(press(&keymap, "Ctrl+Z"), None);
    assert_eq!(
        press(&keymap, "Ctrl+S"),
        Some(Action::Standard(StandardAction::Save))
    );
}

#[test]
fn a_file_without_shortcuts_or_a_missing_file_is_the_conventions() {
    let plain = KdeSource::new(fixture("kdeglobals_plain")).load(KDE);
    let missing = KdeSource::new(fixture("does_not_exist")).load(KDE);
    for keymap in [plain, missing] {
        let keymap = keymap.unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(
            press(&keymap, "Ctrl+C"),
            Some(Action::Standard(StandardAction::Copy))
        );
    }
}

#[test]
fn an_unreadable_path_is_an_error_in_words() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let error = KdeSource::new(dir)
        .load(KDE)
        .err()
        .map(|e| e.to_string())
        .unwrap_or_default();
    assert!(
        error.starts_with("couldn't read the shortcut settings at "),
        "{error}"
    );
    assert!(error.ends_with("it is not a plain file"), "{error}");
}
