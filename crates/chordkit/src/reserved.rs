//! Chords a platform or desktop keeps for itself, which no app may take.

use crate::chord::Chord;
use crate::key::Key;
use crate::modifier::Modifier;
use crate::platform::{Desktop, Platform};

enum Rule {
    /// Exactly this chord (text `Chord` reads).
    Exact(&'static str),
    /// Every chord that uses the OS key.
    AnyWithSuper,
}

type ReservedRow = (Rule, &'static str);

use Rule::{AnyWithSuper, Exact};

const OURS: &[ReservedRow] = &[
    (
        Exact("Super"),
        "The desktop keeps the \u{2318} key pressed on its own for itself.",
    ),
    (
        Exact("Super+Space"),
        "The desktop opens the launcher with it.",
    ),
    (Exact("Super+Tab"), "The desktop switches apps with it."),
    (
        Exact("Shift+Super+Tab"),
        "The desktop switches apps backwards with it.",
    ),
    (
        Exact("Ctrl+Space"),
        "The desktop switches the input language with it.",
    ),
    (Exact("Ctrl+1"), "The desktop switches to Space 1 with it."),
    (Exact("Ctrl+2"), "The desktop switches to Space 2 with it."),
    (Exact("Ctrl+3"), "The desktop switches to Space 3 with it."),
    (Exact("Ctrl+4"), "The desktop switches to Space 4 with it."),
    (Exact("Ctrl+5"), "The desktop switches to Space 5 with it."),
    (Exact("Ctrl+6"), "The desktop switches to Space 6 with it."),
    (Exact("Ctrl+7"), "The desktop switches to Space 7 with it."),
    (Exact("Ctrl+8"), "The desktop switches to Space 8 with it."),
    (Exact("Ctrl+9"), "The desktop switches to Space 9 with it."),
];

const MAC: &[ReservedRow] = &[
    (Exact("Super+Space"), "macOS opens Spotlight with it."),
    (Exact("Super+Tab"), "macOS switches apps with it."),
    (
        Exact("Shift+Super+Tab"),
        "macOS switches apps backwards with it.",
    ),
    (
        Exact("Ctrl+Space"),
        "macOS switches the input source with it.",
    ),
];

const WINDOWS: &[ReservedRow] = &[
    (
        AnyWithSuper,
        "Windows keeps every shortcut with the Windows key for itself.",
    ),
    (Exact("Super"), "Windows opens the Start menu with it."),
    (Exact("Alt+Tab"), "Windows switches windows with it."),
    (
        Exact("Ctrl+Alt+Delete"),
        "Windows keeps it for the secure sign-in screen.",
    ),
    (
        Exact("Ctrl+Escape"),
        "Windows opens the Start menu with it.",
    ),
];

const KDE: &[ReservedRow] = &[
    (
        Exact("Super"),
        "Plasma opens the launcher with the Meta key on its own.",
    ),
    (Exact("Alt+Tab"), "Plasma switches windows with it."),
    (
        Exact("Ctrl+Alt+Delete"),
        "Plasma opens the logout screen with it.",
    ),
];

const GNOME: &[ReservedRow] = &[
    (
        Exact("Super"),
        "GNOME opens the overview with the Super key on its own.",
    ),
    (Exact("Alt+Tab"), "GNOME switches windows with it."),
    (Exact("Super+Tab"), "GNOME switches apps with it."),
    (
        Exact("Ctrl+Alt+Delete"),
        "GNOME opens the logout dialog with it.",
    ),
];

const OTHER: &[ReservedRow] = &[
    (Exact("Alt+Tab"), "The desktop switches windows with it."),
    (
        Exact("Ctrl+Alt+Delete"),
        "The desktop keeps it for logging out.",
    ),
];

fn rows(platform: Platform) -> &'static [ReservedRow] {
    match platform {
        Platform::MacOs => MAC,
        Platform::Windows => WINDOWS,
        Platform::Linux {
            desktop: Desktop::Kde,
        } => KDE,
        Platform::Linux {
            desktop: Desktop::Gnome,
        } => GNOME,
        Platform::Linux {
            desktop: Desktop::Ours,
        } => OURS,
        Platform::Linux {
            desktop: Desktop::Other,
        } => OTHER,
    }
}

fn uses_super(chord: Chord) -> bool {
    chord.modifiers().contains(Modifier::Super) || chord.key() == Key::Tap(Modifier::Super)
}

fn matches_rule(rule: &Rule, chord: Chord) -> bool {
    match rule {
        Rule::Exact(text) => text
            .parse::<Chord>()
            .is_ok_and(|reserved| reserved == chord),
        Rule::AnyWithSuper => uses_super(chord),
    }
}

/// Why `chord` is off limits to apps on `platform`, in plain words, or `None` if it is free.
pub fn reserved_reason(platform: Platform, chord: Chord) -> Option<&'static str> {
    rows(platform)
        .iter()
        .find(|(rule, _)| matches_rule(rule, chord))
        .map(|(_, reason)| *reason)
}

#[cfg(test)]
mod tests {
    use super::*;

    type Case = (Platform, &'static str, bool);

    #[test]
    fn every_reserved_row_reads() {
        for platform in Platform::ALL {
            for (rule, reason) in rows(platform) {
                if let Rule::Exact(text) = rule {
                    assert!(text.parse::<Chord>().is_ok(), "{platform:?}: {text}");
                }
                assert!(reason.ends_with('.'), "{platform:?}: {reason}");
            }
        }
    }

    #[test]
    fn reservations_per_platform() {
        let ours = Platform::Linux {
            desktop: Desktop::Ours,
        };
        let gnome = Platform::Linux {
            desktop: Desktop::Gnome,
        };
        let cases: &[Case] = &[
            (ours, "Super", true),
            (ours, "Super+Space", true),
            (ours, "Super+Tab", true),
            (ours, "Ctrl+Space", true),
            (ours, "Ctrl+1", true),
            (ours, "Ctrl+9", true),
            (ours, "Super+1", false),
            (ours, "Super+N", false),
            (Platform::Windows, "Super+N", true),
            (Platform::Windows, "Ctrl+1", false),
            (Platform::Windows, "Alt+Tab", true),
            (Platform::MacOs, "Super+1", false),
            (Platform::MacOs, "Super+Space", true),
            (gnome, "Super", true),
            (gnome, "Super+N", false),
        ];
        for (platform, text, reserved) in cases {
            let chord: Chord = text.parse().unwrap_or_else(|e| panic!("{text}: {e}"));
            assert_eq!(
                reserved_reason(*platform, chord).is_some(),
                *reserved,
                "{platform:?} {text}"
            );
        }
    }
}
