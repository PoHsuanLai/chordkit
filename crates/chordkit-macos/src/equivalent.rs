//! The key-equivalent text: `@` Command, `$` Shift, `~` Option, `^` Control, then one key.

use chordkit::{Chord, Key, Modifier, Modifiers, NamedKey};

/// Reads "@$z" and the like. `Err` is a plain-words reason.
pub fn parse_key_equivalent(text: &str) -> Result<Chord, String> {
    let mut modifiers = Modifiers::NONE;
    let mut rest = text;
    while let Some(c) = rest.chars().next() {
        let Some(modifier) = modifier_of(c) else {
            break;
        };
        modifiers = modifiers.with(modifier);
        rest = &rest[c.len_utf8()..];
    }
    let mut keys = rest.chars();
    match (keys.next(), keys.next()) {
        (Some(c), None) => Ok(Chord::new(modifiers, key_of(c))),
        (None, _) => Err(format!("\"{text}\" has no key")),
        _ => Err(format!("\"{text}\" has more than one key")),
    }
}

fn modifier_of(c: char) -> Option<Modifier> {
    match c {
        '@' => Some(Modifier::Super),
        '$' => Some(Modifier::Shift),
        '~' => Some(Modifier::Alt),
        '^' => Some(Modifier::Ctrl),
        _ => None,
    }
}

/// Function keys live in a private-use block: arrows from U+F700, F1 from U+F704.
fn key_of(c: char) -> Key {
    match c {
        '\r' | '\n' | '\u{3}' => Key::Named(NamedKey::Enter),
        '\t' => Key::Named(NamedKey::Tab),
        '\u{1b}' => Key::Named(NamedKey::Escape),
        '\u{8}' | '\u{7f}' => Key::Named(NamedKey::Backspace),
        ' ' => Key::Named(NamedKey::Space),
        '\u{f700}' => Key::Named(NamedKey::Up),
        '\u{f701}' => Key::Named(NamedKey::Down),
        '\u{f702}' => Key::Named(NamedKey::Left),
        '\u{f703}' => Key::Named(NamedKey::Right),
        '\u{f728}' => Key::Named(NamedKey::Delete),
        '\u{f729}' => Key::Named(NamedKey::Home),
        '\u{f72b}' => Key::Named(NamedKey::End),
        '\u{f72c}' => Key::Named(NamedKey::PageUp),
        '\u{f72d}' => Key::Named(NamedKey::PageDown),
        function @ '\u{f704}'..='\u{f71b}' => {
            let number = u32::from(function) - 0xf704 + 1;
            u8::try_from(number)
                .ok()
                .and_then(Key::function)
                .unwrap_or(Key::Char(function))
        }
        other => Key::character(other),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type Case = (&'static str, &'static str);

    #[test]
    fn key_equivalents_read_as_chords() {
        let cases: &[Case] = &[
            ("@z", "Super+Z"),
            ("@$z", "Super+Shift+Z"),
            ("@~^c", "Ctrl+Alt+Super+C"),
            ("@\u{f700}", "Super+Up"),
            ("@\u{f704}", "Super+F1"),
            ("\u{f707}", "F4"),
            ("@\r", "Super+Enter"),
            ("$@=", "Super++"),
            ("@,", "Super+,"),
        ];
        for (text, expected) in cases {
            let wanted: Chord = expected
                .parse()
                .unwrap_or_else(|e| panic!("{expected}: {e}"));
            assert_eq!(parse_key_equivalent(text), Ok(wanted), "{text:?}");
        }
    }

    #[test]
    fn modifiers_without_a_key_or_two_keys_are_refused() {
        assert!(parse_key_equivalent("@$").is_err());
        assert!(parse_key_equivalent("@zz").is_err());
        assert!(parse_key_equivalent("").is_err());
    }
}
