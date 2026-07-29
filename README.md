# Yabe Webfont — Documentation & Marketing Site

The documentation and marketing site for [Yabe Webfont](https://wordpress.org/plugins/yabe-webfont/), the GDPR-friendly font plugin for WordPress.

Hosted at **https://yabe-webfont.jooo.si/**.

This is a static site built with [Nuxt](https://nuxt.com), [Nuxt UI Pro](https://ui.nuxt.com/pro), and [Nuxt Content](https://content.nuxt.com). The TTF ↔ WOFF2 converter runs entirely in the browser via [WebAssembly](https://webassembly.org/) — no font data is ever uploaded to a server.

## Features

- Self-host Google Fonts, embed Adobe Fonts, and manage custom fonts docs
- First-class visual builder integration docs (Elementor, Bricks, Oxygen, Breakdance, and more)
- In-browser TTF to WOFF2 / WOFF2 to TTF converter (WASM, fully private)
- Static site generation — no server required to host

## Development

```sh
pnpm install
pnpm dev
```

The app is accessible at http://localhost:3000

## Build

```sh
pnpm build
```

The generated static site is output to `.output/public/`.

## Deploy

The output in `.output/public/` can be deployed to any static hosting provider (Vercel, Netlify, Cloudflare Pages, GitHub Pages, etc.). Set `NUXT_PUBLIC_SITE_URL=https://yabe-webfont.jooo.si` for production.

## TTF to WOFF2 Converter

The converter at `/docs/misc/convert-ttf-woff2` uses the pre-built WebAssembly module from the open-source [My Font Converter](https://github.com/LibreService/my_font_converter) project, which compiles Google's [woff2](https://github.com/google/woff2) library with [emscripten](https://emscripten.org/).

The WASM glue code and binary live in `public/wasm/`:

- `woff2.js` — emscripten-generated JavaScript glue
- `woff2.wasm` — the compiled WebAssembly binary
- `woff2-worker.js` — a Web Worker that loads the module and performs conversions

The Vue composable `app/composables/useWoff2Converter.ts` and component `app/components/content/FontConverter.vue` provide the UI. All conversion happens inside the Web Worker on the user's device; no font file ever leaves the browser.

## Credits

- Built on top of the [WindPress](https://wind.press) Nuxt site structure.
- TTF/WOFF2 conversion powered by [My Font Converter](https://github.com/LibreService/my_font_converter) (AGPL-3.0-or-later).
- Documentation content migrated from the original [yabe-webfont-docs](https://github.com/orgrosua/yabe-webfont-docs) Astro site.
