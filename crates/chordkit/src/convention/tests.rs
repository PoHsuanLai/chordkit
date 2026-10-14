use super::*;
use crate::chord::Chord;
use crate::platform::Context;

fn chord_on(platform: Platform, action: StandardAction) -> Option<Chord> {
    match ConventionTable::for_platform(platform).lookup(action)? {
        Convention::Bound(default) => default.resolve(platform, Context::Normal),
        Convention::Unbound(_) => None,
    }
}

#[test]
fn every_standard_action_has_a_default_or_a_reason_on_every_platform() {
    for platform in Platform::ALL {
        let table = ConventionTable::for_platform(platform);
        for action in StandardAction::all() {
            assert!(
                table.lookup(action).is_some(),
                "{platform:?} has no row, or an unreadable row, for {action:?}"
            );
        }
        assert_eq!(table.iter().count(), 67, "{platform:?}");
    }
}

#[test]
fn overrides_only_name_actions_that_exist_and_never_repeat() {
    for (name, rows) in [
        ("windows", windows_rows()),
        ("kde", kde_rows()),
        ("gnome", gnome_rows()),
    ] {
        let mut seen: Vec<StandardAction> = Vec::new();
        for (action, _) in rows {
            assert!(!seen.contains(action), "{name} repeats {action:?}");
            seen.push(*action);
        }
    }
}

fn windows_rows() -> &'static [TableRow] {
    super::windows::ROWS
}

fn kde_rows() -> &'static [TableRow] {
    super::kde::ROWS
}

fn gnome_rows() -> &'static [TableRow] {
    super::gnome::ROWS
}

type Case = (Platform, StandardAction, &'static str);

#[test]
fn conventions_resolve_to_the_expected_chords() {
    let ours = Platform::Linux {
        desktop: Desktop::Ours,
    };
    let kde = Platform::Linux {
        desktop: Desktop::Kde,
    };
    let gnome = Platform::Linux {
        desktop: Desktop::Gnome,
    };
    let other = Platform::Linux {
        desktop: Desktop::Other,
    };
    let cases: &[Case] = &[
        (Platform::MacOs, StandardAction::Copy, "Super+C"),
        (Platform::MacOs, StandardAction::Redo, "Shift+Super+Z"),
        (
            Platform::MacOs,
            StandardAction::PasteAndMatchStyle,
            "Alt+Shift+Super+V",
        ),
        (Platform::MacOs, StandardAction::Bigger, "Super++"),
        (Platform::Windows, StandardAction::Bigger, "Ctrl++"),
        (gnome, StandardAction::Smaller, "Ctrl+-"),
        (Platform::MacOs, StandardAction::ActualSize, "Super+0"),
        (Platform::Windows, StandardAction::ActualSize, "Ctrl+0"),
        (gnome, StandardAction::ActualSize, "Ctrl+0"),
        (ours, StandardAction::Copy, "Super+C"),
        (ours, StandardAction::LockScreen, "Ctrl+Super+Q"),
        (Platform::Windows, StandardAction::Copy, "Ctrl+C"),
        (Platform::Windows, StandardAction::Redo, "Ctrl+Y"),
        (Platform::Windows, StandardAction::Quit, "Alt+F4"),
        (kde, StandardAction::Redo, "Ctrl+Shift+Z"),
        (kde, StandardAction::LockScreen, "Ctrl+Alt+L"),
        (gnome, StandardAction::FindNext, "Ctrl+G"),
        (gnome, StandardAction::Quit, "Ctrl+Q"),
        (other, StandardAction::Save, "Ctrl+S"),
        (other, StandardAction::Help, "F1"),
    ];
    for (platform, action, expected) in cases {
        let wanted: Chord = expected
            .parse()
            .unwrap_or_else(|e| panic!("{expected}: {e}"));
        assert_eq!(
            chord_on(*platform, *action),
            Some(wanted),
            "{platform:?} {action:?}"
        );
    }
}

#[test]
fn space_switches_follow_the_platform() {
    let space3 = StandardAction::SwitchToSpace(SpaceNumber::new(3).unwrap_or_else(|| panic!("3")));
    let kde = Platform::Linux {
        desktop: Desktop::Kde,
    };
    let wanted = |text: &str| text.parse::<Chord>().ok();
    assert_eq!(chord_on(Platform::MacOs, space3), wanted("Ctrl+3"));
    assert_eq!(chord_on(kde, space3), wanted("Ctrl+F3"));
    assert_eq!(chord_on(Platform::Windows, space3), None);
    assert!(matches!(
        ConventionTable::for_platform(Platform::Windows).lookup(space3),
        Some(Convention::Unbound(_))
    ));
}

#[test]
fn bound_conventions_are_unique_within_a_platform_and_context() {
    for platform in Platform::ALL {
        for context in [Context::Normal] {
            let mut seen: Vec<(Chord, StandardAction)> = Vec::new();
            for (action, convention) in ConventionTable::for_platform(platform).iter() {
                let Convention::Bound(default) = convention else {
                    continue;
                };
                let Some(chord) = default.resolve(platform, context) else {
                    continue;
                };
                if let Some((_, other)) = seen.iter().find(|(c, _)| *c == chord) {
                    panic!("{platform:?}: {action:?} and {other:?} share {chord:?}");
                }
                seen.push((chord, action));
            }
        }
    }
}
