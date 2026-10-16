use std::path::PathBuf;

use chordkit::{
    Action, Chord, Context, Desktop, KeyInput, Keymap, KeymapSource, Platform, SpaceNumber,
    StandardAction,
};
use chordkit_gnome::{GnomeSource, parse_dconf_dump};

const GNOME: Platform = Platform::Linux {
    desktop: Desktop::Gnome,
};

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/dconf_dump")
}

fn press(keymap: &Keymap, text: &str) -> Option<Action> {
    let chord: Chord = text.parse().unwrap_or_else(|e| panic!("{text}: {e}"));
    keymap.resolve(
        &KeyInput::new(chord.modifiers(), chord.key()),
        Context::Normal,
    )
}

fn standard(action: StandardAction) -> Option<Action> {
    Some(Action::Standard(action))
}

#[test]
fn dump_maps_window_manager_media_and_mutter_keys() {
    let text = std::fs::read_to_string(fixture()).unwrap_or_default();
    let import = parse_dconf_dump(&text);
    assert_eq!(import.problems.len(), 1, "{:?}", import.problems);
    assert!(import.problems[0].contains("switch-to-workspace-left"));
    let minimize = import
        .bindings
        .iter()
        .find(|b| b.action == StandardAction::Minimize);
    assert_eq!(minimize.map(|b| b.chords.len()), Some(2));
    let unbound: Vec<StandardAction> = import
        .bindings
        .iter()
        .filter(|b| b.chords.is_empty())
        .map(|b| b.action)
        .collect();
    assert_eq!(
        unbound,
        [StandardAction::NextWindow, StandardAction::LogOut]
    );
}

#[test]
fn load_lays_the_dump_over_gnomes_conventions() {
    let keymap = GnomeSource::new(fixture())
        .load(GNOME)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        press(&keymap, "Super+H"),
        standard(StandardAction::Minimize)
    );
    assert_eq!(press(&keymap, "Alt+Tab"), standard(StandardAction::NextApp));
    assert_eq!(press(&keymap, "Super+Tab"), None);
    assert_eq!(
        press(&keymap, "Alt+Super+Left"),
        standard(StandardAction::TileLeft)
    );
    assert_eq!(press(&keymap, "Super+Left"), None);
    assert_eq!(
        press(&keymap, "Super+L"),
        standard(StandardAction::LockScreen)
    );
    assert_eq!(
        press(&keymap, "Super"),
        standard(StandardAction::MissionControl)
    );
    let space3 = SpaceNumber::new(3).map(StandardAction::SwitchToSpace);
    assert_eq!(press(&keymap, "Super+3"), space3.map(Action::Standard));
    // Untouched keys stay conventional, and the unreadable one leaves its action alone.
    assert_eq!(press(&keymap, "Ctrl+C"), standard(StandardAction::Copy));
    assert_eq!(
        press(&keymap, "Super+PageUp"),
        standard(StandardAction::SpaceLeft)
    );
}

#[test]
fn a_missing_dump_is_the_conventions() {
    let missing = fixture().with_file_name("nothing_here");
    let keymap = GnomeSource::new(missing)
        .load(GNOME)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        press(&keymap, "Super+Tab"),
        standard(StandardAction::NextApp)
    );
}

#[test]
fn a_folder_instead_of_a_dump_is_an_error_in_words() {
    let folder = fixture().parent().map(PathBuf::from).unwrap_or_default();
    let error = GnomeSource::new(folder)
        .load(GNOME)
        .err()
        .map(|e| e.to_string())
        .unwrap_or_default();
    assert!(
        error.starts_with("couldn't read the shortcut settings at "),
        "{error}"
    );
    assert!(error.ends_with("it is not a plain file"), "{error}");
}
