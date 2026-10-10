//! GTK accelerator text: `<Super>Tab`, `<Primary><Shift>z`, `Print`.

use chordkit::{Chord, Key, Modifier, Modifiers, NamedKey};

/// Reads one accelerator. `Err` is a plain-words reason.
pub fn parse_accelerator(text: &str) -> Result<Chord, String> {
    let mut modifiers = Modifiers::NONE;
    let mut rest = text.trim();
    while let Some(inner) = rest.strip_prefix('<') {
        let (name, after) = inner
            .split_once('>')
            .ok_or_else(|| format!("\"{text}\" is missing a \">\""))?;
        modifiers = modifiers
            .with(modifier(name).ok_or_else(|| format!("\"<{name}>\" is not a modifier I know"))?);
        rest = after;
    }
    let lower = rest.to_lowercase();
    match lower.as_str() {
        "" => Err(format!("\"{text}\" has no key")),
        "super_l" | "super_r" if modifiers.is_empty() => {
            Ok(Chord::new(modifiers, Key::Tap(Modifier::Super)))
        }
        "iso_left_tab" => Ok(Chord::new(
            modifiers.with(Modifier::Shift),
            Key::Named(NamedKey::Tab),
        )),
        _ => key(&lower)
            .map(|key| Chord::new(modifiers, key))
            .ok_or_else(|| format!("\"{rest}\" is not a key I know")),
    }
}

fn modifier(name: &str) -> Option<Modifier> {
    match name.to_lowercase().as_str() {
        "primary" | "control" | "ctrl" => Some(Modifier::Ctrl),
        "alt" | "mod1" => Some(Modifier::Alt),
        "shift" => Some(Modifier::Shift),
        "super" | "mod4" | "meta" => Some(Modifier::Super),
        _ => None,
    }
}

type KeySym = (&'static str, Key);

const KEYSYMS: &[KeySym] = &[
    ("space", Key::Named(NamedKey::Space)),
    ("tab", Key::Named(NamedKey::Tab)),
    ("return", Key::Named(NamedKey::Enter)),
    ("kp_enter", Key::Named(NamedKey::Enter)),
    ("escape", Key::Named(NamedKey::Escape)),
    ("backspace", Key::Named(NamedKey::Backspace)),
    ("delete", Key::Named(NamedKey::Delete)),
    ("insert", Key::Named(NamedKey::Insert)),
    ("home", Key::Named(NamedKey::Home)),
    ("end", Key::Named(NamedKey::End)),
    ("page_up", Key::Named(NamedKey::PageUp)),
    ("prior", Key::Named(NamedKey::PageUp)),
    ("page_down", Key::Named(NamedKey::PageDown)),
    ("next", Key::Named(NamedKey::PageDown)),
    ("up", Key::Named(NamedKey::Up)),
    ("down", Key::Named(NamedKey::Down)),
    ("left", Key::Named(NamedKey::Left)),
    ("right", Key::Named(NamedKey::Right)),
    ("print", Key::Named(NamedKey::PrintScreen)),
    ("plus", Key::Char('+')),
    ("minus", Key::Char('-')),
    ("comma", Key::Char(',')),
    ("period", Key::Char('.')),
    ("grave", Key::Char('`')),
    ("slash", Key::Char('/')),
    ("equal", Key::Char('=')),
];

fn key(lower: &str) -> Option<Key> {
    if let Some((_, key)) = KEYSYMS.iter().find(|entry| entry.0 == lower) {
        return Some(*key);
    }
    let function = lower
        .strip_prefix('f')
        .and_then(|digits| digits.parse::<u8>().ok())
        .and_then(Key::function);
    if function.is_some() {
        return function;
    }
    let mut chars = lower.chars();
    match (chars.next(), chars.next()) {
        (Some(c), None) => Some(Key::character(c)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type Case = (&'static str, &'static str);

    #[test]
    fn accelerators_read_as_chords() {
        let cases: &[Case] = &[
            ("<Super>Tab", "Super+Tab"),
            ("<Primary><Shift>z", "Ctrl+Shift+Z"),
            ("<Alt>F4", "Alt+F4"),
            ("Print", "PrtSc"),
            ("<Shift>Print", "Shift+PrtSc"),
            ("<Super>space", "Super+Space"),
            ("<Control><Alt>Left", "Ctrl+Alt+Left"),
            ("<Super>Page_Up", "Super+PgUp"),
            ("Super_L", "Super"),
            ("<Shift>ISO_Left_Tab", "Shift+Tab"),
            ("<Primary>plus", "Ctrl++"),
        ];
        for (text, expected) in cases {
            let wanted: Chord = expected
                .parse()
                .unwrap_or_else(|e| panic!("{expected}: {e}"));
            assert_eq!(parse_accelerator(text), Ok(wanted), "{text}");
        }
    }

    #[test]
    fn bad_accelerators_say_why() {
        for text in ["", "<Super>", "<Hyperdrive>a", "<Super", "<Super>Banana"] {
            assert!(parse_accelerator(text).is_err(), "{text:?}");
        }
    }
}
