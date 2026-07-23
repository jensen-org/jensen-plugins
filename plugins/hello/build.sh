#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

cargo build --release --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/release/hello_plugin.wasm plugin.wasm

echo "built plugin.wasm"
echo "next: jensen publish $(pwd)"
