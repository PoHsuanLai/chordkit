//! Forgone standard actions, fallback defaults and revived overrides.

use super::*;
use crate::action::AppId;
use crate::platform::Desktop;
use crate::reach::Reach;

fn on(desktop: Desktop) -> Platform {
    Platform::Linux { desktop }
}

fn input(text: &str) -> KeyInput {
    let chord: Chord = text.parse().unwrap_or_else(|e| panic!("{text}: {e}"));
    KeyInput::new(chord.modifiers(), chord.key())
}

fn chord(text: &str) -> DefaultChord {
    text.parse().unwrap_or_else(|e| panic!("{text}: {e}"))
}

fn action(id: &str) -> AppAction {
    AppAction::new(id).unwrap_or_else(|e| panic!("{e}"))
}

fn temor() -> AppId {
    AppId::new("temor").unwrap_or_else(|e| panic!("{e}"))
}

fn standard(action: StandardAction) -> Option<Action> {
    Some(Action::Standard(action))
}

fn app(id: &str) -> Option<Action> {
    Some(Action::App(action(id)))
}

type ForgoCase = (&'static str, Platform, &'static str);

const FONT_PLATFORMS: [ForgoCase; 2] = [
    ("macos", Platform::MacOs, "Super+T"),
    (
        "ours",
        Platform::Linux {
            desktop: Desktop::Ours,
        },
        "Super+T",
    ),
];

#[test]
fn an_app_without_fonts_takes_the_fonts_chord_and_others_keep_it() {
    for (name, platform, pressed) in FONT_PLATFORMS {
        let fonts = action("temor.tab");
        let mut plain = Keymap::conventional(platform);
        let refused = plain.register(&temor(), &[(fonts.clone(), chord("Primary+T"))]);
        assert!(
            matches!(refused, Err(Conflict::Standard { .. })),
            "{name}: {refused:?}"
        );

        let registration = Registration::new(&temor())
            .forgo(StandardAction::ShowFonts)
            .action(fonts, chord("Primary+T"));
        let mut keymap = Keymap::conventional(platform);
        assert_eq!(keymap.register_with(&registration), Ok(()), "{name}");
        assert_eq!(
            keymap.resolve(&input(pressed), Context::Normal),
            app("temor.tab"),
            "{name}"
        );
        assert!(
            keymap
                .chords_of(&Action::Standard(StandardAction::ShowFonts))
                .is_empty(),
            "{name}"
        );

        let other = Keymap::conventional(platform);
        assert_eq!(
            other.resolve(&input(pressed), Context::Normal),
            standard(StandardAction::ShowFonts),
            "{name}"
        );
    }
}

#[test]
fn a_forgone_action_stays_gone_when_the_system_or_person_binds_it() {
    let platform = Platform::MacOs;
    let registration = Registration::new(&temor()).forgo(StandardAction::ShowFonts);
    let mut keymap = Keymap::conventional(platform);
    assert_eq!(keymap.register_with(&registration), Ok(()));
    let (overrides, _) = Overrides::parse("show-fonts = Primary+Alt+T\n");
    let (keymap, _) = keymap.with_overrides(&overrides);
    // ⌥⌘T is Show Toolbar on a Mac; what matters is that Show Fonts is bound nowhere.
    assert_ne!(
        keymap.resolve(&input("Super+Alt+T"), Context::Normal),
        standard(StandardAction::ShowFonts)
    );
    assert!(
        keymap
            .chords_of(&Action::Standard(StandardAction::ShowFonts))
            .is_empty()
    );
}

#[test]
fn universal_actions_cannot_be_forgone_and_nothing_is_registered() {
    let universal = [
        StandardAction::Copy,
        StandardAction::Quit,
        StandardAction::Find,
    ];
    for gone in universal {
        let registration = Registration::new(&temor())
            .forgo(StandardAction::ShowFonts)
            .forgo(gone)
            .action(action("temor.tab"), chord("Primary+T"));
        let mut keymap = Keymap::conventional(Platform::MacOs);
        let result = keymap.register_with(&registration);
        assert!(
            matches!(&result, Err(Conflict::Universal { action, .. }) if *action == gone),
            "{gone:?}: {result:?}"
        );
        assert_eq!(
            keymap.resolve(&input("Super+T"), Context::Normal),
            standard(StandardAction::ShowFonts)
        );
        assert_eq!(gone.reach(), Reach::Universal);
    }
    let words = Conflict::Universal {
        action: StandardAction::Copy,
        meaning: StandardAction::Copy.label(),
        app: "temor".into(),
    };
    assert_eq!(
        words.to_string(),
        "\"copy\" works in every app, so temor can't give it up"
    );
}

type FallbackCase = (
    &'static str,
    Platform,
    Context,
    &'static str,
    Option<&'static str>,
);

