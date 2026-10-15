# Findings

## Standing facts

- Primary is Command (Super) on macOS and on our desktop, Ctrl on Windows and other Linux. In a
  terminal, Primary is Super on macOS and our desktop and Ctrl+Shift elsewhere, and plain Ctrl is never
  matched except with `=`, `-` and `0`, the text size keys every terminal takes (Ctrl+_, with Shift,
  stays the program's). On our desktop a terminal also accepts Ctrl+Shift for Primary (keycap remaps Command to it
  there), as an input alias that resolves to the same action and displays as the Super form. A Primary
  chord that already uses Shift is Ctrl+Shift+Alt+key in a terminal off the Mac style (temor's
  stand-in for Redo, Save As, Find Previous); one that also uses Alt has no terminal form.
- A press with AltGr held (`KeyInput::alt_gr`) types text: it resolves to nothing and is never in the
  Primary form. Chords never contain AltGr.
- A chord's key is the key with Shift not applied. `Chord::new` writes the symbols Shift makes on a US
  layout (`Shift+=` as `+`, `Shift+/` as `?`, `Shift+-` as `_`) as the symbol with no Shift, as menus write them.
- Text fields keep typing and editing keys without Ctrl, Alt or Super; terminals also keep plain Ctrl, but for the text size keys.
- The overrides format is lines, not TOML: the core stays serde-free; one line per chord; a bad line is
  reported and skipped. TOML or JSON users build `Overrides::from_entries`.
- Person-layer entries for app actions nobody registered stay dormant (`Keymap::dormant_overrides`) and
  go live when the app registers. An app action whose registration was refused is remembered with no
  chords, so the person's chord for it binds at once (over standard actions too, never over a reserved
  chord).
- Standard actions have a `Reach`: `Universal` (every app, never forgone) or `IfOffered` (an app that
  lacks the feature may forgo it with `Registration::forgo` and take the chord). The table is
  `StandardAction::reach` in `reach.rs`; a forgone action is unbound in that app's keymap whatever the
  system or the person says.
- A registration row may carry a fallback default, live in each context where the first default has no
  form. Open: the fallback is not tried when the first default has a form but collides; that row is
  refused.

- Window tiling (Tile Left, Right, Top, Bottom, the four quarters, Fill, Center, Return to Previous Size)
  follows Sequoia's Window > Move & Resize menu. Apple's keys are Globe(Fn)+Ctrl+Left, Right, Up, Down,
  F, C, R; the quarters have no Apple key (only the "& Quarters" arrangements have, with Shift). Fn is
  not a `Modifier`: a Linux keyboard mostly keeps Fn in its firmware, XKB does not treat a reported
  KEY_FN as a modifier, and macOS itself refuses Globe in custom shortcuts. So the Mac-style table
  (macOS and our desktop) swaps Fn for Alt: Ctrl+Alt+Left, Right, Up, Down for the halves,
  Ctrl+Alt+F Fill, Ctrl+Alt+C Center, Ctrl+Alt+R Return to Previous Size, and the quarters take the
  Ctrl+Alt+U, I, J, K square that Rectangle and Magnet use. The Ctrl-arrow Space and Mission
  Control keys stay as they are. The Ctrl-based base (Windows, GNOME, Plasma, other) binds the keys
  all three desktops share: Super+Left and Super+Right tile, Super+Up fills; the rest are unbound
  with a reason, because Ctrl+Alt+arrows are the Space keys there. The name is `Center` (US
  spelling, as `Minimize` and `ShowColors`). The tiles are `Universal`: the desktop does them.
  These Ctrl+Alt keys are desktop keys: as plain-Ctrl chords an app in a terminal context would not
  see them, which is right because the compositor takes them first.

## Sources

- KDE: reads the `[Shortcuts]` group of kdeglobals (KStandardShortcut names to standard actions).
  The group holds only what the person changed, so a missing file is the conventions.
