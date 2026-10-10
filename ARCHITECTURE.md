# Architecture

chordkit is the one place keymap nouns live: actions, modifiers, chords, platforms, conventions,
the resolver. `CONVENTIONS.md` holds the shared rules (with the deviations listed in section 5),
`FINDINGS.md` the open items and standing facts, `CONSUMING.md` how a toolkit and an app use it.

## 1. Crate map

| Crate | Path | Purpose | May depend on |
|---|---|---|---|
| `chordkit` | `crates/chordkit` | the pure core | `thiserror`; `serde` behind the `serde` feature |
| `chordkit-kde` | `crates/chordkit-kde` | `KdeSource`, `parse_kdeglobals` | `chordkit` |
| `chordkit-gnome` | `crates/chordkit-gnome` | `GnomeSource`, `parse_dconf_dump`, `parse_accelerator` | `chordkit` |
| `chordkit-macos` | `crates/chordkit-macos` | `parse_key_equivalent`, `ENGLISH` menu titles, `import_user_key_equivalents` | `chordkit` |

Edges: every source crate depends on `chordkit` only; `chordkit` depends on no workspace crate.
`scripts/check-boundary.sh` asserts the core and `chordkit-macos` reach no I/O, runtime, bus,
file-watching or plist crate. The only file reads are in `KdeSource::load` and `GnomeSource::load`
(a path injected by the caller). Our desktop's source is not here: keycap implements
`KeymapSource` in its own repo.

## 2. Core modules (`crates/chordkit/src`)

Layers, lowest first; a module imports from its own layer or a lower one.

| Layer | Modules | Holds |
|---|---|---|
| L0 vocabulary | `modifier`, `key`, `standard_action`, `reach`, `action` | `Modifier`, `Modifiers`, `Key`, `NamedKey`, `StandardAction`, `SpaceNumber`, `Reach`, `Action`, `AppAction`, `AppId` |
| L1 chords | `chord`, `chord_text`, `platform`, `detect` | `Chord`, `DefaultChord`, `PrimaryUse`, parse and display, `Platform`, `Desktop`, `Context`, `detect` |
| L2 data | `convention/`, `reserved` | per-platform default chords, per-platform reserved chords |
| L3 input | `key_input` | `KeyInput`, `AltGr`, `KeyInput::normalise`, `normalise_in` |
| L4 keymap | `keymap/` (`layers`, `effective`, `resolved`, `register`, `registration`, `conflict`), `overrides`, `source`, `keyfile` | `Keymap`, `Registration`, `Conflict`, `Overrides`, `KeymapSource`, `SystemBinding` |
| edge | `serde_support` (feature `serde`) | string forms for stored types |

## 3. The one-home table

| Concept | Home |
|---|---|
| what Primary means on a platform and context | `Platform::primary` |
| Meta folding, shifted-symbol folding | `Chord::new`, `Platform::meta_as`, `KeyInput::normalise` |
| default chords | `convention/` (one file per platform family) |
| chords an app may not take | `reserved.rs` |
| which binding wins | `keymap/effective.rs` (layers: convention, app, system, person, forgone) |
| which standard actions an app may give up | `StandardAction::reach` in `reach.rs`; `Registration::forgo` |
| a fallback default per context | `Binding` in `keymap/layers.rs` |
| Primary input alias in a terminal (our desktop: Ctrl+Shift), Primary form test | `Platform::primary_alias`, `KeyInput::normalise_in`, `Keymap::is_primary_form` |
| which keys a text field or terminal keeps | `Chord::belongs_to_text_field`, `Chord::is_plain_ctrl`, applied in `keymap/effective.rs` |
| chord text and per-platform display | `chord_text.rs` |
| the person's overrides format | `overrides.rs` |
| `[group]`/`key=value` reading | `keyfile.rs` |

## 4. Recipes

Add a standard action: add the variant (documented by what it does, not which key) and its slug to
`NAMED` in `standard_action.rs`, add a row to every table in `convention/` (a chord or `Unbound`
with a reason; the coverage test fails otherwise), bump the count in the two slug tests.

Add a platform family: add the `Platform`/`Desktop` variant, its slug, `Platform::primary` and
`meta_as` arms, a display style in `chord_text.rs`, a reserved list, and a convention file.

Add a source: a new crate depending on `chordkit`; parse into `SystemImport`; implement
`KeymapSource::load` as `Keymap::conventional(platform).with_system(import.bindings)`.

## 5. Repo rules

- Gate: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`,
  `cargo test --workspace --all-features`, `./scripts/check-boundary.sh`.
- No `unsafe` anywhere (`unsafe_code = "forbid"`).
- Deviation from CONVENTIONS section 3: `StandardAction` and `Action` are `#[non_exhaustive]`,
  because the standard set grows and a consumer outside this crate must not break on a new meaning.
- Tables (conventions, reserved chords, names) are text rows parsed on use; a test reads every row.
