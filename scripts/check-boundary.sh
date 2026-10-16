#!/usr/bin/env bash
# The core is pure: no I/O, runtime, bus or file-watching crate may be reachable from it.
# `cargo tree -i <dep>` exits 101 when the dependency is absent, so we check for OUTPUT.
set -uo pipefail
cd "$(dirname "$0")/.."

EFFECTS="tokio async-std smol futures zbus zvariant notify rustix libc nix mio evdev toml serde_json plist chordkit-file"
RULES=(
  "chordkit: $EFFECTS"
  "chordkit-macos: $EFFECTS"
)
fail=0
for rule in "${RULES[@]}"; do
  crate="${rule%%:*}"
  read -r -a forbidden <<<"${rule#*:}"
  if ! cargo tree -p "$crate" --depth 0 >/dev/null 2>&1; then
    echo "ERROR: cargo tree cannot resolve $crate; the boundary was not checked"
    fail=1
    continue
  fi
  for dep in "${forbidden[@]}"; do
    if cargo tree -p "$crate" -i "$dep" -e normal,build 2>/dev/null | grep -q .; then
      echo "LEAK: $crate depends on $dep"
      fail=1
    fi
  done
  echo "checked $crate against: ${forbidden[*]}"
done
exit "$fail"