#[test]
fn a_fallback_serves_a_context_where_the_default_has_no_form() {
    let registration = Registration::new(&temor()).action_with_fallback(
        action("temor.equalize"),
        chord("Primary+Shift+Alt+E"),
        chord("Primary+Alt+Q"),
    );
    let cases: &[FallbackCase] = &[
        (
            "windows terminal falls back",
            Platform::Windows,
            Context::Terminal,
            "Ctrl+Shift+Alt+Q",
            Some("temor.equalize"),
        ),
        (
            "windows terminal has no default form",
            Platform::Windows,
            Context::Terminal,
            "Ctrl+Shift+Alt+E",
            None,
        ),
        (
            "windows window keeps the default",
            Platform::Windows,
            Context::Normal,
            "Ctrl+Shift+Alt+E",
            Some("temor.equalize"),
        ),
        (
            "ours keeps the first in a terminal",
            on(Desktop::Ours),
            Context::Terminal,
            "Super+Shift+Alt+E",
            Some("temor.equalize"),
        ),
        (
            "ours does not use the fallback",
            on(Desktop::Ours),
            Context::Terminal,
            "Super+Alt+Q",
            None,
        ),
    ];
    for (name, platform, context, pressed, expected) in cases {
        let mut keymap = Keymap::conventional(*platform);
        assert_eq!(keymap.register_with(&registration), Ok(()), "{name}");
        assert_eq!(
            keymap.resolve(&input(pressed), *context),
            expected.and_then(app),
            "{name}"
        );
    }
}

#[test]
fn a_fallback_is_checked_where_it_is_live() {
    let registration = Registration::new(&temor()).action_with_fallback(
        action("temor.equalize"),
        chord("Primary+Shift+Alt+E"),
        chord("Primary+C"),
    );
    let mut keymap = Keymap::conventional(Platform::Windows);
    let result = keymap.register_with(&registration);
    assert!(
        matches!(
            result,
            Err(Conflict::Standard {
                action: StandardAction::Copy,
                ..
            })
        ),
        "{result:?}"
    );
}

fn pane_left() -> AppAction {
    action("temor.pane-left")
}

fn refused_pane_keymap(overrides: &str, register_first: bool) -> (Keymap, Vec<OverrideProblem>) {
    let (overrides, _) = Overrides::parse(overrides);
    let rows = [(pane_left(), chord("Primary+Alt+Left"))];
    let mut keymap = Keymap::conventional(on(Desktop::Other));
    if register_first {
        assert!(keymap.register(&temor(), &rows).is_err());
        return keymap.with_overrides(&overrides);
    }
    let (mut keymap, problems) = keymap.with_overrides(&overrides);
    assert!(keymap.register(&temor(), &rows).is_err());
    (keymap, problems)
}

#[test]
fn a_person_revives_a_refused_app_action() {
    for register_first in [true, false] {
        let (keymap, problems) =
            refused_pane_keymap("temor.pane-left = Ctrl+Alt+Shift+Left\n", register_first);
        assert!(problems.is_empty());
        assert_eq!(
            keymap.resolve(&input("Ctrl+Alt+Shift+Left"), Context::Normal),
            app("temor.pane-left"),
            "register first: {register_first}"
        );
        assert_eq!(
            keymap.resolve(&input("Ctrl+Alt+Left"), Context::Normal),
            standard(StandardAction::SpaceLeft)
        );
        assert!(keymap.dormant_overrides().is_empty());
    }
}

#[test]
fn a_refused_action_without_a_change_has_no_chord() {
    let (keymap, _) = refused_pane_keymap("", true);
    assert!(keymap.chords_of(&Action::App(pane_left())).is_empty());
    assert_eq!(
        keymap.resolve(&input("Ctrl+Alt+Left"), Context::Normal),
        standard(StandardAction::SpaceLeft)
    );
}

#[test]
fn the_person_may_take_a_standard_chord_but_not_a_reserved_one() {
    let (taken, problems) = refused_pane_keymap("temor.pane-left = Ctrl+Alt+Left\n", true);
    assert!(problems.is_empty());
    assert_eq!(
        taken.resolve(&input("Ctrl+Alt+Left"), Context::Normal),
        app("temor.pane-left")
    );
    let (kept, problems) = refused_pane_keymap("temor.pane-left = Alt+Tab\n", true);
    assert_eq!(problems.len(), 1);
    // The refused override leaves Alt+Tab with its standard meaning.
    assert_eq!(
        kept.resolve(&input("Alt+Tab"), Context::Normal),
        standard(StandardAction::NextApp)
    );
    assert!(kept.chords_of(&Action::App(pane_left())).is_empty());
}

#[test]
fn changes_to_actions_nobody_registered_stay_dormant() {
    let (overrides, _) = Overrides::parse("temor.ghost = Ctrl+Alt+Shift+Left\n");
    let (keymap, _) = Keymap::conventional(on(Desktop::Other)).with_overrides(&overrides);
    assert_eq!(keymap.dormant_overrides(), [action("temor.ghost")]);
    assert_eq!(
        keymap.resolve(&input("Ctrl+Alt+Shift+Left"), Context::Normal),
        None
    );
}

#[test]
fn a_default_that_cannot_fire_in_a_terminal_falls_back() {
    // Primary+Ctrl+= is Ctrl+Shift+= (read as Ctrl++) in a Windows terminal: plain Ctrl, which
    // the program keeps, so it cannot fire there and the fallback serves.
    let registration = Registration::new(&temor()).action_with_fallback(
        action("temor.equalize"),
        chord("Primary+Ctrl+="),
        chord("Primary+Alt+0"),
    );
    let mut keymap = Keymap::conventional(Platform::Windows);
    assert_eq!(keymap.register_with(&registration), Ok(()));
    assert_eq!(
        keymap.resolve(&input("Ctrl+Shift+Alt+0"), Context::Terminal),
        app("temor.equalize")
    );
}
