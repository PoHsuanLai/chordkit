//! Reading and writing chords as text: "Primary+Shift+Z", "ctrl+shift+z", "⇧⌘Z".

use std::str::FromStr;

use thiserror::Error;

use crate::chord::{Chord, DefaultChord, PrimaryUse};
use crate::key::{Key, NamedKey};
use crate::modifier::{Modifier, Modifiers};
use crate::platform::{Desktop, Platform};

/// Why a chord's text could not be read.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ChordError {
    /// Nothing was written.
    #[error("the shortcut is empty")]
    Empty,
    /// A word that is neither a modifier nor a key.
    #[error("\"{0}\" is not a key I know")]
    UnknownKey(String),
    /// A word before the key that is not a modifier.
    #[error("\"{0}\" is not a modifier (Ctrl, Alt, Shift or Super)")]
    NotAModifier(String),
    /// Only modifiers were written.
    #[error("a shortcut needs a key as well as its modifiers")]
    NoKey,
    /// `Primary` where only a concrete chord is allowed.
    #[error(
        "\"Primary\" means Command or Ctrl depending on the platform, so it is not allowed here"
    )]
    PrimaryNotAllowed,
}

enum Word {
    Primary,
    Physical(Modifier),
}

const GLYPHS: [(char, Modifier); 4] = [
    ('⌃', Modifier::Ctrl),
    ('⌥', Modifier::Alt),
    ('⇧', Modifier::Shift),
    ('⌘', Modifier::Super),
];

