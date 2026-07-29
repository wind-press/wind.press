#include <cstdint>
#include <woff2/encode.h>
#include <woff2/decode.h>
#include <emscripten.h>

uint32_t total;

// Throttle progress: only call into JS when the integer percentage changes.
// EM_ASM has non-trivial overhead (WASM→JS stack switch); the brotli patch
// calls _progress on every ~1KB chunk, so without throttling a 4MB font
// triggers thousands of EM_ASM calls that dominate runtime.
static uint32_t last_pct = 101;

extern "C" {
    void _progress (uint32_t n) {
        if (n > total) n = total;
        uint32_t pct = total ? (uint32_t)((double)n / total * 100) : 0;
        if (pct > 100) pct = 100;
        if (pct == last_pct) return;
        last_pct = pct;
        EM_ASM(_progress($0), (int)pct);
    }

    size_t ttf_to_woff2 (uint8_t *ttf, size_t n) {
        total = n;
        last_pct = 101;
        size_t output_size = woff2::MaxWOFF2CompressedSize(ttf, n);
        uint8_t *woff2 = new uint8_t[output_size];
        if (!woff2::ConvertTTFToWOFF2(ttf, n, woff2, &output_size)) {
            delete[] woff2;
            return 0;
        }
        EM_ASM(_ptr = $0, woff2);
        return output_size;
    }

    size_t woff2_to_ttf (uint8_t *woff2, size_t n) {
        size_t output_size = std::min(
            woff2::ComputeWOFF2FinalSize(woff2, n),
            woff2::kDefaultMaxSize
        );
        // For decode, progress is based on decompressed bytes written vs the
        // total output size (not the compressed input size, which is much
        // smaller and would make n/total exceed 100%).
        total = output_size;
        last_pct = 101;
        uint8_t *ttf = new uint8_t[output_size];
        woff2::WOFF2MemoryOut out(ttf, output_size);
        if (!woff2::ConvertWOFF2ToTTF(woff2, n, &out)) {
            delete[] ttf;
            return 0;
        }
        EM_ASM(_ptr = $0, ttf);
        return output_size;
    }
}
