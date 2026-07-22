#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

cargo build --release --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/release/hello_plugin.wasm plugin.wasm

echo "built plugin.wasm next to manifest.json"
echo "install: copy this directory to \$JENSEN_STATE_DIR/plugins/dev.jensen.hello/"
