//! The conventions: each platform's default chord for every standard action, as pure data.
//!
//! A platform's table is a complete base plus a short list of rows that differ. Every standard
//! action has a row on every platform: a chord, or an explicit "none" with the reason.

mod cua;
mod gnome;
mod kde;
mod mac;
mod windows;

use crate::chord::DefaultChord;
use crate::platform::{Desktop, Platform};
use crate::standard_action::{SpaceNumber, StandardAction};

/// One row's right-hand side, as written in the tables.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Row {
    /// The default chord, in the text `DefaultChord` reads.
    Key(&'static str),
    /// No default, and why.
    Unbound(&'static str),
}

pub(crate) type TableRow = (StandardAction, Row);

/// The nine Space-switch chords of a platform, or why it has none.
#[derive(Debug)]
pub(crate) enum Spaces {
    Keys([&'static str; 9]),
    None(&'static str),
}

/// What a platform's conventions say about one standard action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Convention {
    /// The default chord.
    Bound(DefaultChord),
    /// Deliberately no default, with the reason in plain words.
    Unbound(&'static str),
}

/// A platform's conventions.
#[derive(Debug, Clone, Copy)]
pub struct ConventionTable {
    platform: Platform,
    overrides: &'static [TableRow],
    base: &'static [TableRow],
    spaces: &'static Spaces,
}

impl ConventionTable {
    /// The conventions of `platform`.
    pub fn for_platform(platform: Platform) -> ConventionTable {
        let (overrides, base, spaces): (&[TableRow], &[TableRow], &Spaces) = match platform {
            Platform::MacOs
            | Platform::Linux {
                desktop: Desktop::Ours,
            } => (&[], mac::ROWS, &mac::SPACES),
            Platform::Windows => (windows::ROWS, cua::ROWS, &windows::SPACES),
            Platform::Linux {
                desktop: Desktop::Kde,
            } => (kde::ROWS, cua::ROWS, &kde::SPACES),
            Platform::Linux {
                desktop: Desktop::Gnome,
            } => (gnome::ROWS, cua::ROWS, &gnome::SPACES),
            Platform::Linux {
                desktop: Desktop::Other,
            } => (&[], cua::ROWS, &cua::SPACES),
        };
        ConventionTable {
            platform,
            overrides,
            base,
            spaces,
        }
    }

    /// The platform this table is for.
    pub fn platform(&self) -> Platform {
        self.platform
    }

    /// The convention for `action`. `None` means the table has a gap or a row that does not
    /// read, which the table tests rule out.
    pub fn lookup(&self, action: StandardAction) -> Option<Convention> {
        let row = match action {
            StandardAction::SwitchToSpace(n) => Some(self.space_row(n)),
            other => self.find(other),
        }?;
        match row {
            Row::Key(text) => text.parse().ok().map(Convention::Bound),
            Row::Unbound(reason) => Some(Convention::Unbound(reason)),
        }
    }

    /// Every standard action with its convention.
    pub fn iter(&self) -> impl Iterator<Item = (StandardAction, Convention)> + '_ {
        StandardAction::all().filter_map(|action| self.lookup(action).map(|c| (action, c)))
    }

    fn find(&self, action: StandardAction) -> Option<Row> {
        self.overrides
            .iter()
            .chain(self.base)
            .find(|(candidate, _)| *candidate == action)
            .map(|(_, row)| *row)
    }

    fn space_row(&self, n: SpaceNumber) -> Row {
        match self.spaces {
            Spaces::Keys(keys) => Row::Key(keys[usize::from(n.get()) - 1]),
            Spaces::None(reason) => Row::Unbound(reason),
        }
    }
}

#[cfg(test)]
mod tests;
