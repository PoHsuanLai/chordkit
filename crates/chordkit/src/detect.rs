//! Choosing the platform from the environment. The lookups are injected, so tests never read
//! the real environment and no machine is hardcoded.

use crate::platform::{Desktop, Platform};

/// The operating system the program was built for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostOs {
    /// macOS.
    MacOs,
    /// Windows.
    Windows,
    /// Linux.
    Linux,
    /// Anything else; treated as a Linux with an unknown desktop.
    Other,
}

impl HostOs {
    /// The operating system this build targets.
    pub const fn compiled() -> HostOs {
        if cfg!(target_os = "macos") {
            HostOs::MacOs
        } else if cfg!(target_os = "windows") {
            HostOs::Windows
        } else if cfg!(target_os = "linux") {
            HostOs::Linux
        } else {
            HostOs::Other
        }
    }
}

/// The variable that names the platform outright, for machines the guesses get wrong.
pub const OVERRIDE_VAR: &str = "CHORDKIT_PLATFORM";

/// The platform for `os`, reading variables through `var`. `CHORDKIT_PLATFORM` (a platform slug)
/// wins; otherwise Linux desktops come from `XDG_CURRENT_DESKTOP` (a colon-separated list),
/// then `DESKTOP_SESSION`, then `KDE_FULL_SESSION`.
pub fn detect(os: HostOs, var: impl Fn(&str) -> Option<String>) -> Platform {
    if let Some(platform) = var(OVERRIDE_VAR).and_then(|text| text.parse().ok()) {
        return platform;
    }
    match os {
        HostOs::MacOs => Platform::MacOs,
        HostOs::Windows => Platform::Windows,
        HostOs::Linux | HostOs::Other => Platform::Linux {
            desktop: linux_desktop(&var),
        },
    }
}

/// `detect` over this process's real environment: the one ambient read in the crate.
pub fn detect_process() -> Platform {
    detect(HostOs::compiled(), |name| std::env::var(name).ok())
}

fn linux_desktop(var: &impl Fn(&str) -> Option<String>) -> Desktop {
    let current = var("XDG_CURRENT_DESKTOP").unwrap_or_default();
    current
        .split(':')
        .find_map(|token| from_current_desktop(&token.to_lowercase()))
        .or_else(|| {
            var("DESKTOP_SESSION").and_then(|session| from_session(&session.to_lowercase()))
        })
        .or_else(|| var("KDE_FULL_SESSION").map(|_| Desktop::Kde))
        .unwrap_or(Desktop::Other)
}

fn from_current_desktop(token: &str) -> Option<Desktop> {
    match token {
        "kde" | "plasma" => Some(Desktop::Kde),
        "gnome" | "gnome-classic" => Some(Desktop::Gnome),
        "quire" | "sill" => Some(Desktop::Ours),
        _ => None,
    }
}

fn from_session(session: &str) -> Option<Desktop> {
    match session {
        s if s.starts_with("quire") || s.starts_with("sill") => Some(Desktop::Ours),
        s if s.starts_with("plasma") => Some(Desktop::Kde),
        s if s.starts_with("gnome") => Some(Desktop::Gnome),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type Vars = &'static [(&'static str, &'static str)];
    type Case = (&'static str, HostOs, Vars, Platform);

    fn linux(desktop: Desktop) -> Platform {
        Platform::Linux { desktop }
    }

    #[test]
    fn detection_table() {
        let cases: &[Case] = &[
            ("mac", HostOs::MacOs, &[], Platform::MacOs),
            ("windows", HostOs::Windows, &[], Platform::Windows),
            (
                "kde",
                HostOs::Linux,
                &[("XDG_CURRENT_DESKTOP", "KDE")],
                linux(Desktop::Kde),
            ),
            (
                "ubuntu gnome",
                HostOs::Linux,
                &[("XDG_CURRENT_DESKTOP", "ubuntu:GNOME")],
                linux(Desktop::Gnome),
            ),
            (
                "ours",
                HostOs::Linux,
                &[("XDG_CURRENT_DESKTOP", "quire")],
                linux(Desktop::Ours),
            ),
            (
                "session fallback",
                HostOs::Linux,
                &[("DESKTOP_SESSION", "plasmawayland")],
                linux(Desktop::Kde),
            ),
            (
                "kde full session",
                HostOs::Linux,
                &[("KDE_FULL_SESSION", "true")],
                linux(Desktop::Kde),
            ),
            (
                "unknown desktop",
                HostOs::Linux,
                &[("XDG_CURRENT_DESKTOP", "X-Cinnamon")],
                linux(Desktop::Other),
            ),
            ("no variables", HostOs::Linux, &[], linux(Desktop::Other)),
            ("other os", HostOs::Other, &[], linux(Desktop::Other)),
            (
                "override wins",
                HostOs::Linux,
                &[
                    ("CHORDKIT_PLATFORM", "windows"),
                    ("XDG_CURRENT_DESKTOP", "KDE"),
                ],
                Platform::Windows,
            ),
            (
                "bad override ignored",
                HostOs::Linux,
                &[
                    ("CHORDKIT_PLATFORM", "amiga"),
                    ("XDG_CURRENT_DESKTOP", "KDE"),
                ],
                linux(Desktop::Kde),
            ),
        ];
        for (name, os, vars, expected) in cases {
            let found = detect(*os, |key| {
                vars.iter()
                    .find(|(k, _)| *k == key)
                    .map(|(_, v)| (*v).to_owned())
            });
            assert_eq!(found, *expected, "{name}");
        }
    }
}
