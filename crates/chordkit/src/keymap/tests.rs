use super::*;
use crate::action::AppId;
use crate::key::Key;
use crate::modifier::{Modifier, Modifiers};
use crate::platform::Desktop;

fn ours() -> Platform {
    Platform::Linux {
        desktop: Desktop::Ours,
    }
}

fn kde() -> Platform {
    Platform::Linux {
        desktop: Desktop::Kde,
    }
}

fn input(text: &str) -> KeyInput {
    let chord: Chord = text.parse().unwrap_or_else(|e| panic!("{text}: {e}"));
    KeyInput::new(chord.modifiers(), chord.key())
}

fn default_chord(text: &str) -> DefaultChord {
    text.parse().unwrap_or_else(|e| panic!("{text}: {e}"))
}

fn app_action(id: &str) -> AppAction {
    AppAction::new(id).unwrap_or_else(|e| panic!("{e}"))
}

fn app(name: &str) -> AppId {
    AppId::new(name).unwrap_or_else(|e| panic!("{e}"))
}

fn std_action(action: StandardAction) -> Option<Action> {
    Some(Action::Standard(action))
}

type Case = (
    &'static str,
    Platform,
    Context,
    &'static str,
    Option<StandardAction>,
);

#[test]
fn resolution_follows_platform_and_context() {
    use StandardAction as S;
    let cases: &[Case] = &[
        (
            "windows copy",
            Platform::Windows,
            Context::Normal,
            "Ctrl+C",
            Some(S::Copy),
        ),
        (
            "windows copy in a field",
            Platform::Windows,
            Context::TextEntry,
            "Ctrl+C",
            Some(S::Copy),
        ),
        (
            "windows ctrl+c stays the program's",
            Platform::Windows,
            Context::Terminal,
            "Ctrl+C",
            None,
        ),
        (
            "windows terminal copy",
            Platform::Windows,
            Context::Terminal,
            "Ctrl+Shift+C",
            Some(S::Copy),
        ),
        (
            "windows terminal paste",
            Platform::Windows,
            Context::Terminal,
            "Ctrl+Shift+V",
            Some(S::Paste),
        ),
        (
            "mac copy",
            Platform::MacOs,
            Context::Normal,
            "Super+C",
            Some(S::Copy),
        ),
        (
            "mac ctrl+c is not copy",
            Platform::MacOs,
            Context::Normal,
            "Ctrl+C",
            None,
        ),
        (
            "mac terminal copy",
            Platform::MacOs,
            Context::Terminal,
            "Super+C",
            Some(S::Copy),
        ),
        (
            "ours terminal copy",
            ours(),
            Context::Terminal,
            "Super+C",
            Some(S::Copy),
        ),
        (
            "ours terminal ctrl+space",
            ours(),
            Context::Terminal,
            "Ctrl+Space",
            None,
        ),
        (
            "ours ctrl+space",
            ours(),
            Context::Normal,
            "Ctrl+Space",
            Some(S::NextInputSource),
        ),
        (
            "kde terminal paste",
            kde(),
            Context::Terminal,
            "Ctrl+Shift+V",
            Some(S::Paste),
        ),
        (
            "kde redo",
            kde(),
            Context::Normal,
            "Ctrl+Shift+Z",
            Some(S::Redo),
        ),
        (
            "windows redo",
            Platform::Windows,
            Context::Normal,
            "Ctrl+Y",
            Some(S::Redo),
        ),
        (
            "mac bigger on the equals key",
            Platform::MacOs,
            Context::Normal,
            "Super+=",
            Some(S::Bigger),
        ),
        (
            "mac plus via shift equals",
            Platform::MacOs,
            Context::Normal,
            "Super+Shift+=",
            Some(S::Bigger),
        ),
        (
            "ctrl plus via shift equals in a terminal",
            Platform::Windows,
            Context::Terminal,
            "Ctrl+Shift+=",
            Some(S::Bigger),
        ),
        (
            "the text size keys are the window's in a terminal",
            Platform::Windows,
            Context::Terminal,
            "Ctrl+=",
            Some(S::Bigger),
        ),
        (
            "smaller in a terminal",
            Platform::Linux {
                desktop: Desktop::Gnome,
            },
            Context::Terminal,
            "Ctrl+-",
            Some(S::Smaller),
        ),
        (
            "ctrl underscore stays the program's",
            Platform::Linux {
                desktop: Desktop::Gnome,
            },
            Context::Terminal,
            "Ctrl+Shift+-",
            None,
        ),
    ];
    for (name, platform, context, text, expected) in cases {
        let keymap = Keymap::conventional(*platform);
        let found = keymap.resolve(&input(text), *context);
        assert_eq!(found, expected.map(Action::Standard), "{name}");
    }
}

#[test]
fn meta_flag_resolves_like_super() {
    let keymap = Keymap::conventional(Platform::MacOs);
    let meta_c = KeyInput::new(Modifiers::of(Modifier::Meta), Key::Char('c'));
    assert_eq!(
        keymap.resolve(&meta_c, Context::Normal),
        std_action(StandardAction::Copy)
    );
}

