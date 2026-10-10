//! Reading a dconf dump.

use chordkit::keyfile;
use chordkit::{Chord, SystemImport};

use crate::accelerator::parse_accelerator;
use crate::keys::action_for;

/// The system bindings in `text`, the output of `dconf dump /`. Only the keys that mean a
/// standard action are read. Values are GVariant text: `['<Super>h', '<Alt>F9']`, `'<Super>l'`,
/// `@as []` (unbound) or `'disabled'` (unbound). An accelerator that cannot be read is reported
/// in `problems`.
pub fn parse_dconf_dump(text: &str) -> SystemImport {
    let mut import = SystemImport::default();
    for group in keyfile::parse(text) {
        for entry in &group.entries {
            let Some(action) = action_for(&group.name, &entry.key) else {
                continue;
            };
            let mut chords: Vec<Chord> = Vec::new();
            let mut unreadable = 0;
            for accelerator in strings(&entry.value).iter().filter(|a| !is_disabled(a)) {
                match parse_accelerator(accelerator) {
                    Ok(chord) => chords.push(chord),
                    Err(why) => {
                        unreadable += 1;
                        import
                            .problems
                            .push(format!("GNOME's {} shortcut: {why}", entry.key));
                    }
                }
            }
            if chords.is_empty() && unreadable > 0 {
                continue;
            }
            import.add(action, chords);
        }
    }
    import
}

fn is_disabled(accelerator: &str) -> bool {
    accelerator.is_empty() || accelerator.eq_ignore_ascii_case("disabled")
}

/// The strings in a GVariant text value: one quoted string, or an array of them.
fn strings(value: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut current: Option<(char, String)> = None;
    for c in value.chars() {
        current = match (current.take(), c) {
            (None, '\'' | '"') => Some((c, String::new())),
            (None, _) => None,
            (Some((quote, text)), c) if c == quote => {
                found.push(text);
                None
            }
            (Some((quote, mut text)), c) => {
                text.push(c);
                Some((quote, text))
            }
        };
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_split_into_their_strings() {
        assert_eq!(strings("['<Super>h', '<Alt>F9']"), ["<Super>h", "<Alt>F9"]);
        assert_eq!(strings("'<Super>l'"), ["<Super>l"]);
        assert!(strings("@as []").is_empty());
        assert_eq!(strings("[\"<Super>x\"]"), ["<Super>x"]);
    }
}
