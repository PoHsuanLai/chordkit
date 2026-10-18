//! What a toolkit hands in for a key press, and its normalisation to a chord.

use crate::chord::Chord;
use crate::key::Key;
use crate::modifier::{Modifier, Modifiers};
use crate::platform::{Context, Platform};

/// Whether the AltGr key (ISO level 3 shift, keyboard-types' `ALT_GRAPH`) is held.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AltGr {
    /// AltGr is not held.
    #[default]
    Released,
    /// AltGr is held: the key press types a character and is never a shortcut. Windows reports
    /// it as Ctrl+Alt together with this flag.
    Held,
}

/// A key press as a toolkit reports it: the physical modifiers held (Ctrl, Alt, Shift, Super and
/// Meta flags as the toolkit has them), whether AltGr is held, and the key, which is the logical
/// key with Shift not applied (`z`, not `Z`; `=`, not `+`) as most toolkits' "key without
/// modifiers" gives.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyInput {
    /// The modifiers held. When AltGr is held, leave Ctrl and Alt as the toolkit has them.
    pub modifiers: Modifiers,
    /// The key pressed.
    pub key: Key,
    /// Whether AltGr is held.
    pub alt_gr: AltGr,
}

impl KeyInput {
    /// A key press with AltGr released.
    pub fn new(modifiers: Modifiers, key: Key) -> KeyInput {
        KeyInput {
            modifiers,
            key,
            alt_gr: AltGr::Released,
        }
    }

    /// This key press with `alt_gr` as the toolkit reports it.
    pub fn with_alt_gr(self, alt_gr: AltGr) -> KeyInput {
        KeyInput { alt_gr, ..self }
    }

    /// Whether the press types a character rather than triggering a shortcut: AltGr is held.
    /// Such a press never resolves and is never in the Primary form.
    pub fn types_text(&self) -> bool {
        self.alt_gr == AltGr::Held
    }

    /// The canonical chord on `platform`: the Meta flag folds into the modifier the platform
    /// means by it, letters are lower-cased, and a symbol Shift makes is written without Shift.
    /// With AltGr held, Ctrl and Alt are dropped (Windows adds them to AltGr), so the chord can
    /// never match a Ctrl or Alt binding. This is the one place toolkit differences are absorbed.
    pub fn normalise(&self, platform: Platform) -> Chord {
        self.chord(platform, None)
    }

    /// Like `normalise`, and where `context` has an alias for Primary on `platform`
    /// (`Platform::primary_alias`), the alias modifiers are written as the Primary form: on our
    /// desktop in a terminal, Ctrl+Shift+D is Super+D.
    pub fn normalise_in(&self, platform: Platform, context: Context) -> Chord {
        self.chord(platform, platform.primary_alias(context))
    }

