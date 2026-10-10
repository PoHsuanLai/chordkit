//! The terminal rules: the Primary alias on our desktop, the Primary form, the Shift stand-in
//! and AltGr text.

use super::*;
use crate::action::AppId;
use crate::key::Key;
use crate::key_input::AltGr;
use crate::modifier::{Modifier, Modifiers};
use crate::platform::Desktop;

fn on(desktop: Desktop) -> Platform {
    Platform::Linux { desktop }
}

fn input(text: &str) -> KeyInput {
    let chord: Chord = text.parse().unwrap_or_else(|e| panic!("{text}: {e}"));
    KeyInput::new(chord.modifiers(), chord.key())
}

fn mods(list: &[Modifier]) -> Modifiers {
    list.iter().copied().collect()
}

fn with_app_action(platform: Platform, id: &str, chord: &str) -> (Keymap, Action) {
    let app = AppId::new("term").unwrap_or_else(|e| panic!("{e}"));
    let action = AppAction::new(id).unwrap_or_else(|e| panic!("{e}"));
    let default: DefaultChord = chord.parse().unwrap_or_else(|e| panic!("{chord}: {e}"));
    let mut keymap = Keymap::conventional(platform);
    assert_eq!(keymap.register(&app, &[(action.clone(), default)]), Ok(()));
    (keymap, Action::App(action))
}

#[test]
fn our_terminal_takes_super_and_ctrl_shift_as_one_primary() {
    let (keymap, action) = with_app_action(on(Desktop::Ours), "term.dup", "Primary+D");
    for text in ["Super+D", "Ctrl+Shift+D"] {
        let found = keymap.resolve(&input(text), Context::Terminal);
        assert_eq!(found, Some(action.clone()), "{text}");
    }
    let shown: Vec<String> = keymap
        .chords_in(&action, Context::Terminal)
        .iter()
        .map(|chord| chord.display(keymap.platform()))
        .collect();
    assert_eq!(shown, ["⌘D"]);
    assert_eq!(keymap.resolve(&input("Ctrl+D"), Context::Terminal), None);
    assert_eq!(
        keymap.resolve(&input("Ctrl+Shift+D"), Context::Normal),
        None
    );
}

type FormCase = (&'static str, Platform, Context, &'static str, bool);

#[test]
fn the_primary_form_holds_whether_or_not_anything_is_bound() {
    let ours = on(Desktop::Ours);
    let kde = on(Desktop::Kde);
    let cases: &[FormCase] = &[
        ("ours super", ours, Context::Terminal, "Super+K", true),
        (
            "ours ctrl+shift",
            ours,
            Context::Terminal,
            "Ctrl+Shift+K",
            true,
        ),
        ("ours plain ctrl", ours, Context::Terminal, "Ctrl+D", false),
        ("ours digit", ours, Context::Terminal, "Super+7", true),
        ("ours symbol", ours, Context::Terminal, "Super+,", true),
        (
            "ours super+space",
            ours,
            Context::Terminal,
            "Super+Space",
            false,
        ),
        ("ours bound copy", ours, Context::Terminal, "Super+C", true),
        (
            "ours normal ctrl+shift",
            ours,
            Context::Normal,
            "Ctrl+Shift+K",
            false,
        ),
        ("ours normal super", ours, Context::Normal, "Super+K", true),
        (
            "mac super",
            Platform::MacOs,
            Context::Terminal,
            "Super+K",
            true,
        ),
        (
            "mac ctrl+shift",
            Platform::MacOs,
            Context::Terminal,
            "Ctrl+Shift+K",
            false,
        ),
        (
            "windows ctrl+shift",
            Platform::Windows,
            Context::Terminal,
            "Ctrl+Shift+K",
            true,
        ),
        (
            "windows plain ctrl",
            Platform::Windows,
            Context::Terminal,
            "Ctrl+D",
            false,
        ),
        (
            "windows super",
            Platform::Windows,
            Context::Terminal,
            "Super+K",
            false,
        ),
        (
            "kde ctrl+shift",
            kde,
            Context::Terminal,
            "Ctrl+Shift+K",
            true,
        ),
        (
            "kde stand-in",
            kde,
            Context::Terminal,
            "Ctrl+Shift+Alt+K",
            true,
        ),
        ("kde tab", kde, Context::Terminal, "Ctrl+Shift+Tab", false),
        ("kde normal ctrl", kde, Context::Normal, "Ctrl+K", true),
        ("kde normal plain", kde, Context::Normal, "K", false),
        ("kde field ctrl", kde, Context::TextEntry, "Ctrl+K", true),
        (
            "other ctrl+shift",
            on(Desktop::Other),
            Context::Terminal,
            "Ctrl+Shift+K",
            true,
        ),
        (
            "gnome ctrl+shift",
            on(Desktop::Gnome),
            Context::Terminal,
            "Ctrl+Shift+K",
            true,
        ),
    ];
    for (name, platform, context, text, expected) in cases {
        let keymap = Keymap::conventional(*platform);
        let found = keymap.is_primary_form(&input(text), *context);
        assert_eq!(found, *expected, "{name}");
    }
}