#[test]
fn chords_of_shows_the_platforms_chord() {
    let keymap = Keymap::conventional(Platform::Windows);
    let redo = Action::Standard(StandardAction::Redo);
    let shown: Vec<String> = keymap
        .chords_of(&redo)
        .iter()
        .map(|c| c.display(Platform::Windows))
        .collect();
    assert_eq!(shown, ["Ctrl+Y"]);
}

#[test]
fn text_fields_keep_typing_keys_from_app_actions() {
    let mut keymap = Keymap::conventional(ours());
    let next = app_action("reader.next-page");
    assert_eq!(
        keymap.register(&app("reader"), &[(next.clone(), default_chord("J"))]),
        Ok(())
    );
    let j = input("J");
    assert_eq!(keymap.resolve(&j, Context::Normal), Some(Action::App(next)));
    assert_eq!(keymap.resolve(&j, Context::TextEntry), None);
    assert_eq!(keymap.resolve(&j, Context::Terminal), None);
}

type Refusal = (&'static str, Platform, &'static str, fn(&Conflict) -> bool);

#[test]
fn registration_refuses_reserved_standard_and_taken_chords() {
    let cases: &[Refusal] = &[
        ("ours super+space", ours(), "Super+Space", |c| {
            matches!(c, Conflict::Reserved { .. })
        }),
        ("ours bare super", ours(), "Super", |c| {
            matches!(c, Conflict::Reserved { .. })
        }),
        ("ours ctrl+3", ours(), "Ctrl+3", |c| {
            matches!(c, Conflict::Reserved { .. })
        }),
        (
            "ours ctrl+space via primary-free",
            ours(),
            "Ctrl+Space",
            |c| matches!(c, Conflict::Reserved { .. }),
        ),
        ("windows win+n", Platform::Windows, "Super+N", |c| {
            matches!(c, Conflict::Reserved { .. })
        }),
        ("windows primary+c", Platform::Windows, "Primary+C", |c| {
            matches!(
                c,
                Conflict::Standard {
                    action: StandardAction::Copy,
                    ..
                }
            )
        }),
        ("ours primary+s", ours(), "Primary+S", |c| {
            matches!(
                c,
                Conflict::Standard {
                    action: StandardAction::Save,
                    ..
                }
            )
        }),
    ];
    for (name, platform, chord, expected) in cases {
        let mut keymap = Keymap::conventional(*platform);
        let before = keymap.bindings(Context::Normal).len();
        let result = keymap.register(
            &app("mail"),
            &[(app_action("mail.go"), default_chord(chord))],
        );
        match result {
            Err(conflict) => assert!(expected(&conflict), "{name}: {conflict}"),
            Ok(()) => panic!("{name}: registration was allowed"),
        }
        assert_eq!(
            keymap.bindings(Context::Normal).len(),
            before,
            "{name}: nothing registered"
        );
    }
}

#[test]
fn apps_own_primary_digits_and_the_refusal_reads_in_words() {
    let mut keymap = Keymap::conventional(ours());
    let tab3 = app_action("mail.tab-3");
    assert_eq!(
        keymap.register(&app("mail"), &[(tab3.clone(), default_chord("Primary+3"))]),
        Ok(())
    );
    assert_eq!(
        keymap.resolve(&input("Super+3"), Context::Normal),
        Some(Action::App(tab3))
    );

    let refused = keymap.register(
        &app("mail"),
        &[(app_action("mail.jump"), default_chord("Ctrl+Space"))],
    );
    let words = refused.err().map(|c| c.to_string()).unwrap_or_default();
    assert_eq!(
        words,
        "\u{2303}Space can't be used for mail.jump: The desktop switches the input language with it."
    );
}

#[test]
fn two_apps_cannot_share_a_chord_and_foreign_ids_are_refused() {
    let mut keymap = Keymap::conventional(Platform::Windows);
    let first = (app_action("mail.compose"), default_chord("Primary+Alt+N"));
    assert_eq!(keymap.register(&app("mail"), &[first]), Ok(()));
    let second = (app_action("notes.capture"), default_chord("Primary+Alt+N"));
    assert!(matches!(
        keymap.register(&app("notes"), &[second]),
        Err(Conflict::Taken { held_by, .. }) if held_by == "mail.compose"
    ));
    let foreign = (app_action("notes.capture"), default_chord("Primary+Alt+M"));
    assert!(matches!(
        keymap.register(&app("mail"), &[foreign]),
        Err(Conflict::ForeignAction { .. })
    ));
}

#[test]
fn a_refused_row_registers_nothing_from_its_batch() {
    let mut keymap = Keymap::conventional(Platform::Windows);
    let rows = [
        (app_action("mail.a"), default_chord("Primary+Alt+A")),
        (app_action("mail.b"), default_chord("Primary+C")),
    ];
    assert!(keymap.register(&app("mail"), &rows).is_err());
    assert_eq!(keymap.resolve(&input("Ctrl+Alt+A"), Context::Normal), None);
}

