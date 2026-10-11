#!/usr/bin/env bash
# Builds the single-file web release into dist/index.html (and dist/bevsidian-web.zip).
# Requires: rustup target wasm32-unknown-unknown, wasm-bindgen-cli matching Cargo.lock,
# python3, and optionally wasm-opt (binaryen) and zip.
set -euo pipefail
cd "$(dirname "$0")/.."

PROFILE="${PROFILE:-wasm-release}"
cargo build --profile "$PROFILE" --target wasm32-unknown-unknown
WASM="target/wasm32-unknown-unknown/$PROFILE/bevsidian.wasm"

rm -rf target/web-pkg dist
mkdir -p target/web-pkg dist
wasm-bindgen --target web --no-typescript --out-dir target/web-pkg --out-name bevsidian "$WASM"

if command -v wasm-opt >/dev/null; then
  wasm-opt -O3 --enable-bulk-memory --enable-nontrapping-float-to-int --enable-sign-ext \
    --enable-mutable-globals --enable-reference-types --enable-multivalue \
    -o target/web-pkg/bevsidian_bg.wasm target/web-pkg/bevsidian_bg.wasm
else
  echo "wasm-opt not found; skipping size optimisation"
fi

python3 tools/bundle.py target/web-pkg web dist/index.html
if command -v zip >/dev/null; then
  (cd dist && zip -9 -q bevsidian-web.zip index.html)
  echo "wrote dist/bevsidian-web.zip"
fi
