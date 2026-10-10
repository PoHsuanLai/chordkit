# Findings

## Standing facts

- Primary is Command (Super) on macOS and on our desktop, Ctrl on Windows and other Linux. In a
  terminal, Primary is Super on macOS and our desktop and Ctrl+Shift elsewhere, and plain Ctrl is never
  matched. A Primary chord that already uses Shift has no terminal form off the Mac style, so it is not
  bound there (Redo, Paste and Match Style, Save As, Find Previous).
- A chord's key is the key with Shift not applied. `Chord::new` writes the symbols Shift makes on a US
  layout (`Shift+=` as `+`, `Shift+/` as `?`) as the symbol with no Shift, as menus write them.
- Text fields keep typing and editing keys without Ctrl, Alt or Super; terminals also keep plain Ctrl.
- The overrides format is lines, not TOML: the core stays serde-free; one line per chord; a bad line is
  reported and skipped. TOML or JSON users build `Overrides::from_entries`.
- Person-layer entries for app actions nobody registered stay dormant (`Keymap::dormant_overrides`) and
  go live when the app registers.

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
- `StandardAction` has no text-editing actions (word moves, line deletes); add them when a source
  (macOS DefaultKeyBinding) or an app needs them.
- Terminal Primary chords that already use Shift have no terminal form; if a terminal wants
  Ctrl+Shift+Z for Redo, that needs a Terminal-specific convention row.
