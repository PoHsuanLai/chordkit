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
    // Primary+Alt+= is Ctrl+Shift+Alt+= (read as Ctrl+Alt++) in a Windows terminal: plain Ctrl,
    // which the program keeps, so it cannot fire there and the fallback serves.
    let registration = Registration::new(&temor()).action_with_fallback(
        action("temor.equalize"),
        chord("Primary+Alt+="),
        chord("Primary+Alt+0"),
    );
    let mut keymap = Keymap::conventional(Platform::Windows);
    assert_eq!(keymap.register_with(&registration), Ok(()));
    assert_eq!(
        keymap.resolve(&input("Ctrl+Shift+Alt+0"), Context::Terminal),
        app("temor.equalize")
    );
}

/// Registers `rows` (id, default, fallback) for temor on other Linux, returning the keymap and
/// the result.
fn register_rows(rows: &[(&str, &str, Option<&str>)]) -> (Keymap, Result<(), Conflict>) {
    let mut registration = Registration::new(&temor());
    for (id, first, fallback) in rows {
        registration = match fallback {
            Some(fallback) => {
                registration.action_with_fallback(action(id), chord(first), chord(fallback))
            }
            None => registration.action(action(id), chord(first)),
        };
    }
    let mut keymap = Keymap::conventional(on(Desktop::Other));
    let result = keymap.register_with(&registration);
    (keymap, result)
}

type CollideCase = (
    &'static str,
    &'static str,
    &'static str,
    Context,
    &'static str,
    Option<Action>,
);

#[test]
fn a_colliding_default_gives_way_to_a_free_fallback() {
    // On other Linux, Ctrl+Alt+Left is Space left and Alt+Tab is reserved.
    let mine = app("temor.pane-left");
    let cases: &[CollideCase] = &[
        (
            "default free",
            "Primary+Alt+J",
            "Primary+Alt+Q",
            Context::Normal,
            "Ctrl+Alt+J",
            mine.clone(),
        ),
        (
            "default free: the fallback stays unbound",
            "Primary+Alt+J",
            "Primary+Alt+Q",
            Context::Normal,
            "Ctrl+Alt+Q",
            None,
        ),
        (
            "standard collision, fallback free",
            "Primary+Alt+Left",
            "Primary+Alt+Q",
            Context::Normal,
            "Ctrl+Alt+Q",
            mine.clone(),
        ),
        (
            "standard collision: the standard action keeps its chord",
            "Primary+Alt+Left",
            "Primary+Alt+Q",
            Context::Normal,
            "Ctrl+Alt+Left",
            standard(StandardAction::SpaceLeft),
        ),
        (
            "standard collision, in a terminal",
            "Primary+Alt+Left",
            "Primary+Alt+Q",
            Context::Terminal,
            "Ctrl+Shift+Alt+Q",
            mine.clone(),
        ),
        (
            "reserved collision",
            "Alt+Tab",
            "Primary+Alt+Q",
            Context::Normal,
            "Ctrl+Alt+Q",
            mine,
        ),
    ];
    for (name, first, fallback, context, pressed, expected) in cases {
        let (keymap, result) = register_rows(&[("temor.pane-left", first, Some(fallback))]);
        assert_eq!(result, Ok(()), "{name}");
        assert_eq!(
            keymap.resolve(&input(pressed), *context),
            *expected,
            "{name}"
        );
    }
}

#[test]
fn a_default_taken_by_another_app_falls_back() {
    let mut keymap = Keymap::conventional(on(Desktop::Other));
    let other = AppId::new("other").unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        keymap.register(&other, &[(action("other.go"), chord("Primary+Alt+J"))]),
        Ok(())
    );
    let registration = Registration::new(&temor()).action_with_fallback(
        action("temor.jump"),
        chord("Primary+Alt+J"),
        chord("Primary+Alt+K"),
    );
    assert_eq!(keymap.register_with(&registration), Ok(()));
    assert_eq!(
        keymap.resolve(&input("Ctrl+Alt+J"), Context::Normal),
        app("other.go")
    );
    assert_eq!(
        keymap.resolve(&input("Ctrl+Alt+K"), Context::Normal),
        app("temor.jump")
    );
}

#[test]
fn both_defaults_colliding_refuses_the_row_and_the_batch() {
    let (keymap, result) = register_rows(&[
        (
            "temor.pane-left",
            "Primary+Alt+Left",
            Some("Primary+Alt+Right"),
        ),
        ("temor.fine", "Primary+Alt+J", None),
    ]);
    assert!(
        matches!(
            result,
            Err(Conflict::Standard {
                action: StandardAction::SpaceLeft,
                ..
            })
        ),
        "{result:?}"
    );
    assert!(keymap.chords_of(&Action::App(pane_left())).is_empty());
    // Nothing from a refused batch registers.
    assert_eq!(keymap.resolve(&input("Ctrl+Alt+J"), Context::Normal), None);
}

#[test]
fn a_fallback_with_no_form_where_the_default_had_one_is_refused() {
    // In a terminal Primary is Ctrl+Shift on other Linux, so Primary+Alt+Shift+Left has no form
    // there while the default (Ctrl+Shift+Alt+Left) does: the fallback cannot cover it.
    let (keymap, result) = register_rows(&[(
        "temor.pane-left",
        "Primary+Alt+Left",
        Some("Primary+Alt+Shift+Left"),
    )]);
    assert!(
        matches!(result, Err(Conflict::Standard { .. })),
        "{result:?}"
    );
    assert!(keymap.chords_of(&Action::App(pane_left())).is_empty());
}

#[test]
fn a_fallback_colliding_with_a_staged_default_is_refused() {
    let rows = [
        ("temor.a", "Primary+Alt+Left", Some("Primary+Alt+J")),
        ("temor.b", "Primary+Alt+J", None),
    ];
    let (keymap, result) = register_rows(&rows);
    assert!(result.is_err(), "{result:?}");
    assert!(keymap.chords_of(&Action::App(action("temor.a"))).is_empty());
}

#[test]
fn fallbacks_never_take_a_later_rows_default_whatever_the_order() {
    let a = ("temor.a", "Primary+Alt+Left", Some("Primary+Alt+J"));
    let b = ("temor.b", "Primary+Alt+J", None);
    for rows in [[a, b], [b, a]] {
        let (keymap, result) = register_rows(&rows);
        assert!(result.is_err(), "{rows:?}");
        assert_eq!(keymap.resolve(&input("Ctrl+Alt+J"), Context::Normal), None);
    }
    // With a free fallback the same two rows register alike in both orders.
    let a = ("temor.a", "Primary+Alt+Left", Some("Primary+Alt+K"));
    let mut seen = Vec::new();
    for rows in [[a, b], [b, a]] {
        let (keymap, result) = register_rows(&rows);
        assert_eq!(result, Ok(()));
        seen.push((
            keymap.resolve(&input("Ctrl+Alt+K"), Context::Normal),
            keymap.resolve(&input("Ctrl+Alt+J"), Context::Normal),
        ));
    }
    assert_eq!(seen[0], seen[1]);
    assert_eq!(seen[0], (app("temor.a"), app("temor.b")));
}

#[test]
fn two_fallbacks_wanting_one_chord_go_to_the_first_declared() {
    let a = ("temor.a", "Primary+Alt+Left", Some("Primary+Alt+K"));
    let b = ("temor.b", "Primary+Alt+Right", Some("Primary+Alt+K"));
    let (keymap, result) = register_rows(&[a, b]);
    assert!(result.is_err(), "{result:?}");
    assert!(keymap.chords_of(&Action::App(action("temor.b"))).is_empty());
}