type KeyWord = (&'static str, Key);

const KEY_WORDS: &[KeyWord] = &[
    ("space", Key::Named(NamedKey::Space)),
    ("tab", Key::Named(NamedKey::Tab)),
    ("⇥", Key::Named(NamedKey::Tab)),
    ("enter", Key::Named(NamedKey::Enter)),
    ("return", Key::Named(NamedKey::Enter)),
    ("↩", Key::Named(NamedKey::Enter)),
    ("esc", Key::Named(NamedKey::Escape)),
    ("escape", Key::Named(NamedKey::Escape)),
    ("⎋", Key::Named(NamedKey::Escape)),
    ("backspace", Key::Named(NamedKey::Backspace)),
    ("⌫", Key::Named(NamedKey::Backspace)),
    ("delete", Key::Named(NamedKey::Delete)),
    ("del", Key::Named(NamedKey::Delete)),
    ("⌦", Key::Named(NamedKey::Delete)),
    ("insert", Key::Named(NamedKey::Insert)),
    ("ins", Key::Named(NamedKey::Insert)),
    ("home", Key::Named(NamedKey::Home)),
    ("↖", Key::Named(NamedKey::Home)),
    ("end", Key::Named(NamedKey::End)),
    ("↘", Key::Named(NamedKey::End)),
    ("pageup", Key::Named(NamedKey::PageUp)),
    ("pgup", Key::Named(NamedKey::PageUp)),
    ("⇞", Key::Named(NamedKey::PageUp)),
    ("pagedown", Key::Named(NamedKey::PageDown)),
    ("pgdn", Key::Named(NamedKey::PageDown)),
    ("⇟", Key::Named(NamedKey::PageDown)),
    ("up", Key::Named(NamedKey::Up)),
    ("↑", Key::Named(NamedKey::Up)),
    ("down", Key::Named(NamedKey::Down)),
    ("↓", Key::Named(NamedKey::Down)),
    ("left", Key::Named(NamedKey::Left)),
    ("←", Key::Named(NamedKey::Left)),
    ("right", Key::Named(NamedKey::Right)),
    ("→", Key::Named(NamedKey::Right)),
    ("printscreen", Key::Named(NamedKey::PrintScreen)),
    ("print", Key::Named(NamedKey::PrintScreen)),
    ("prtsc", Key::Named(NamedKey::PrintScreen)),
    ("plus", Key::Char('+')),
    ("minus", Key::Char('-')),
    ("comma", Key::Char(',')),
    ("period", Key::Char('.')),
    ("backtick", Key::Char('`')),
    ("grave", Key::Char('`')),
    ("slash", Key::Char('/')),
];

fn word(token: &str) -> Option<Word> {
    match token.to_lowercase().as_str() {
        "primary" => Some(Word::Primary),
        "ctrl" | "control" | "ctl" => Some(Word::Physical(Modifier::Ctrl)),
        "alt" | "option" | "opt" => Some(Word::Physical(Modifier::Alt)),
        "shift" => Some(Word::Physical(Modifier::Shift)),
        "super" | "cmd" | "command" | "win" | "windows" | "meta" | "logo" => {
            Some(Word::Physical(Modifier::Super))
        }
        _ => None,
    }
}

fn parse_key(token: &str) -> Result<Key, ChordError> {
    let lower = token.to_lowercase();
    if let Some((_, key)) = KEY_WORDS.iter().find(|entry| entry.0 == lower) {
        return Ok(*key);
    }
    let function = lower
        .strip_prefix('f')
        .and_then(|digits| digits.parse::<u8>().ok())
        .and_then(Key::function);
    if let Some(key) = function {
        return Ok(key);
    }
    let mut chars = token.chars();
    match (chars.next(), chars.next()) {
        (Some(c), None) => Ok(Key::character(c)),
        _ => Err(ChordError::UnknownKey(token.to_owned())),
    }
}

fn take_glyphs(text: &str) -> (Modifiers, &str) {
    let mut modifiers = Modifiers::NONE;
    let mut rest = text;
    while let Some(c) = rest.chars().next() {
        match GLYPHS.iter().find(|(glyph, _)| *glyph == c) {
            Some((_, modifier)) => {
                modifiers = modifiers.with(*modifier);
                rest = &rest[c.len_utf8()..];
            }
            None => break,
        }
    }
    (modifiers, rest)
}

/// Splits "Ctrl+Shift+Z" into ("Ctrl+Shift", "Z"), keeping a trailing `+` key whole.
fn split_last(rest: &str) -> (&str, &str) {
    if rest == "+" {
        ("", "+")
    } else if let Some(head) = rest.strip_suffix("++") {
        (head, "+")
    } else {
        rest.rsplit_once('+').unwrap_or(("", rest))
    }
}

fn parse_default(text: &str) -> Result<DefaultChord, ChordError> {
    let text = text.trim();
    if text.is_empty() {
        return Err(ChordError::Empty);
    }
    let (glyphs, rest) = take_glyphs(text);
    let mut modifiers = glyphs;
    let mut primary = PrimaryUse::Unused;
    let (head, last) = split_last(rest);
    for token in head.split('+').map(str::trim).filter(|t| !t.is_empty()) {
        match word(token) {
            Some(Word::Primary) => primary = PrimaryUse::Held,
            Some(Word::Physical(modifier)) => modifiers = modifiers.with(modifier),
            None => return Err(ChordError::NotAModifier(token.to_owned())),
        }
    }
    let last = last.trim();
    let alone = modifiers.is_empty() && primary == PrimaryUse::Unused;
    let key = if last.is_empty() {
        tap_of(glyphs, rest, alone)?
    } else {
        match word(last) {
            Some(Word::Physical(modifier)) if alone => Key::Tap(modifier),
            Some(_) => return Err(ChordError::NoKey),
            None => parse_key(last)?,
        }
    };
    let modifiers = if matches!(key, Key::Tap(_)) {
        Modifiers::NONE
    } else {
        modifiers
    };
    Ok(DefaultChord::new(primary, Chord::new(modifiers, key)))
}

/// A bare glyph such as "⌘" is a tap of that modifier; anything else with no key is an error.
fn tap_of(glyphs: Modifiers, rest: &str, alone: bool) -> Result<Key, ChordError> {
    let mut held = glyphs.iter();
    match (held.next(), held.next()) {
        (Some(modifier), None) if rest.is_empty() => Ok(Key::Tap(modifier)),
        _ if alone => Err(ChordError::Empty),
        _ => Err(ChordError::NoKey),
    }
}

impl FromStr for DefaultChord {
    type Err = ChordError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        parse_default(text)
    }
}

impl FromStr for Chord {
    type Err = ChordError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let parsed = parse_default(text)?;
        match parsed.primary() {
            PrimaryUse::Held => Err(ChordError::PrimaryNotAllowed),
            PrimaryUse::Unused => Ok(parsed.base()),
        }
    }
}

/// How a platform writes shortcuts.
enum Style {
    /// Symbols in the order ⌃⌥⇧⌘ with no separator.
    Glyphs,
    /// Words joined by `+`, modifiers in this order, the OS key under this name.
    Words {
        order: [Modifier; 4],
        os_key: &'static str,
    },
}

const CTRL_FIRST: [Modifier; 4] = [
    Modifier::Ctrl,
    Modifier::Alt,
    Modifier::Shift,
    Modifier::Super,
];
const QT_ORDER: [Modifier; 4] = [
    Modifier::Super,
    Modifier::Ctrl,
    Modifier::Alt,
    Modifier::Shift,
];