    fn chord(&self, platform: Platform, alias: Option<Modifiers>) -> Chord {
        let held: Modifiers = self.modifiers.iter().map(Modifier::folded).collect();
        let held = match self.alt_gr {
            AltGr::Held => held.without(Modifier::Ctrl).without(Modifier::Alt),
            AltGr::Released => held,
        };
        let modifiers = match alias {
            Some(alias) if held.contains_all(alias) && !held.contains(Modifier::Super) => {
                held.minus(alias).with(Modifier::Super)
            }
            _ => held,
        };
        let key = match self.key {
            Key::Char(c) => Key::character(c),
            Key::Tap(modifier) => Key::Tap(fold(modifier)),
            named @ Key::Named(_) => named,
        };
        Chord::new(modifiers, key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::key::NamedKey;

    type Case = (
        &'static str,
        Platform,
        &'static [Modifier],
        Key,
        &'static str,
    );

    #[test]
    fn toolkit_input_normalises_per_platform() {
        let linux = Platform::Linux {
            desktop: crate::platform::Desktop::Kde,
        };
        let ours = Platform::Linux {
            desktop: crate::platform::Desktop::Ours,
        };
        let cases: &[Case] = &[
            (
                "mac command as super",
                Platform::MacOs,
                &[Modifier::Super],
                Key::Char('c'),
                "Super+C",
            ),
            (
                "mac command as meta",
                Platform::MacOs,
                &[Modifier::Meta],
                Key::Char('c'),
                "Super+C",
            ),
            (
                "windows key as meta",
                Platform::Windows,
                &[Modifier::Meta],
                Key::Char('s'),
                "Super+S",
            ),
            (
                "qt meta on kde",
                linux,
                &[Modifier::Meta, Modifier::Shift],
                Key::Char('s'),
                "Shift+Super+S",
            ),
            (
                "ours super",
                ours,
                &[Modifier::Super],
                Key::Char('c'),
                "Super+C",
            ),
            (
                "ours meta",
                ours,
                &[Modifier::Meta],
                Key::Char('c'),
                "Super+C",
            ),
            (
                "upper case letter",
                Platform::Windows,
                &[Modifier::Ctrl],
                Key::Char('Z'),
                "Ctrl+Z",
            ),
            (
                "shift equals is plus",
                Platform::MacOs,
                &[Modifier::Super, Modifier::Shift],
                Key::Char('='),
                "Super++",
            ),
            (
                "shift slash is question",
                Platform::MacOs,
                &[Modifier::Super, Modifier::Shift],
                Key::Char('/'),
                "Super+?",
            ),
            ("meta tap", linux, &[], Key::Tap(Modifier::Meta), "Super"),
            (
                "named key",
                Platform::Windows,
                &[Modifier::Alt],
                Key::Named(NamedKey::F(4)),
                "Alt+F4",
            ),
        ];
        for (name, platform, mods, key, expected) in cases {
            let input = KeyInput::new(mods.iter().copied().collect(), *key);
            let wanted: Chord = expected
                .parse()
                .unwrap_or_else(|e| panic!("{expected}: {e}"));
            assert_eq!(input.normalise(*platform), wanted, "{name}");
        }
    }

    type AliasCase = (
        &'static str,
        Platform,
        Context,
        &'static [Modifier],
        &'static str,
    );

    #[test]
    fn our_terminal_reads_ctrl_shift_as_command() {
        let ours = Platform::Linux {
            desktop: crate::platform::Desktop::Ours,
        };
        let kde = Platform::Linux {
            desktop: crate::platform::Desktop::Kde,
        };
        let ctrl_shift = &[Modifier::Ctrl, Modifier::Shift];
        let cases: &[AliasCase] = &[
            (
                "ours terminal alias",
                ours,
                Context::Terminal,
                ctrl_shift,
                "Super+D",
            ),
            (
                "ours terminal keeps alt",
                ours,
                Context::Terminal,
                &[Modifier::Ctrl, Modifier::Shift, Modifier::Alt],
                "Alt+Super+D",
            ),
            (
                "ours terminal super",
                ours,
                Context::Terminal,
                &[Modifier::Super],
                "Super+D",
            ),
            (
                "ours terminal plain ctrl",
                ours,
                Context::Terminal,
                &[Modifier::Ctrl],
                "Ctrl+D",
            ),
            (
                "ours normal no alias",
                ours,
                Context::Normal,
                ctrl_shift,
                "Ctrl+Shift+D",
            ),
            (
                "kde terminal no alias",
                kde,
                Context::Terminal,
                ctrl_shift,
                "Ctrl+Shift+D",
            ),
            (
                "mac terminal no alias",
                Platform::MacOs,
                Context::Terminal,
                ctrl_shift,
                "Ctrl+Shift+D",
            ),
        ];
        for (name, platform, context, mods, expected) in cases {
            let input = KeyInput::new(mods.iter().copied().collect(), Key::Char('d'));
            let wanted: Chord = expected
                .parse()
                .unwrap_or_else(|e| panic!("{expected}: {e}"));
            assert_eq!(input.normalise_in(*platform, *context), wanted, "{name}");
        }
    }

    #[test]
    fn alt_gr_drops_ctrl_and_alt() {
        let input = KeyInput::new(
            [Modifier::Ctrl, Modifier::Alt].into_iter().collect(),
            Key::Char('@'),
        );
        assert!(!input.types_text());
        assert!(input.with_alt_gr(AltGr::Held).types_text());
        let held = input.with_alt_gr(AltGr::Held).normalise(Platform::Windows);
        assert_eq!(held.modifiers(), Modifiers::NONE);
        let plain = input.normalise(Platform::Windows);
        assert!(
            plain.modifiers().contains(Modifier::Ctrl) && plain.modifiers().contains(Modifier::Alt)
        );
    }
}
