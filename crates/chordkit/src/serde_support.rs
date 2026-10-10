//! Serde for the types that are stored or cross a wire. Each is a string in its stable form:
//! a chord as its canonical text, an action as its id, a platform as its slug.

use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error};

use crate::action::{Action, AppAction};
use crate::chord::{Chord, DefaultChord};
use crate::platform::Platform;
use crate::standard_action::StandardAction;

macro_rules! string_serde {
    ($ty:ty, $write:ident) => {
        impl Serialize for $ty {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.serialize_str(&self.$write())
            }
        }

        impl<'de> Deserialize<'de> for $ty {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let text = String::deserialize(deserializer)?;
                text.parse::<$ty>().map_err(D::Error::custom)
            }
        }
    };
}

impl Platform {
    fn to_text(self) -> String {
        self.slug().to_owned()
    }
}

impl AppAction {
    fn to_text(&self) -> String {
        self.id().to_owned()
    }
}

impl StandardAction {
    fn to_text(self) -> String {
        self.slug()
    }
}

string_serde!(Chord, canonical);
string_serde!(DefaultChord, canonical);
string_serde!(Action, id);
string_serde!(AppAction, to_text);
string_serde!(Platform, to_text);

string_serde!(StandardAction, to_text);

#[cfg(test)]
mod tests {
    use super::*;

    fn round_trip<T>(value: &T) -> T
    where
        T: Serialize + for<'de> Deserialize<'de>,
    {
        let json = serde_json::to_string(value).unwrap_or_else(|e| panic!("{e}"));
        serde_json::from_str(&json).unwrap_or_else(|e| panic!("{json}: {e}"))
    }

    #[test]
    fn stored_types_round_trip_as_strings() {
        let chord: DefaultChord = "Primary+Shift+Z".parse().unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(round_trip(&chord), chord);
        assert_eq!(
            serde_json::to_string(&chord).unwrap_or_default(),
            "\"Primary+Shift+Z\""
        );
        for platform in Platform::ALL {
            assert_eq!(round_trip(&platform), platform);
        }
        let action = Action::App(AppAction::new("mail.compose").unwrap_or_else(|e| panic!("{e}")));
        assert_eq!(round_trip(&action), action);
        for standard in StandardAction::all() {
            assert_eq!(round_trip(&standard), standard);
        }
    }
}