fn style_of(platform: Platform) -> Style {
    match platform {
        Platform::MacOs
        | Platform::Linux {
            desktop: Desktop::Ours,
        } => Style::Glyphs,
        Platform::Windows => Style::Words {
            order: CTRL_FIRST,
            os_key: "Win",
        },
        Platform::Linux {
            desktop: Desktop::Kde,
        } => Style::Words {
            order: QT_ORDER,
            os_key: "Meta",
        },
        Platform::Linux { .. } => Style::Words {
            order: CTRL_FIRST,
            os_key: "Super",
        },
    }
}

fn glyph_of(modifier: Modifier) -> &'static str {
    match modifier {
        Modifier::Ctrl => "⌃",
        Modifier::Alt => "⌥",
        Modifier::Shift => "⇧",
        Modifier::Super | Modifier::Meta => "⌘",
    }
}

fn word_of(modifier: Modifier, os_key: &'static str) -> &'static str {
    match modifier {
        Modifier::Ctrl => "Ctrl",
        Modifier::Alt => "Alt",
        Modifier::Shift => "Shift",
        Modifier::Super | Modifier::Meta => os_key,
    }
}

fn key_text(key: Key, style: &Style) -> String {
    let glyphs = matches!(style, Style::Glyphs);
    let os_key: &'static str = match style {
        Style::Words { os_key, .. } => os_key,
        Style::Glyphs => "",
    };
    match key {
        Key::Char(c) => c.to_uppercase().collect(),
        Key::Tap(modifier) if glyphs => glyph_of(modifier).to_owned(),
        Key::Tap(modifier) => word_of(modifier, os_key).to_owned(),
        Key::Named(NamedKey::F(n)) => format!("F{n}"),
        Key::Named(named) => named_text(named, glyphs).to_owned(),
    }
}

fn named_text(key: NamedKey, glyphs: bool) -> &'static str {
    let (glyph, words) = match key {
        NamedKey::Space => ("Space", "Space"),
        NamedKey::Tab => ("⇥", "Tab"),
        NamedKey::Enter => ("↩", "Enter"),
        NamedKey::Escape => ("⎋", "Esc"),
        NamedKey::Backspace => ("⌫", "Backspace"),
        NamedKey::Delete => ("⌦", "Delete"),
        NamedKey::Insert => ("Ins", "Ins"),
        NamedKey::Home => ("↖", "Home"),
        NamedKey::End => ("↘", "End"),
        NamedKey::PageUp => ("⇞", "PgUp"),
        NamedKey::PageDown => ("⇟", "PgDn"),
        NamedKey::Up => ("↑", "Up"),
        NamedKey::Down => ("↓", "Down"),
        NamedKey::Left => ("←", "Left"),
        NamedKey::Right => ("→", "Right"),
        NamedKey::PrintScreen => ("PrtSc", "PrtSc"),
        NamedKey::F(_) => ("F", "F"),
    };
    if glyphs { glyph } else { words }
}

fn render(prefix: &[&str], modifiers: Modifiers, key: Key, style: &Style) -> String {
    match style {
        Style::Glyphs => {
            let mods: String = GLYPHS
                .iter()
                .filter(|(_, modifier)| modifiers.contains(*modifier))
                .map(|(glyph, _)| *glyph)
                .collect();
            format!("{mods}{}", key_text(key, style))
        }
        Style::Words { order, os_key } => {
            let mods = order
                .iter()
                .filter(|modifier| modifiers.contains(**modifier))
                .map(|modifier| word_of(*modifier, os_key));
            let parts: Vec<String> = prefix
                .iter()
                .copied()
                .chain(mods)
                .map(str::to_owned)
                .chain([key_text(key, style)])
                .collect();
            parts.join("+")
        }
    }
}

impl Chord {
    /// The chord as `platform` shows it to a person: "⇧⌘Z" on a Mac or our desktop,
    /// "Ctrl+Shift+Z" elsewhere in that platform's order and words. `FromStr` reads it back.
    pub fn display(self, platform: Platform) -> String {
        render(&[], self.modifiers(), self.key(), &style_of(platform))
    }

    /// The platform-neutral text form (always words, fixed order), stable for files.
    pub fn canonical(self) -> String {
        DefaultChord::from(self).canonical()
    }
}

