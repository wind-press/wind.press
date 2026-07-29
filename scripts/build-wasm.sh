#!/usr/bin/env bash
set -euo pipefail

# Build the woff2 WASM module from the latest google/woff2 commit using an
# isolated emsdk Docker container. The resulting woff2.js + woff2.wasm are
# placed in public/wasm/.
#
# Usage:
#   ./scripts/build-wasm.sh              # build with emsdk:latest
#   ./scripts/build-wasm.sh --tag 3.1.74 # build with a specific emsdk tag
#
# Prerequisites: Docker (or OrbStack) must be running.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SRC_DIR="$ROOT_DIR/wasm-src"
OUTPUT_DIR="$ROOT_DIR/public/wasm"
EMSCK_TAG="latest"

while [[ $# -gt 0 ]]; do
  case $1 in
    --tag)
      EMSCK_TAG="$2"
      shift 2
      ;;
    *)
      echo "Unknown option: $1"
      exit 1
      ;;
  esac
done

echo "Building woff2 WASM with emscripten/emsdk:$EMSCK_TAG"
echo "  Source: $SRC_DIR"
echo "  Output: $OUTPUT_DIR"
echo ""

mkdir -p "$OUTPUT_DIR"

docker run --rm \
  -v "$SRC_DIR:/src:ro" \
  -v "$OUTPUT_DIR:/output" \
  -w /build \
  "emscripten/emsdk:$EMSCK_TAG" \
  bash /src/build.sh

echo ""
echo "✓ WASM build complete. Artifacts in $OUTPUT_DIR:"
ls -lh "$OUTPUT_DIR/woff2.js" "$OUTPUT_DIR/woff2.wasm"
