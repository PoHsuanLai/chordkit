//! Reading `[group]` / `key=value` text, the shape of kdeglobals and of a dconf dump. Shared by
//! the source crates so it has one home.

/// One `[group]` and its `key=value` lines, in file order.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Group {
    /// The group's name without brackets; empty before the first header.
    pub name: String,
    /// The entries, keys and values trimmed.
    pub entries: Vec<Entry>,
}

/// One `key=value` line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// The key.
    pub key: String,
    /// The value, unparsed.
    pub value: String,
}

/// Splits `text` into groups. Blank lines and lines starting with `#` or `;` are skipped, and
/// so are lines that are neither a header nor `key=value`.
pub fn parse(text: &str) -> Vec<Group> {
    let mut groups: Vec<Group> = vec![Group::default()];
    let lines = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with(['#', ';']));
    for line in lines {
        let header = line
            .strip_prefix('[')
            .and_then(|rest| rest.strip_suffix(']'));
        if let Some(name) = header {
            groups.push(Group {
                name: name.to_owned(),
                entries: Vec::new(),
            });
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let entry = Entry {
            key: key.trim().to_owned(),
            value: value.trim().to_owned(),
        };
        if let Some(group) = groups.last_mut() {
            group.entries.push(entry);
        }
    }
    groups
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groups_keep_their_entries_in_order() {
        let groups = parse("a=1\n# note\n[One]\nx = 2\ny=3\n\n[Two]\nnonsense\nz=4\n");
        let names: Vec<&str> = groups.iter().map(|g| g.name.as_str()).collect();
        assert_eq!(names, ["", "One", "Two"]);
        assert_eq!(
            groups[1].entries[0],
            Entry {
                key: "x".into(),
                value: "2".into()
            }
        );
        assert_eq!(groups[1].entries.len(), 2);
        assert_eq!(groups[2].entries.len(), 1);
    }
}
