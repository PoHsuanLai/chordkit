# Consuming chordkit

Two roles use chordkit: a **toolkit** (quire) that turns key events into actions, and an **app**
that declares and handles actions. Neither reads modifiers or decides Command versus Ctrl.

## The toolkit

### 1. Pick the platform once

```rust
use chordkit::{detect_process, Platform};

let platform: Platform = detect_process(); // CHORDKIT_PLATFORM overrides; tests use `detect(os, lookup)`
```

### 2. Load a keymap from a source

```rust
use chordkit::{KeymapSource, ConventionSource, Desktop, Platform};
use chordkit_kde::KdeSource;

let source: Box<dyn KeymapSource> = match platform {
    Platform::Linux { desktop: Desktop::Kde } => Box::new(KdeSource::new(config_dir.join("kdeglobals"))),
    _ => Box::new(ConventionSource),
};
let mut keymap = source.load(platform)?;
```

Then lay the person's changes over it and keep the problems for a settings screen:

```rust
let (overrides, bad_lines) = chordkit::Overrides::parse(&std::fs::read_to_string(path)?);
let (keymap, refused) = keymap.with_overrides(&overrides);
```

Reload (and re-register apps) when the source says something changed:

```rust
let _watch = source.watch(Box::new(move || tell_main_loop_to_reload()))?; // keep `_watch` alive
```

### 3. Convert the toolkit's key event to `KeyInput`

`KeyInput` is the physical modifiers as the toolkit reports them plus the logical key with Shift
not applied (winit's `key_without_modifiers`, GTK's keyval of the unshifted level, Qt's key).

```rust
use chordkit::{AltGr, Key, KeyInput, Modifier, Modifiers, NamedKey};

fn to_input(event: &ToolkitKeyEvent) -> Option<KeyInput> {
    let mut mods = Modifiers::NONE;
    if event.ctrl()  { mods = mods.with(Modifier::Ctrl); }
    if event.alt()   { mods = mods.with(Modifier::Alt); }
    if event.shift() { mods = mods.with(Modifier::Shift); }
    if event.logo()  { mods = mods.with(Modifier::Super); }   // or Modifier::Meta for Qt's Meta
    let alt_gr = if event.alt_graph() { AltGr::Held } else { AltGr::Released }; // keyboard-types' ALT_GRAPH
    let key = match event.key_without_modifiers() {
        ToolkitKey::Character(c) => Key::character(c),
        ToolkitKey::Space => Key::Named(NamedKey::Space),
        ToolkitKey::Escape => Key::Named(NamedKey::Escape),
        ToolkitKey::F(n) => Key::function(n)?,
        // ... the rest of NamedKey
        _ => return None,
    };
    Some(KeyInput::new(mods, key).with_alt_gr(alt_gr))
}
```

Pass the AltGr flag whenever the toolkit has one (keyboard-types' `ALT_GRAPH`, GTK's level-3 shift).
A press with AltGr held types a character: `resolve` returns `None` and `is_primary_form` is false,
so it never matches a Ctrl or Alt chord. Windows reports AltGr as Ctrl+Alt plus the flag; leave Ctrl
and Alt as the toolkit has them, chordkit drops them. Without the flag, Ctrl+Alt+T is an ordinary
Ctrl+Alt chord. Chords themselves never contain AltGr.

Pass modifiers as the hardware has them. If a toolkit swaps Ctrl and Command (Qt on macOS does), undo
the swap in `to_input`; chordkit assumes physical keys.

### 4. Resolve, and hand the app an action

```rust
use chordkit::Context;

let context = match focus {
    Focus::TextField => Context::TextEntry,
    Focus::Terminal => Context::Terminal,
    _ => Context::Normal,
};
if let Some(action) = keymap.resolve(&to_input(&event)?, context) {
    app.on_action(action); // `Action::Standard(..)` or `Action::App(..)`; the event is consumed
}
```

If `resolve` is `None`, deliver the key to the focused widget as usual.

### 5. Show shortcuts in menus

```rust
let shown: Vec<String> = keymap
    .chords_of(&Action::Standard(StandardAction::Copy))
    .iter()
    .map(|chord| chord.display(keymap.platform()))
    .collect(); // "⌘C" on a Mac or our desktop, "Ctrl+C" elsewhere
```

`keymap.bindings(context)` lists every live (action, chord) for a settings screen.

### 6. Terminals

A terminal (temor) passes `Context::Terminal` and drops its own keymap. The rules:

- Primary in a terminal is Super on macOS and on our desktop, and Ctrl+Shift elsewhere, so plain Ctrl
  stays the program's.
- On our desktop the compositor (keycap) remaps Command to Ctrl+Shift for terminal windows, so
  `resolve` accepts both Super+D and Ctrl+Shift+D in `Context::Terminal` and finds the same action;
  menus show the Super form (`⌘D`). Other platforms have no alias.
- A Primary chord that already uses Shift (Redo, Save As) is Ctrl+Shift+Alt+key in a terminal on the
  Ctrl+Shift platforms (Windows, KDE, GNOME, other Linux), e.g. Primary+Shift+Z is Ctrl+Shift+Alt+Z. A
  default that also uses Alt has no terminal form.
- When `resolve` finds nothing, ask `keymap.is_primary_form(&input, Context::Terminal)`. If it is true
  the chord is in the Primary form (Super+key, plus Ctrl+Shift+key on our desktop; Ctrl+Shift+key on
  the other Ctrl+Shift platforms; extra modifiers allowed) with a letter, digit, punctuation or
  symbol key, whether or not anything is bound. Drop it, as Ghostty does, instead of sending a
  control byte to the program. Plain Ctrl+D is false and goes to the program.

```rust
match keymap.resolve(&input, Context::Terminal) {
    Some(action) => app.on_action(action),
    None if keymap.is_primary_form(&input, Context::Terminal) => {} // swallowed
    None => pty.send(encode(&event)),
}
```

## The app

An app declares its actions with a portable default chord, registers them, and handles actions.

```rust
use chordkit::{Action, AppAction, AppId, DefaultChord, StandardAction};

let mail = AppId::new("mail")?;
let compose = AppAction::new("mail.compose")?;
let tab3 = AppAction::new("mail.tab-3")?;

keymap.register(&mail, &[
    (compose.clone(), "Primary+Shift+N".parse::<DefaultChord>()?),
    (tab3.clone(),    "Primary+3".parse()?),   // apps own Primary+1..9
])?; // Err(Conflict) says in plain words why a chord was refused

fn on_action(action: Action) {
    match action {
        Action::Standard(StandardAction::Save) => save(),
        Action::App(a) if a.id() == "mail.compose" => compose(),
        _ => {}
    }
}
```

Rules for apps: write defaults with `Primary` (never "Cmd" or "Ctrl"); never register a standard
action's chord (Save is `Standard(Save)`, handle it); never define a chord type. A person's own
change to your action is stored under its id (`mail.compose = Primary+Alt+N` in their overrides) and
wins over your default (see below). Register at startup, and again after a reload.

### Standard actions an app does not offer

Every standard action has a `Reach` (`StandardAction::reach`). `Reach::Universal` actions are
honoured by every app and keep their chord: copy, cut, paste, select all, undo, redo, find (and next,
previous), settings, help, cancel, quit, close, minimise, hide, full screen, the app and window
switchers and every desktop action (Spaces, screenshots, lock, log out). `Reach::IfOffered` actions
mean something only where the app has the feature: paste and match style, use selection for find,
jump to selection, bold, italic, underline, show fonts, show colours, bigger, smaller, new, open, save,
save as, print, page setup, toggle toolbar, toggle sidebar, quick look, reveal.

An app without the feature gives the action up and may then use its chord. Build the registration with
`Registration`; the forgone action stops holding its chord in that app's keymap (every other app, with
its own keymap, still sees it). Forgoing a universal action is refused (`Conflict::Universal`), and a
reserved platform chord is refused as before.

