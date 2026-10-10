//! The person's changes to the keymap, as a tiny line format.
//!
//! One decision: the core stays serde-free, so the format is lines, not TOML. A person's file
//! is a short list of `action = chord`, which needs no nesting, and a line format lets a bad
//! line be reported and skipped without losing the rest. Apps and daemons that already hold
//! TOML or JSON can build `Overrides` from their own structures through `Overrides::from_entries`.
//!
//! ```text
//! # comments start with #
//! copy = Primary+C              # standard actions by slug
//! mail.compose = Primary+Alt+N  # app actions by app.action id
//! mail.compose = Primary+Shift+N  # a repeated action adds another chord
//! redo = none                   # unbind
//! ```

use std::fmt;

use crate::action::Action;
use crate::chord::DefaultChord;

/// One action's replacement chords. No chords unbinds the action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverrideEntry {
    /// The line it came from (1-based; 0 when built in code).
    pub line: usize,
    /// The action changed.
    pub action: Action,
    /// Its chords now; empty means unbound.
    pub chords: Vec<DefaultChord>,
}

/// A line (or an applied change) that was skipped, with the reason in plain words.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverrideProblem {
    /// The line (1-based; 0 when it did not come from a file).
    pub line: usize,
    /// What was wrong.
    pub what: String,
}

impl fmt::Display for OverrideProblem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.line {
            0 => write!(f, "{}", self.what),
            line => write!(f, "line {line}: {}", self.what),
        }
    }
}

/// The person's changes.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Overrides {
    entries: Vec<OverrideEntry>,
}

impl Overrides {
    /// Overrides built in code. A later entry for the same action replaces an earlier one.
    pub fn from_entries(entries: impl IntoIterator<Item = OverrideEntry>) -> Overrides {
        let mut kept: Vec<OverrideEntry> = Vec::new();
        for entry in entries {
            match kept.iter_mut().find(|e| e.action == entry.action) {
                Some(existing) => *existing = entry,
                None => kept.push(entry),
            }
        }
        Overrides { entries: kept }
    }

    /// The changes, one per action.
    pub fn entries(&self) -> &[OverrideEntry] {
        &self.entries
    }

    /// Reads the line format. Bad lines are reported and skipped, never fatal.
    pub fn parse(text: &str) -> (Overrides, Vec<OverrideProblem>) {
        let mut entries: Vec<OverrideEntry> = Vec::new();
        let mut problems = Vec::new();
        for (index, raw) in text.lines().enumerate() {
            let line = index + 1;
            let content = raw
                .split_once(" #")
                .map_or(raw, |(before, _)| before)
                .trim();
            if content.is_empty() || content.starts_with('#') {
                continue;
            }
            match parse_line(content) {
                Ok((action, chord)) => add(&mut entries, line, action, chord),
                Err(what) => problems.push(OverrideProblem { line, what }),
            }
        }
        (Overrides { entries }, problems)
    }

    /// Writes the line format; `parse` reads it back.
    pub fn render(&self) -> String {
        let lines = self.entries.iter().flat_map(|entry| {
            let id = entry.action.id();
            let chords: Vec<String> = entry.chords.iter().map(|c| c.canonical()).collect();
            if chords.is_empty() {
                vec![format!("{id} = none")]
            } else {
                chords.into_iter().map(|c| format!("{id} = {c}")).collect()
            }
        });
        lines.map(|line| line + "\n").collect()
    }
}

/// `None` is the unbind marker; otherwise one chord.
type Value = Option<DefaultChord>;

fn parse_line(content: &str) -> Result<(Action, Value), String> {
    let (name, value) = content
        .split_once('=')
        .ok_or_else(|| format!("\"{content}\" has no \"=\"; write action = shortcut"))?;
    let action: Action = name.trim().parse().map_err(|e| format!("{e}"))?;
    let value = value.trim();
    if value.eq_ignore_ascii_case("none") {
        return Ok((action, None));
    }
    let chord: DefaultChord = value.parse().map_err(|e| format!("{e}"))?;
    Ok((action, Some(chord)))
}

fn add(entries: &mut Vec<OverrideEntry>, line: usize, action: Action, value: Value) {
    let chords: Vec<DefaultChord> = value.into_iter().collect();
    match entries.iter_mut().find(|e| e.action == action) {
        Some(existing) if chords.is_empty() => existing.chords.clear(),
        Some(existing) => existing.chords.extend(chords),
        None => entries.push(OverrideEntry {
            line,
            action,
            chords,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::standard_action::StandardAction;

    #[test]
    fn lines_parse_merge_and_report_what_they_skip() {
        let text = "\
# my keys
copy = Primary+C   # same as before
mail.compose = Primary+Alt+N
mail.compose = Primary+Shift+N
redo = none
teleport = Primary+T
paste Primary+V
cut = Primary+Nope
Bad.id = Primary+B
";
        let (overrides, problems) = Overrides::parse(text);
        let ids: Vec<String> = overrides.entries().iter().map(|e| e.action.id()).collect();
        assert_eq!(ids, ["copy", "mail.compose", "redo"]);
        assert_eq!(overrides.entries()[1].chords.len(), 2);
        assert!(overrides.entries()[2].chords.is_empty());
        let lines: Vec<usize> = problems.iter().map(|p| p.line).collect();
        assert_eq!(lines, [6, 7, 8, 9]);
        assert!(problems[0].to_string().starts_with("line 6: "));
    }

    #[test]
    fn render_reads_back() {
        let (overrides, _) =
            Overrides::parse("undo = Primary+Z\nundo = Ctrl+Y\nredo = none\nmail.x-y = F5\n");
        let (again, problems) = Overrides::parse(&overrides.render());
        assert!(problems.is_empty());
        let strip = |o: &Overrides| -> Vec<(Action, Vec<DefaultChord>)> {
            o.entries()
                .iter()
                .map(|e| (e.action.clone(), e.chords.clone()))
                .collect()
        };
        assert_eq!(strip(&again), strip(&overrides));
        assert_eq!(
            overrides.entries()[0].action,
            Action::Standard(StandardAction::Undo)
        );
    }
}
