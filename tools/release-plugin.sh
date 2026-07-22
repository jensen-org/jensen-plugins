#!/usr/bin/env bash
set -euo pipefail

# Assemble a plugin release from its source directory: build the wasm (when the manifest declares one),
# zip the UI (when it declares one), copy every theme tokens file it references, pin each artifact's
# sha256 into a `dist` block, and print the release manifest's own sha256 (the value for the registry
# entry). A theme-only plugin has no wasm and no UI, only tokens assets.

if ! command -v jq >/dev/null; then echo "need jq" >&2; exit 1; fi
if ! command -v shasum >/dev/null; then echo "need shasum" >&2; exit 1; fi

PLUGIN_DIR=${1:?plugin dir}
PLUGIN_DIR=$(cd "$PLUGIN_DIR" && pwd)
OUT=${2:-"$PLUGIN_DIR/release"}
MANIFEST="$PLUGIN_DIR/manifest.json"

sha256() { shasum -a 256 "$1" | awk '{print $1}'; }

rm -rf "$OUT"
mkdir -p "$OUT"

dist='{}'

wasm_entry=$(jq -r '.entry.wasm // empty' "$MANIFEST")
if [ -n "$wasm_entry" ]; then
  ( cd "$PLUGIN_DIR" && cargo build -q --release --target wasm32-unknown-unknown )
  wasm=$(ls "$PLUGIN_DIR"/target/wasm32-unknown-unknown/release/*.wasm | head -1)
  cp "$wasm" "$OUT/plugin.wasm"
  dist=$(echo "$dist" | jq --arg s "$(sha256 "$OUT/plugin.wasm")" '.wasm = {asset:"plugin.wasm", sha256:$s}')
fi

ui_entry=$(jq -r '.entry.ui // empty' "$MANIFEST")
if [ -n "$ui_entry" ] && [ -d "$PLUGIN_DIR/ui" ]; then
  ( cd "$PLUGIN_DIR/ui" && zip -qr "$OUT/ui.zip" . )
  dist=$(echo "$dist" | jq --arg s "$(sha256 "$OUT/ui.zip")" '.ui = {asset:"ui.zip", sha256:$s}')
fi

while IFS= read -r asset; do
  [ -z "$asset" ] && continue
  src="$PLUGIN_DIR/$asset"
  if [ ! -f "$src" ]; then echo "missing tokens file: $asset" >&2; exit 1; fi
  mkdir -p "$OUT/$(dirname "$asset")"
  cp "$src" "$OUT/$asset"
  dist=$(echo "$dist" | jq --arg a "$asset" --arg s "$(sha256 "$OUT/$asset")" \
    '.assets = ((.assets // []) + [{asset:$a, sha256:$s}])')
done < <(jq -r '.contributes.themes[]?.tokensFile // empty' "$MANIFEST")

jq --argjson dist "$dist" '. + {dist: $dist}' "$MANIFEST" > "$OUT/manifest.json"

echo "release assembled in $OUT"
ls -1 "$OUT"
echo "manifest sha256 (put this in the registry entry):"
sha256 "$OUT/manifest.json"