#[test]
fn the_system_replaces_conventions_and_the_person_replaces_both() {
    let insert: Chord = "Ctrl+Insert".parse().unwrap_or_else(|e| panic!("{e}"));
    let system = vec![SystemBinding {
        action: StandardAction::Copy,
        chords: vec![insert],
    }];
    let keymap = Keymap::conventional(kde()).with_system(system);
    assert_eq!(keymap.resolve(&input("Ctrl+C"), Context::Normal), None);
    assert_eq!(
        keymap.resolve(&input("Ctrl+Insert"), Context::Normal),
        std_action(StandardAction::Copy)
    );

    let (overrides, problems) =
        Overrides::parse("copy = Primary+Alt+C\nundo = none\nsuper = Primary+K\n");
    assert_eq!(problems.len(), 1);
    let (keymap, refused) = keymap.with_overrides(&overrides);
    assert!(refused.is_empty());
    assert_eq!(keymap.resolve(&input("Ctrl+Insert"), Context::Normal), None);
    assert_eq!(
        keymap.resolve(&input("Ctrl+Alt+C"), Context::Normal),
        std_action(StandardAction::Copy)
    );
    assert_eq!(keymap.resolve(&input("Ctrl+Z"), Context::Normal), None);
    let defaults = keymap.default_chords_of(StandardAction::Copy);
    assert_eq!(defaults, [insert]);
}

#[test]
fn a_person_wins_a_chord_and_reserved_ones_are_refused_in_words() {
    let (overrides, _) = Overrides::parse("bold = Primary+S\nitalic = Super+Space\n");
    let (keymap, refused) = Keymap::conventional(ours()).with_overrides(&overrides);
    assert_eq!(
        keymap.resolve(&input("Super+S"), Context::Normal),
        std_action(StandardAction::Bold)
    );
    assert_eq!(
        keymap.resolve(&input("Super+I"), Context::Normal),
        std_action(StandardAction::Italic)
    );
    assert_eq!(refused.len(), 1);
    assert_eq!(refused[0].line, 2);
    assert!(refused[0].what.contains("can't be used"), "{}", refused[0]);
}

#[test]
fn overrides_of_unregistered_app_actions_wait_until_registered() {
    let (overrides, _) = Overrides::parse("mail.compose = Primary+Shift+M\n");
    let (mut keymap, _) = Keymap::conventional(Platform::Windows).with_overrides(&overrides);
    let compose = app_action("mail.compose");
    assert_eq!(keymap.dormant_overrides(), std::slice::from_ref(&compose));
    assert_eq!(
        keymap.resolve(&input("Ctrl+Shift+M"), Context::Normal),
        None
    );
    let registered = keymap.register(
        &app("mail"),
        &[(compose.clone(), default_chord("Primary+Alt+N"))],
    );
    assert_eq!(registered, Ok(()));
    assert!(keymap.dormant_overrides().is_empty());
    assert_eq!(
        keymap.resolve(&input("Ctrl+Shift+M"), Context::Normal),
        Some(Action::App(compose))
    );
    assert_eq!(keymap.resolve(&input("Ctrl+Alt+N"), Context::Normal), None);
}

/// Raw presses as a window reports them: the key with Shift not applied, Shift in the modifiers.
#[test]
fn shifted_minus_and_equals_in_a_terminal() {
    use StandardAction as S;
    let ctrl_shift = Modifiers::of(Modifier::Ctrl).with(Modifier::Shift);
    let gnome = Platform::Linux {
        desktop: Desktop::Gnome,
    };
    let ours = Platform::Linux {
        desktop: Desktop::Ours,
    };
    let cases: &[(&str, Platform, char, Option<S>, bool)] = &[
        // Ctrl+Shift+- is Ctrl+_, the program's undo: not bound and not in the Primary form.
        ("ctrl underscore off our desktop", gnome, '-', None, false),
        (
            "ctrl underscore on windows",
            Platform::Windows,
            '-',
            None,
            false,
        ),
        (
            "ctrl plus off our desktop",
            gnome,
            '=',
            Some(S::Bigger),
            false,
        ),
        // On our desktop keycap sends ⌘ as Ctrl+Shift: these are ⌘- and ⌘=.
        ("keycap's command minus", ours, '-', Some(S::Smaller), true),
        ("keycap's command equals", ours, '=', Some(S::Bigger), true),
    ];
    for (name, platform, key, expected, primary_form) in cases {
        let keymap = Keymap::conventional(*platform);
        let press = KeyInput::new(ctrl_shift, Key::Char(*key));
        assert_eq!(
            keymap.resolve(&press, Context::Terminal),
            expected.map(Action::Standard),
            "{name}"
        );
        assert_eq!(
            keymap.is_primary_form(&press, Context::Terminal),
            *primary_form,
            "{name}"
        );
    }
}