#[test]
fn primary_shift_chords_stand_in_as_ctrl_shift_alt_in_terminals() {
    for platform in [Platform::Windows, on(Desktop::Kde)] {
        let (keymap, action) = with_app_action(platform, "term.split", "Primary+Shift+D");
        let found = keymap.resolve(&input("Ctrl+Shift+Alt+D"), Context::Terminal);
        assert_eq!(found, Some(action.clone()), "{platform:?}");
        assert_eq!(
            keymap.resolve(&input("Ctrl+Shift+D"), Context::Terminal),
            None
        );
        assert_eq!(
            keymap.resolve(&input("Ctrl+Shift+D"), Context::Normal),
            Some(action)
        );
    }
    let kde = Keymap::conventional(on(Desktop::Kde));
    let redo = kde.resolve(&input("Ctrl+Shift+Alt+Z"), Context::Terminal);
    assert_eq!(redo, Some(Action::Standard(StandardAction::Redo)));
    let windows = Keymap::conventional(Platform::Windows);
    let save_as = windows.resolve(&input("Ctrl+Shift+Alt+S"), Context::Terminal);
    assert_eq!(save_as, Some(Action::Standard(StandardAction::SaveAs)));
}

#[test]
fn alt_gr_text_never_matches_a_ctrl_alt_chord() {
    let (keymap, action) = with_app_action(Platform::Windows, "term.at", "Ctrl+Alt+@");
    let ctrl_alt = mods(&[Modifier::Ctrl, Modifier::Alt]);
    let at = KeyInput::new(ctrl_alt, Key::Char('@'));
    assert_eq!(keymap.resolve(&at, Context::Normal), Some(action));
    let typed = at.with_alt_gr(AltGr::Held);
    assert_eq!(keymap.resolve(&typed, Context::Normal), None);
    assert_eq!(keymap.resolve(&typed, Context::Terminal), None);
    assert!(!keymap.is_primary_form(&typed, Context::Terminal));
    let ctrl_shift_alt = KeyInput::new(ctrl_alt.with(Modifier::Shift), Key::Char('@'));
    assert!(keymap.is_primary_form(&ctrl_shift_alt, Context::Terminal));
    assert!(!keymap.is_primary_form(&ctrl_shift_alt.with_alt_gr(AltGr::Held), Context::Terminal));
}

#[test]
fn ctrl_alt_without_alt_gr_still_matches_a_ctrl_alt_chord() {
    let (keymap, action) = with_app_action(Platform::Windows, "term.new-tab", "Ctrl+Alt+T");
    assert_eq!(
        keymap.resolve(&input("Ctrl+Alt+T"), Context::Normal),
        Some(action)
    );
}
