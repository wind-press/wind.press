#!/usr/bin/env bash
set -euo pipefail

# Compile the Rust/Fontations engine to browser WebAssembly and generate the
# ESM glue module consumed by static-to-variable-worker.js.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
OUTPUT_DIR="$PROJECT_ROOT/public/wasm/static-to-variable"
TARGET="wasm32-unknown-unknown"
WASM_BINDGEN_VERSION="0.2.126"

if ! rustup target list --installed | grep -qx "$TARGET"; then
  echo "Missing Rust target: $TARGET"
  echo "Install it with: rustup target add $TARGET"
  exit 1
fi

if ! command -v wasm-bindgen >/dev/null 2>&1; then
  echo "Missing wasm-bindgen-cli $WASM_BINDGEN_VERSION"
  echo "Install it with: cargo install wasm-bindgen-cli --version $WASM_BINDGEN_VERSION --locked"
  exit 1
fi

if ! wasm-bindgen --version | grep -qx "wasm-bindgen $WASM_BINDGEN_VERSION"; then
  echo "wasm-bindgen-cli must be version $WASM_BINDGEN_VERSION to match Cargo.lock"
  exit 1
fi

mkdir -p "$OUTPUT_DIR"
cd "$SCRIPT_DIR"
cargo build --release --target "$TARGET"
wasm-bindgen \
  --target web \
  --out-dir "$OUTPUT_DIR" \
  --out-name static-to-variable \
  "target/$TARGET/release/yabe_static_to_variable.wasm"

echo "Built static-to-variable WASM in $OUTPUT_DIR"
