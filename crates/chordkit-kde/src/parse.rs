//! Reading the `[Shortcuts]` group of kdeglobals.

use chordkit::keyfile;
use chordkit::{Chord, SystemImport};

use crate::names::action_named;

/// The value for an unbound shortcut.
const UNBOUND: &str = "none";

/// The system bindings in `text`, a kdeglobals file. A value lists alternatives separated by
/// `;` or a tab; `none` unbinds. A chord that cannot be read is reported in `problems` and
/// skipped; other names in the group are not ours and are ignored.
pub fn parse_kdeglobals(text: &str) -> SystemImport {
    let mut import = SystemImport::default();
    let shortcuts = keyfile::parse(text)
        .into_iter()
        .filter(|group| group.name == "Shortcuts");
    for entry in shortcuts.flat_map(|group| group.entries) {
        let Some(action) = action_named(&entry.key) else {
            continue;
        };
        let problems_before = import.problems.len();
        let mut chords: Vec<Chord> = Vec::new();
        for part in entry
            .value
            .split([';', '\t'])
            .map(str::trim)
            .filter(|p| !p.is_empty())
        {
            if part.eq_ignore_ascii_case(UNBOUND) {
                continue;
            }
            match part.parse::<Chord>() {
                Ok(chord) => chords.push(chord),
                Err(why) => import
                    .problems
                    .push(format!("KDE's {} shortcut \"{part}\": {why}", entry.key)),
            }
        }
        // A value that was all unreadable leaves the action as it was, not unbound.
        let all_unreadable = chords.is_empty() && import.problems.len() > problems_before;
        if !all_unreadable {
            import.add(action, chords);
        }
    }
    import
}
