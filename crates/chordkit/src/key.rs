//! The key part of a chord.

use crate::modifier::Modifier;

/// A key with a name rather than a character.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NamedKey {
    /// The space bar.
    Space,
    /// Tab.
    Tab,
    /// Enter or Return.
    Enter,
    /// Escape.
    Escape,
    /// Backspace.
    Backspace,
    /// Forward delete.
    Delete,
    /// Insert.
    Insert,
    /// Home.
    Home,
    /// End.
    End,
    /// Page up.
    PageUp,
    /// Page down.
    PageDown,
    /// Up arrow.
    Up,
    /// Down arrow.
    Down,
    /// Left arrow.
    Left,
    /// Right arrow.
    Right,
    /// Print Screen.
    PrintScreen,
    /// A function key, F1 to F24.
    F(u8),
}

/// The key of a chord.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    /// A character key, lower case (the key without Shift applied).
    Char(char),
    /// A named key.
    Named(NamedKey),
    /// A modifier pressed and released on its own.
    Tap(Modifier),
}

impl Key {
    /// A character key; letters are lower-cased so `Z` and `z` are one key.
    pub fn character(c: char) -> Key {
        Key::Char(c.to_lowercase().next().unwrap_or(c))
    }

    /// Function key `n`, when `n` is 1 to 24.
    pub fn function(n: u8) -> Option<Key> {
        (1..=24).contains(&n).then_some(Key::Named(NamedKey::F(n)))
    }

    /// Whether this is a character a person types: a letter, digit, punctuation or symbol.
    pub(crate) fn is_printable(self) -> bool {
        matches!(self, Key::Char(c) if !c.is_whitespace() && !c.is_control())
    }

    /// Whether this key is not a shortcut key on its own inside a text field: characters,
    /// the space bar and the editing keys belong to the field until a modifier joins them.
    pub(crate) fn belongs_to_text_field(self) -> bool {
        match self {
            Key::Char(_) => true,
            Key::Tap(_) => false,
            Key::Named(NamedKey::Escape | NamedKey::PrintScreen | NamedKey::F(_)) => false,
            Key::Named(_) => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn characters_fold_case_and_function_keys_are_bounded() {
        assert_eq!(Key::character('Z'), Key::Char('z'));
        assert!(Key::Char('d').is_printable() && Key::Char('@').is_printable());
        assert!(!Key::Char(' ').is_printable());
        assert!(!Key::Named(NamedKey::Tab).is_printable());
        assert_eq!(Key::function(0), None);
        assert_eq!(Key::function(24), Some(Key::Named(NamedKey::F(24))));
        assert_eq!(Key::function(25), None);
    }
}
