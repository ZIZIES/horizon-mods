#!/bin/sh
# Stage every mod in mods/ onto pristine Horizon with horizon-mixins and
# cargo check the game. Usage: tools/check.sh [mod ids...] (default: all).
set -eu
ROOT=$(cd "$(dirname "$0")/.." && pwd)
B="$ROOT/build"
CLI="${MIXINS_CLI:-$HOME/.local/share/horizon-launcher/native/tools/3f99a5c60dfa/bin/horizon-mixins}"
rm -rf "$B/bundles"; mkdir -p "$B/bundles"
if [ $# -eq 0 ]; then set -- $(ls "$ROOT/mods"); fi
for id in "$@"; do cp -r "$ROOT/mods/$id/mixins/"* "$B/bundles/"; done
# Official mods from completed/, unpacked in vendor/: staged with their pin set
# to this revision (they use only the stable hook points) so one build takes all.
for v in "$ROOT"/vendor/*/; do
  [ -d "$v" ] || continue
  cp -r "$v" "$B/bundles/"
  sed -i 's/^horizon_revision = .*/horizon_revision = "9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0"/' "$B/bundles/$(basename "$v")/horizon-mixin.toml"
done
"$CLI" plan --workspace "$B/original" --mods "$B/bundles" --out "$B/plan.json" >/dev/null
"$CLI" apply --workspace "$B/original" --mods "$B/bundles" --out "$B/modded" >/dev/null
cd "$B/modded"
cargo check --locked -p horizon-client --bin horizon-client --target-dir "$B/target" 2>&1 | grep -vE '^\s*(Checking|Compiling|Fresh) ' 
