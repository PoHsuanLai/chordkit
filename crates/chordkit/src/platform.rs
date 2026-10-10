//! Where the keymap runs, and what "Primary" means there.

use std::str::FromStr;

use thiserror::Error;

use crate::modifier::{Modifier, Modifiers};

/// The Linux desktop in use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Desktop {
    /// KDE Plasma.
    Kde,
    /// GNOME.
    Gnome,
    /// This project's desktop: Mac-style, with the Command key arriving as Super.
    Ours,
    /// Any other desktop; the common Ctrl-based conventions apply.
    Other,
}

/// An operating system and, on Linux, its desktop.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Platform {
    /// macOS.
    MacOs,
    /// Windows.
    Windows,
    /// Linux with a desktop.
    Linux {
        /// The desktop in use.
        desktop: Desktop,
    },
}

/// What has the keyboard focus, which decides which shortcuts may fire.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Context {
    /// Ordinary window content.
    Normal,
    /// A text field: typing keys belong to the field.
    TextEntry,
    /// A terminal: plain Ctrl and typing keys belong to the program.
    Terminal,
}

/// The text is not a platform name.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error(
    "\"{0}\" is not a platform I know (try macos, windows, linux-kde, linux-gnome, linux-ours or linux)"
)]
pub struct PlatformError(pub String);

impl Platform {
    /// Every platform there is a conventions table for.
    pub const ALL: [Platform; 6] = [
        Platform::MacOs,
        Platform::Windows,
        Platform::Linux {
            desktop: Desktop::Kde,
        },
        Platform::Linux {
            desktop: Desktop::Gnome,
        },
        Platform::Linux {
            desktop: Desktop::Ours,
        },
        Platform::Linux {
            desktop: Desktop::Other,
        },
    ];

    /// Whether the Command key is the primary modifier here (a Mac, or our desktop).
    pub const fn is_mac_style(self) -> bool {
        matches!(
            self,
            Platform::MacOs
                | Platform::Linux {
                    desktop: Desktop::Ours
                }
        )
    }

    /// The modifiers that `Primary` stands for in `context`: Command on Mac-style platforms,
    /// Ctrl elsewhere; in a terminal off the Mac style, Ctrl+Shift so Ctrl stays the program's.
    pub const fn primary(self, context: Context) -> Modifiers {
        match (self.is_mac_style(), context) {
            (true, _) => Modifiers::of(Modifier::Super),
            (false, Context::Terminal) => Modifiers::of(Modifier::Ctrl).with(Modifier::Shift),
            (false, _) => Modifiers::of(Modifier::Ctrl),
        }
    }

    /// The modifier a toolkit's Meta flag means on this platform. Toolkits disagree on Meta;
    /// the ones that report it at all (Qt) mean the OS key, so every platform folds it there.
    pub const fn meta_as(self) -> Modifier {
        Modifier::Super
    }

    /// A short stable name, the form `FromStr` reads back.
    pub const fn slug(self) -> &'static str {
        match self {
            Platform::MacOs => "macos",
            Platform::Windows => "windows",
            Platform::Linux {
                desktop: Desktop::Kde,
            } => "linux-kde",
            Platform::Linux {
                desktop: Desktop::Gnome,
            } => "linux-gnome",
            Platform::Linux {
                desktop: Desktop::Ours,
            } => "linux-ours",
            Platform::Linux {
                desktop: Desktop::Other,
            } => "linux",
        }
    }
}

impl FromStr for Platform {
    type Err = PlatformError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let wanted = text.trim().to_lowercase();
        Platform::ALL
            .into_iter()
            .find(|platform| platform.slug() == wanted)
            .ok_or_else(|| PlatformError(text.trim().to_owned()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type Case = (Platform, Context, Modifiers);

    fn mods(list: &[Modifier]) -> Modifiers {
        list.iter().copied().collect()
    }

    #[test]
    fn primary_per_platform_and_context() {
        let command = mods(&[Modifier::Super]);
        let ctrl = mods(&[Modifier::Ctrl]);
        let ctrl_shift = mods(&[Modifier::Ctrl, Modifier::Shift]);
        let ours = Platform::Linux {
            desktop: Desktop::Ours,
        };
        let kde = Platform::Linux {
            desktop: Desktop::Kde,
        };
        let cases: &[Case] = &[
            (Platform::MacOs, Context::Normal, command),
            (Platform::MacOs, Context::TextEntry, command),
            (Platform::MacOs, Context::Terminal, command),
            (ours, Context::Normal, command),
            (ours, Context::Terminal, command),
            (Platform::Windows, Context::Normal, ctrl),
            (Platform::Windows, Context::Terminal, ctrl_shift),
            (kde, Context::TextEntry, ctrl),
            (kde, Context::Terminal, ctrl_shift),
        ];
        for (platform, context, expected) in cases {
            assert_eq!(
                platform.primary(*context),
                *expected,
                "{platform:?} {context:?}"
            );
        }
    }

    #[test]
    fn slugs_round_trip() {
        for platform in Platform::ALL {
            assert_eq!(platform.slug().parse::<Platform>(), Ok(platform));
        }
        assert!("amiga".parse::<Platform>().is_err());
    }
}
