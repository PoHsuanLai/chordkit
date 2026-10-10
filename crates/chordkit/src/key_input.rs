//! What a toolkit hands in for a key press, and its normalisation to a chord.

use crate::chord::Chord;
use crate::key::Key;
use crate::modifier::{Modifier, Modifiers};
use crate::platform::Platform;

/// A key press as a toolkit reports it: the physical modifiers held (Ctrl, Alt, Shift, Super and
/// Meta flags as the toolkit has them) and the key, which is the logical key with Shift not
/// applied (`z`, not `Z`; `=`, not `+`) as most toolkits' "key without modifiers" gives.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyInput {
    /// The modifiers held.
    pub modifiers: Modifiers,
    /// The key pressed.
    pub key: Key,
}

impl KeyInput {
    /// A key press.
    pub fn new(modifiers: Modifiers, key: Key) -> KeyInput {
        KeyInput { modifiers, key }
    }

    /// The canonical chord on `platform`: the Meta flag folds into the modifier the platform
    /// means by it, letters are lower-cased, and a symbol Shift makes is written without Shift.
    /// This is the one place toolkit differences are absorbed.
    pub fn normalise(&self, platform: Platform) -> Chord {
        let meta = platform.meta_as();
        let fold = |modifier: Modifier| {
            if modifier == Modifier::Meta {
                meta
            } else {
                modifier
            }
        };
        let modifiers: Modifiers = self.modifiers.iter().map(fold).collect();
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
}