- GNOME: there is no portable settings file. GTK's editing shortcuts are fixed; the window manager,
  shell and media keys live in dconf, which is a binary database. `GnomeSource` reads a text dump
  (`dconf dump /`, saved by the caller). Gap: nothing runs `dconf dump` or reads
  `~/.config/dconf/user` directly; the dconf database format could be parsed directly, or the caller
  can shell out.
- macOS has three plist sources, all readable by a pure plist parser with no ObjC:
  1. `NSUserKeyEquivalents` (System Settings > Keyboard > App Shortcuts), in
     `~/Library/Preferences/.GlobalPreferences.plist` and each app's domain plist. Keyed by menu item
     title (localised: "Copy", "Save As..."); values like `@$z` (`@` Command, `$` Shift, `~` Option,
     `^` Control). `chordkit-macos` has the value parser (`parse_key_equivalent`), an English
     title table and `import_user_key_equivalents`; the program that reads the plist supplies titles per
     locale.
  2. `AppleSymbolicHotKeys` in `~/Library/Preferences/com.apple.symbolichotkeys.plist`: system shortcuts
     (Spotlight, Mission Control, screenshots, input source), each with a numeric id, enabled flag and
     parameters (character, virtual key code, modifier mask). These are the desktop's reserved chords
     and would feed a macOS reserved list and `StandardAction` bindings.
  3. `~/Library/KeyBindings/DefaultKeyBinding.dict` over AppKit's StandardKeyBinding.dict: keys to
     selectors (`moveWordForward:`, `deleteToBeginningOfLine:`) matching text-editing actions; there is
     no text-editing `StandardAction` yet.
  The standard app chords (Cmd+C, Cmd+S) are conventions carried by the menus, so the conventions table
  supplies them.
- Windows has no central or user-editable keymap: standard chords are built into the common controls,
  app accelerators are per app, and RegisterHotKey hotkeys cannot be listed. The source is the
  conventions table plus the reserved chords (any Win+key, Alt+Tab, Ctrl+Alt+Del, Ctrl+Esc).
- Change notification: the trait is a callback; no source here implements a file watcher (that would
  need a watcher crate in a source crate). The default `watch` never fires.

## Open items

- Condition to close each: names in `chordkit-kde/src/names.rs` come from memory of KStandardShortcut,
  and the GNOME dconf keys from memory of the schemas; verify both against a real kdeglobals and
  `dconf dump /` on a KDE and a GNOME machine.
- Default chords for GNOME screenshots, GNOME and KDE workspace switching, KDE overview and Meta
  display order ("Meta+Ctrl+Alt+Shift") are best-effort; confirm against each desktop's shortcut
  settings page.
- `Platform::meta_as` folds Meta into Super everywhere. Toolkits disagree (Qt's Meta is the OS key;
  GDK's Meta is usually Alt); revisit when quire's real event source is wired.
- The shifted-symbol folding is US-layout only.
- Window tiling in sources: GNOME keys `maximize`, `move-to-center` (wm keybindings) and
  `toggle-tiled-left`, `toggle-tiled-right` (org/gnome/mutter/keybindings) are mapped from memory;
  verify against a real `dconf dump /`. KDE's tiling actions are kwin global shortcuts, not
  KStandardShortcut names, so `chordkit-kde` maps none. The macOS title table maps only the titles that
  are unambiguous in an app's menus (the four quarters, Return to Previous Size); "Left", "Right",
  "Top", "Bottom", "Fill" and "Center" also name text alignment and fill items, so they stay out until
  the importer knows which menu a title came from.
- `StandardAction` has no text-editing actions (word moves, line deletes); add them when a source
  (macOS DefaultKeyBinding) or an app needs them.
- On our desktop in a terminal, Ctrl+Shift+Z is read as Super+Z (Undo), so Redo (Super+Shift+Z)
  arrives only as the real Super chord; if keycap remaps Command+Shift+Z to Ctrl+Shift+Alt+Z, add
  that as an alias form.
