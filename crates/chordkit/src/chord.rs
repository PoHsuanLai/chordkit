//! Chords: modifiers plus a key. `Chord` is concrete (what the keyboard sends);
//! `DefaultChord` may say `Primary`, which each platform resolves.

use crate::key::Key;
use crate::modifier::{Modifier, Modifiers};
use crate::platform::{Context, Platform};

/// A concrete chord: physical modifiers plus a key, already canonical.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Chord {
    modifiers: Modifiers,
    key: Key,
}

impl Chord {
    /// A chord, made canonical: Meta folds into Super; a tapped modifier carries no other
    /// modifier; and the symbols Shift makes on a US layout (`Shift+=` is `+`, `Shift+/` is
    /// `?`) are written as the symbol with no Shift, as menus write them.
    pub fn new(modifiers: Modifiers, key: Key) -> Chord {
        let modifiers = if modifiers.contains(Modifier::Meta) {
            modifiers.without(Modifier::Meta).with(Modifier::Super)
        } else {
            modifiers
        };
        let shifted = modifiers.contains(Modifier::Shift);
        match (key, shifted) {
            (Key::Tap(modifier), _) => Chord {
                modifiers: Modifiers::NONE,
                key: Key::Tap(fold_meta(modifier)),
            },
            (Key::Char('='), true) => Chord {
                modifiers: modifiers.without(Modifier::Shift),
                key: Key::Char('+'),
            },
            (Key::Char('/'), true) => Chord {
                modifiers: modifiers.without(Modifier::Shift),
                key: Key::Char('?'),
            },
            (Key::Char('+' | '?'), true) => Chord {
                modifiers: modifiers.without(Modifier::Shift),
                key,
            },
            _ => Chord { modifiers, key },
        }
    }

    /// The modifiers held.
    pub fn modifiers(self) -> Modifiers {
        self.modifiers
    }

    /// The key.
    pub fn key(self) -> Key {
        self.key
    }

    /// Whether a text field wants this chord: a typing or editing key with no Ctrl, Alt or
    /// Super (Shift alone still types).
    pub fn belongs_to_text_field(self) -> bool {
        let command = [Modifier::Ctrl, Modifier::Alt, Modifier::Super];
        self.key.belongs_to_text_field() && !command.iter().any(|m| self.modifiers.contains(*m))
    }

    /// Whether a program in a terminal keeps this chord: Ctrl without Shift or Super.
    pub fn is_plain_ctrl(self) -> bool {
        self.modifiers.contains(Modifier::Ctrl)
            && !self.modifiers.contains(Modifier::Shift)
            && !self.modifiers.contains(Modifier::Super)
    }
}

fn fold_meta(modifier: Modifier) -> Modifier {
    match modifier {
        Modifier::Meta => Modifier::Super,
        other => other,
    }
}

/// Whether a default chord includes the platform's primary modifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrimaryUse {
    /// The chord adds the primary modifier (Command on a Mac, Ctrl elsewhere).
    Held,
    /// The chord is exactly its physical modifiers.
    Unused,
}

/// A chord as defaults and overrides write it: portable, possibly using `Primary`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DefaultChord {
    primary: PrimaryUse,
    base: Chord,
}

impl DefaultChord {
    /// A default chord: `base` plus, when `primary` is `Held`, the platform's primary modifier.
    pub fn new(primary: PrimaryUse, base: Chord) -> DefaultChord {
        DefaultChord { primary, base }
    }

    /// Whether the primary modifier is part of it.
    pub fn primary(self) -> PrimaryUse {
        self.primary
    }

    /// The physical part.
    pub fn base(self) -> Chord {
        self.base
    }

    /// The concrete chord on `platform` in `context`. `None` when it has no form there: in a
    /// terminal where Primary is Ctrl+Shift, a chord that already uses Shift cannot add it.
    pub fn resolve(self, platform: Platform, context: Context) -> Option<Chord> {
        match self.primary {
            PrimaryUse::Unused => Some(self.base),
            PrimaryUse::Held => {
                let primary = platform.primary(context);
                let clash = context == Context::Terminal
                    && primary.contains(Modifier::Shift)
                    && self.base.modifiers().contains(Modifier::Shift);
                (!clash).then(|| Chord::new(self.base.modifiers().union(primary), self.base.key()))
            }
        }
    }
}

impl From<Chord> for DefaultChord {
    fn from(chord: Chord) -> DefaultChord {
        DefaultChord::new(PrimaryUse::Unused, chord)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::key::NamedKey;
    use crate::platform::Desktop;

    fn mods(list: &[Modifier]) -> Modifiers {
        list.iter().copied().collect()
    }

    #[test]
    fn canonical_form_folds_meta_and_shifted_symbols() {
        let meta_z = Chord::new(mods(&[Modifier::Meta]), Key::Char('z'));
        assert_eq!(meta_z.modifiers(), mods(&[Modifier::Super]));
        let plus = Chord::new(mods(&[Modifier::Shift, Modifier::Ctrl]), Key::Char('='));
        assert_eq!(plus, Chord::new(mods(&[Modifier::Ctrl]), Key::Char('+')));
        let tap = Chord::new(mods(&[Modifier::Ctrl]), Key::Tap(Modifier::Meta));
        assert_eq!(tap, Chord::new(Modifiers::NONE, Key::Tap(Modifier::Super)));
    }

    #[test]
    fn primary_resolves_per_platform_and_terminal_drops_shift_clashes() {
        let undo = DefaultChord::new(
            PrimaryUse::Held,
            Chord::new(Modifiers::NONE, Key::Char('z')),
        );
        let redo = DefaultChord::new(
            PrimaryUse::Held,
            Chord::new(mods(&[Modifier::Shift]), Key::Char('z')),
        );
        let windows = Platform::Windows;
        let ours = Platform::Linux {
            desktop: Desktop::Ours,
        };
        assert_eq!(
            undo.resolve(windows, Context::Normal),
            Some(Chord::new(mods(&[Modifier::Ctrl]), Key::Char('z')))
        );
        assert_eq!(
            undo.resolve(windows, Context::Terminal),
            Some(Chord::new(
                mods(&[Modifier::Ctrl, Modifier::Shift]),
                Key::Char('z')
            ))
        );
        assert_eq!(redo.resolve(windows, Context::Terminal), None);
        assert_eq!(
            redo.resolve(ours, Context::Terminal),
            Some(Chord::new(
                mods(&[Modifier::Shift, Modifier::Super]),
                Key::Char('z')
            ))
        );
    }

    #[test]
    fn text_fields_and_terminals_claim_their_keys() {
        let plain = Chord::new(Modifiers::NONE, Key::Char('j'));
        let shifted = Chord::new(mods(&[Modifier::Shift]), Key::Named(NamedKey::Tab));
        let command = Chord::new(mods(&[Modifier::Super]), Key::Char('j'));
        let escape = Chord::new(Modifiers::NONE, Key::Named(NamedKey::Escape));
        assert!(plain.belongs_to_text_field() && shifted.belongs_to_text_field());
        assert!(!command.belongs_to_text_field() && !escape.belongs_to_text_field());
        assert!(Chord::new(mods(&[Modifier::Ctrl]), Key::Char('c')).is_plain_ctrl());
        assert!(
            !Chord::new(mods(&[Modifier::Ctrl, Modifier::Shift]), Key::Char('c')).is_plain_ctrl()
        );
    }
}