```rust
use chordkit::{Registration, StandardAction};

let registration = Registration::new(&temor)
    .forgo(StandardAction::ShowFonts)                 // temor has no font panel
    .action(new_tab, "Primary+T".parse()?)            // so Primary+T is free
    .action_with_fallback(
        equalize,
        "Primary+Shift+Alt+E".parse()?,               // no terminal form on a Ctrl+Shift platform
        "Primary+Alt+Q".parse()?,                     // used in the contexts where the first has none
    );
keymap.register_with(&registration)?;
```

A fallback serves each context where the first default resolves to nothing; a chord the keymap refuses
is checked on whichever chord is live in each context. `register(app, rows)` is the same without
forgoing or fallbacks.

### Overrides and refused actions

A person's chord for your action wins over your default. It also revives an action whose registration
was refused (for a default the platform reserves or a standard action holds): the action had no chord,
and the person's chord binds, even on a chord a standard action holds (that action loses it). Only a
chord the platform reserves is refused, with a reason in `with_overrides`' problems.
`dormant_overrides()` lists only changes to actions nobody registered.

## A source in another repo (keycap)

```rust
use chordkit::{Keymap, KeymapSource, Platform, SourceError, SystemBinding, Watch, ChangeCallback};

struct OurDesktopSource { /* socket to the daemon */ }

impl KeymapSource for OurDesktopSource {
    fn load(&self, platform: Platform) -> Result<Keymap, SourceError> {
        let bindings: Vec<SystemBinding> = self.ask_daemon()?;       // chords per StandardAction
        Ok(Keymap::conventional(platform).with_system(bindings))
    }
    fn watch(&self, on_change: ChangeCallback) -> Result<Watch, SourceError> {
        let subscription = self.subscribe(on_change)?;               // a guard that unsubscribes on drop
        Ok(Watch::holding(subscription))
    }
}
```

A source returns bindings only for what the system has actually set; everything else stays at the
platform's conventions. The trait never names a runtime: `watch`'s callback may be called from any
thread, and the program forwards it into its own loop.
