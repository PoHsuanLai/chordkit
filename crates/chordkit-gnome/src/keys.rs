//! The dconf groups and keys that mean standard actions.

use chordkit::{SpaceNumber, StandardAction};

type Mapping = (&'static str, &'static str, StandardAction);

const WM: &str = "org/gnome/desktop/wm/keybindings";
const MEDIA: &str = "org/gnome/settings-daemon/plugins/media-keys";
const SHELL: &str = "org/gnome/shell/keybindings";
const MUTTER: &str = "org/gnome/mutter";

/// (group, key, action).
const MAPPINGS: &[Mapping] = &[
    (WM, "minimize", StandardAction::Minimize),
    (WM, "switch-applications", StandardAction::NextApp),
    (
        WM,
        "switch-applications-backward",
        StandardAction::PreviousApp,
    ),
    (WM, "switch-group", StandardAction::NextWindow),
    (WM, "switch-to-workspace-left", StandardAction::SpaceLeft),
    (WM, "switch-to-workspace-right", StandardAction::SpaceRight),
    (WM, "switch-input-source", StandardAction::NextInputSource),
    (WM, "toggle-fullscreen", StandardAction::FullScreen),
    (MEDIA, "screensaver", StandardAction::LockScreen),
    (MEDIA, "logout", StandardAction::LogOut),
    (MEDIA, "screenshot", StandardAction::ScreenshotScreen),
    (
        MEDIA,
        "area-screenshot",
        StandardAction::ScreenshotSelection,
    ),
    (MEDIA, "search", StandardAction::Launcher),
    (SHELL, "show-screenshot-ui", StandardAction::ScreenshotTools),
    (SHELL, "toggle-overview", StandardAction::MissionControl),
    (MUTTER, "overlay-key", StandardAction::MissionControl),
];

const WORKSPACE_PREFIX: &str = "switch-to-workspace-";

/// The standard action a dconf `group` and `key` mean.
pub(crate) fn action_for(group: &str, key: &str) -> Option<StandardAction> {
    let numbered = (group == WM)
        .then(|| key.strip_prefix(WORKSPACE_PREFIX))
        .flatten()
        .and_then(|digits| digits.parse::<u8>().ok())
        .and_then(SpaceNumber::new);
    if let Some(space) = numbered {
        return Some(StandardAction::SwitchToSpace(space));
    }
    MAPPINGS
        .iter()
        .find(|(g, k, _)| *g == group && *k == key)
        .map(|(_, _, action)| *action)
}