impl DefaultChord {
    /// The platform-neutral text form: "Primary+Shift+Z", always words, fixed order.
    pub fn canonical(self) -> String {
        let neutral = Style::Words {
            order: CTRL_FIRST,
            os_key: "Super",
        };
        let prefix: &[&str] = match self.primary() {
            PrimaryUse::Held => &["Primary"],
            PrimaryUse::Unused => &[],
        };
        render(prefix, self.base().modifiers(), self.base().key(), &neutral)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chord(text: &str) -> Chord {
        text.parse().unwrap_or_else(|e| panic!("{text}: {e}"))
    }

    #[test]
    fn spellings_of_one_chord_agree() {
        let redo = DefaultChord::from_str("Primary+Shift+Z");
        assert_eq!(redo, DefaultChord::from_str("primary+shift+z"));
        assert_eq!(chord("ctrl+shift+z"), chord("Control+Shift+Z"));
        assert_eq!(chord("⇧⌘Z"), chord("Cmd+Shift+Z"));
        assert_eq!(chord("⇧⌘Z"), chord("Super+Shift+z"));
        assert_eq!(chord("⌘+"), chord("Super++"));
        assert_eq!(chord("Ctrl+-"), chord("Ctrl+Minus"));
    }

    type Bad = (&'static str, ChordError);

    #[test]
    fn bad_text_is_refused_in_words() {
        let cases: &[Bad] = &[
            ("", ChordError::Empty),
            ("Ctrl+", ChordError::NoKey),
            ("Ctrl+Shift", ChordError::NoKey),
            ("Ctrl+Nope", ChordError::UnknownKey("Nope".into())),
            ("Zz+A", ChordError::NotAModifier("Zz".into())),
            ("Primary", ChordError::NoKey),
        ];
        for (text, expected) in cases {
            assert_eq!(
                text.parse::<DefaultChord>(),
                Err(expected.clone()),
                "{text:?}"
            );
        }
        assert_eq!(
            "Primary+C".parse::<Chord>(),
            Err(ChordError::PrimaryNotAllowed)
        );
    }

    #[test]
    fn a_lone_modifier_is_a_tap() {
        let tap = Chord::new(Modifiers::NONE, Key::Tap(Modifier::Super));
        assert_eq!(chord("Super"), tap);
        assert_eq!(chord("⌘"), tap);
        assert_eq!(tap.display(Platform::MacOs), "⌘");
        assert_eq!(tap.display(Platform::Windows), "Win");
    }

    type Shown = (Platform, &'static str, &'static str);

    #[test]
    fn display_follows_each_platforms_order_and_words() {
        let ours = Platform::Linux {
            desktop: Desktop::Ours,
        };
        let kde = Platform::Linux {
            desktop: Desktop::Kde,
        };
        let gnome = Platform::Linux {
            desktop: Desktop::Gnome,
        };
        let cases: &[Shown] = &[
            (Platform::MacOs, "Ctrl+Alt+Shift+Super+Z", "⌃⌥⇧⌘Z"),
            (Platform::MacOs, "Shift+Super+Z", "⇧⌘Z"),
            (ours, "Super+Space", "⌘Space"),
            (Platform::Windows, "Shift+Ctrl+Z", "Ctrl+Shift+Z"),
            (Platform::Windows, "Super+Shift+S", "Shift+Win+S"),
            (kde, "Shift+Ctrl+Z", "Ctrl+Shift+Z"),
            (kde, "Super+Alt+K", "Meta+Alt+K"),
            (gnome, "Super+Shift+Tab", "Shift+Super+Tab"),
            (gnome, "Ctrl+F5", "Ctrl+F5"),
        ];
        for (platform, input, shown) in cases {
            let c = chord(input);
            assert_eq!(c.display(*platform), *shown, "{platform:?} {input}");
        }
    }

    #[test]
    fn display_reads_back_on_every_platform() {
        let samples = [
            "Ctrl+Alt+Shift+Super+Z",
            "Super+Space",
            "Ctrl+Up",
            "Alt+F4",
            "Shift+Tab",
            "Ctrl++",
            "Ctrl+-",
            "Ctrl+,",
            "Super+`",
            "Super",
            "Ctrl+Alt+Delete",
            "Shift+PgUp",
            "Alt+Enter",
            "Ctrl+Backspace",
            "PrtSc",
            "Shift+Super+3",
            "Ctrl+Esc",
            "Super+Home",
            "Ctrl+End",
        ];
        for platform in Platform::ALL {
            for text in samples {
                let original = chord(text);
                let shown = original.display(platform);
                assert_eq!(
                    shown.parse::<Chord>(),
                    Ok(original),
                    "{platform:?}: {shown}"
                );
            }
        }
    }

    #[test]
    fn canonical_text_keeps_primary_and_round_trips() {
        for text in [
            "Primary+Shift+Z",
            "Ctrl+Alt+Shift+Super+A",
            "Primary++",
            "Super",
            "F11",
        ] {
            let parsed: DefaultChord = text.parse().unwrap_or_else(|e| panic!("{text}: {e}"));
            assert_eq!(
                parsed.canonical().parse::<DefaultChord>(),
                Ok(parsed),
                "{text}"
            );
        }
        assert_eq!(chord("shift+ctrl+z").canonical(), "Ctrl+Shift+Z");
    }
}
