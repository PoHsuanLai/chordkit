//! The physical modifier keys and sets of them.

/// A modifier key as the hardware has it. `Meta` exists only so a toolkit can report it;
/// every chord folds it into another modifier (see `Platform::meta_as`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Modifier {
    /// The Control key.
    Ctrl,
    /// The Alt key (Option on a Mac).
    Alt,
    /// The Shift key.
    Shift,
    /// The OS key: Command on a Mac, the Windows key on Windows, Super on Linux.
    Super,
    /// The Meta key some toolkits report.
    Meta,
}

impl Modifier {
    /// Every modifier, in the order sets are walked.
    pub const ALL: [Modifier; 5] = [
        Modifier::Ctrl,
        Modifier::Alt,
        Modifier::Shift,
        Modifier::Super,
        Modifier::Meta,
    ];

    const fn bit(self) -> u8 {
        match self {
            Modifier::Ctrl => 1,
            Modifier::Alt => 2,
            Modifier::Shift => 4,
            Modifier::Super => 8,
            Modifier::Meta => 16,
        }
    }
}

/// A set of modifiers held together.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct Modifiers(u8);

impl Modifiers {
    /// No modifier held.
    pub const NONE: Modifiers = Modifiers(0);

    /// The set holding only `modifier`.
    pub const fn of(modifier: Modifier) -> Modifiers {
        Modifiers(modifier.bit())
    }

    /// This set plus `modifier`.
    pub const fn with(self, modifier: Modifier) -> Modifiers {
        Modifiers(self.0 | modifier.bit())
    }

    /// This set minus `modifier`.
    pub const fn without(self, modifier: Modifier) -> Modifiers {
        Modifiers(self.0 & !modifier.bit())
    }

    /// Everything held in either set.
    pub const fn union(self, other: Modifiers) -> Modifiers {
        Modifiers(self.0 | other.0)
    }

    /// Whether `modifier` is in the set.
    pub const fn contains(self, modifier: Modifier) -> bool {
        self.0 & modifier.bit() != 0
    }

    /// Whether nothing is held.
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// The modifiers held, in `Modifier::ALL` order.
    pub fn iter(self) -> impl Iterator<Item = Modifier> {
        Modifier::ALL
            .into_iter()
            .filter(move |modifier| self.contains(*modifier))
    }
}

impl FromIterator<Modifier> for Modifiers {
    fn from_iter<I: IntoIterator<Item = Modifier>>(iter: I) -> Self {
        iter.into_iter()
            .fold(Modifiers::NONE, |set, modifier| set.with(modifier))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_algebra_adds_and_removes() {
        let set = Modifiers::of(Modifier::Ctrl).with(Modifier::Shift);
        assert!(set.contains(Modifier::Ctrl) && set.contains(Modifier::Shift));
        assert!(!set.contains(Modifier::Alt));
        assert_eq!(set.without(Modifier::Ctrl), Modifiers::of(Modifier::Shift));
        assert_eq!(
            set.iter().collect::<Vec<_>>(),
            vec![Modifier::Ctrl, Modifier::Shift]
        );
        assert!(Modifiers::NONE.is_empty());
    }
}
