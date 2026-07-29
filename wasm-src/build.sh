#!/usr/bin/env bash
set -euo pipefail

# This script runs inside the emscripten/emsdk Docker container.
# It clones the latest google/woff2, applies patches, builds the libraries,
# and compiles the WASM module. The resulting woff2.js + woff2.wasm are
# copied to OUTPUT_DIR.
#
# Environment variables (with defaults for Docker volume mounts):
#   SRC_DIR    — directory containing api.cpp, brotli_patch, woff2_patch
#   OUTPUT_DIR — destination for woff2.js + woff2.wasm
#   BUILD_DIR  — temporary build workspace

SRC_DIR="${SRC_DIR:-/src}"
OUTPUT_DIR="${OUTPUT_DIR:-/output}"
BUILD_DIR="${BUILD_DIR:-/build}"
NPROC=$(nproc 2>/dev/null || echo 4)

mkdir -p "$OUTPUT_DIR" "$BUILD_DIR"

echo "=== Cloning google/woff2 (latest) ==="
cd "$BUILD_DIR"
rm -rf woff2
git clone --depth 1 https://github.com/google/woff2.git
cd woff2
git submodule update --init --recursive

echo ""
echo "=== Applying patches ==="
cd brotli
if git apply --check "$SRC_DIR/brotli_patch" 2>/dev/null; then
  git apply "$SRC_DIR/brotli_patch"
  echo "  brotli patch applied"
else
  echo "  WARNING: brotli_patch does not apply cleanly — attempting with --ignore-whitespace"
  git apply --ignore-whitespace "$SRC_DIR/brotli_patch" || {
    echo "  ERROR: brotli_patch failed to apply. The brotli source may have changed."
    echo "  The _progress callback will be missing — progress reporting won't work."
    echo "  Continuing without the brotli patch..."
  }
fi
cd ..

if git apply --check "$SRC_DIR/woff2_patch" 2>/dev/null; then
  git apply "$SRC_DIR/woff2_patch"
  echo "  woff2 patch applied"
else
  echo "  WARNING: woff2_patch does not apply cleanly — attempting fallback"
  git apply --ignore-whitespace "$SRC_DIR/woff2_patch" || {
    echo "  ERROR: woff2_patch failed to apply."
    exit 1
  }
fi

echo ""
echo "=== Building brotli (optimized) ==="
emcmake cmake brotli -B build/brotli \
  -DBROTLI_DISABLE_TESTS:BOOL=ON \
  -DCMAKE_BUILD_TYPE=Release \
  -DCMAKE_C_FLAGS_RELEASE="-O3 -flto -msimd128" \
  -DCMAKE_CXX_FLAGS_RELEASE="-O3 -flto -msimd128" \
  -DCMAKE_INSTALL_PREFIX:PATH=/usr/local
make DESTDIR="$BUILD_DIR/sysroot" -C build/brotli install -j"$NPROC"

echo ""
echo "=== Building woff2 (optimized) ==="
emcmake cmake . -B build/woff2 \
  -DCMAKE_FIND_ROOT_PATH:PATH="$BUILD_DIR/sysroot/usr/local" \
  -DBUILD_SHARED_LIBS=OFF \
  -DCMAKE_BUILD_TYPE=Release \
  -DCMAKE_C_FLAGS_RELEASE="-O3 -flto -msimd128" \
  -DCMAKE_CXX_FLAGS_RELEASE="-O3 -flto -msimd128" \
  -DCMAKE_INSTALL_PREFIX:PATH=/usr/local
make DESTDIR="$BUILD_DIR/sysroot" -C build/woff2 install -j"$NPROC"

echo ""
echo "=== Compiling WASM (optimized) ==="
# Link the static archives directly (not -l flags) to avoid the linker
# picking up brotli's shared libraries (.so) which would cause runtime
# fetches of libbrotlienc.so etc. Brotli's static targets are named
# libbrotli*-static.a; woff2 was built with BUILD_SHARED_LIBS=OFF so
# its archives are already static (.a).
#
# Optimization flags:
#   -O3       — maximum speed optimization (vs -O2 in the original)
#   -flto     — link-time optimization, inlines across library boundaries
#   -msimd128 — enable WASM SIMD 128-bit intrinsics; brotli's hot loops
#               (histogram, bit-cost, entropy) benefit from vectorization
#   -s ASSERTIONS=0 — disable runtime assertions
LIB="$BUILD_DIR/sysroot/usr/local/lib"
em++ \
  -O3 \
  -flto \
  -msimd128 \
  -s ALLOW_MEMORY_GROWTH=1 \
  -s ASSERTIONS=0 \
  -s EXPORTED_FUNCTIONS=_ttf_to_woff2,_woff2_to_ttf,_malloc,_free \
  -s EXPORTED_RUNTIME_METHODS='["ccall","HEAP8","HEAPU8","writeArrayToMemory"]' \
  -I "$BUILD_DIR/sysroot/usr/local/include" \
  -o "$OUTPUT_DIR/woff2.js" \
  "$SRC_DIR/api.cpp" \
  "$LIB/libwoff2enc.a" \
  "$LIB/libwoff2dec.a" \
  "$LIB/libwoff2common.a" \
  "$LIB/libbrotlienc-static.a" \
  "$LIB/libbrotlidec-static.a" \
  "$LIB/libbrotlicommon-static.a"

echo ""
echo "=== Build complete ==="
ls -lh "$OUTPUT_DIR/woff2.js" "$OUTPUT_DIR/woff2.wasm"
echo ""
echo "WASM artifacts written to $OUTPUT_DIR"
