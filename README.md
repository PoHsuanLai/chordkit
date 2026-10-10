# chordkit

A portable keymap core: actions, chords, a resolver and display, with per-platform sources that read the system's own shortcut settings.

Apps declare their own actions with a portable default chord and handle actions. They never read modifiers or decide Command versus Ctrl: on each OS the keymap follows that OS's own shortcut settings when there are any, and that OS's conventions otherwise.

- `chordkit`: the pure core (no I/O, no async).
- `chordkit-kde`: reads kdeglobals.
- `chordkit-gnome`: reads a dconf dump.
- `chordkit-macos`: the NSUserKeyEquivalents value syntax and menu-title table (no platform calls).

See `ARCHITECTURE.md` for the map, `CONSUMING.md` for how a toolkit and an app use it, and `FINDINGS.md` for open items.
