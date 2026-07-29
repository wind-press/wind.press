import { createResolver } from "nuxt/kit";
import { parseMdc } from "./helpers/mdc-parser.mjs";
import yaml from "@rollup/plugin-yaml";

const { resolve } = createResolver(import.meta.url);

// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  modules: [
    "@nuxt/ui-pro",
    "nuxt-content-twoslash",
    "@nuxtjs/sitemap",
    "@nuxt/content",
    "@nuxt/image",
    "@nuxt/eslint",
    "@nuxt/scripts",
    "@vueuse/nuxt",
    "nuxt-og-image",
    "nuxt-llms",
    "nuxt-umami",
  ],
  $development: {
    site: {
      url: "http://localhost:3000",
    },
  },
  $production: {
    site: {
      url: "https://yabe-webfont.jooo.si",
    },
  },
  // $development: {
  //   runtimeConfig: {
  //     public: {
  //       website: {
  //         url: 'http://localhost:3000'
  //       }
  //     }
  //   }
  // },
  devtools: {
    enabled: true,
  },
  app: {
    pageTransition: false,
    layoutTransition: false,
    head: {
      link: [{ rel: "icon", type: "image/svg+xml", href: "/icon.svg" }],
    },
  },
  css: ["~/assets/css/main.css"],
  colorMode: {
    preference: "system", // default value of $colorMode.preference
    fallback: "light", // fallback value if not system preference found
  },
  content: {
    build: {
      markdown: {
        highlight: {
          theme: {
            default: "material-theme-lighter",
            dark: "material-theme-palenight",
          },
          langs: ["sql", "diff", "ini", "html", "css", "postcss", "js", "php"],
        },
      },
    },
    preview: {
      api: "https://api.nuxt.studio",
    },
  },
  mdc: {
    highlight: {
      noApiRoute: false,
    },
  },
  ui: {
    theme: {
      colors: [
        "primary",
        "secondary",
        "info",
        "success",
        "warning",
        "error",
        "important",
      ],
    },
  },
  routeRules: {
    // Pre-Render
    "/": { prerender: true },
    "/404.html": { prerender: true },
    // Redirects
    "/docs": {
      redirect: "/docs/getting-started/introduction",
      prerender: false,
    },
    "/docs/getting-started": {
      redirect: "/docs/getting-started/introduction",
      prerender: false,
    },
    "/docs/fonts": { redirect: "/docs/fonts/custom-font", prerender: false },
    "/docs/integrations": {
      redirect: "/docs/integrations/elementor",
      prerender: false,
    },
    "/docs/misc": {
      redirect: "/docs/misc/convert-ttf-woff2",
      prerender: false,
    },
    "/pricing": { redirect: "/#pricing", prerender: false },
    // Legacy /en/* routes from the old Astro site -> new docs structure
    "/en": { redirect: "/docs/getting-started/introduction", prerender: false },
    "/en/introduction": {
      redirect: "/docs/getting-started/introduction",
      prerender: false,
    },
    "/en/install": {
      redirect: "/docs/getting-started/installation",
      prerender: false,
    },
    "/en/font/custom-font": {
      redirect: "/docs/fonts/custom-font",
      prerender: false,
    },
    "/en/font/google-fonts": {
      redirect: "/docs/fonts/google-fonts",
      prerender: false,
    },
    "/en/font/adobe-fonts": {
      redirect: "/docs/fonts/adobe-fonts",
      prerender: false,
    },
    "/en/misc/convert-ttf-woff2": {
      redirect: "/docs/misc/convert-ttf-woff2",
      prerender: false,
    },
    "/en/misc/no-plugin": {
      redirect: "/docs/misc/no-plugin",
      prerender: false,
    },
    "/en/misc/proxy": { redirect: "/docs/misc", prerender: false },
    "/docs/misc/proxy": { redirect: "/docs/misc", prerender: false },
    "/en/integration/elementor": {
      redirect: "/docs/integrations/elementor",
      prerender: false,
    },
    "/en/integration/oxygen": {
      redirect: "/docs/integrations/oxygen",
      prerender: false,
    },
    "/en/integration/bricks": {
      redirect: "/docs/integrations/bricks",
      prerender: false,
    },
    "/en/integration/gutenberg": {
      redirect: "/docs/integrations/gutenberg",
      prerender: false,
    },
    "/en/integration/breakdance": {
      redirect: "/docs/integrations/breakdance",
      prerender: false,
    },
    "/en/integration/beaver-builder": {
      redirect: "/docs/integrations/beaver-builder",
      prerender: false,
    },
    "/en/integration/builderius": {
      redirect: "/docs/integrations/builderius",
      prerender: false,
    },
    "/en/integration/cwicly": {
      redirect: "/docs/integrations/cwicly",
      prerender: false,
    },
    "/en/integration/classic-editor": {
      redirect: "/docs/integrations/classic-editor",
      prerender: false,
    },
    "/en/integration/developer": {
      redirect: "/docs/integrations/developer",
      prerender: false,
    },
    "/en/integration/generatepress": {
      redirect: "/docs/integrations/generatepress",
      prerender: false,
    },
    "/en/integration/zion-builder": {
      redirect: "/docs/integrations/zion-builder",
      prerender: false,
    },

    // Outbound go/ short links
    "/go": { redirect: "/", prerender: false },
    "/go/github": {
      redirect: "https://github.com/orgrosua/yabe-webfont-docs",
      prerender: false,
    },
    "/go/facebook": {
      redirect: "https://www.facebook.com/groups/1142662969627943",
      prerender: false,
    },
    "/go/sponsor": {
      redirect: "https://ko-fi.com/Q5Q75XSF7",
      prerender: false,
    },
    "/go/ticket": {
      redirect: "https://jooo.si/account?view=support-tickets",
      prerender: false,
    },
    "/go/account": {
      redirect: "https://jooo.si/account",
      prerender: false,
    },
  },
  // sourcemap: true,
  future: {
    compatibilityVersion: 4,
  },
  compatibilityDate: "2024-07-18",
  nitro: {
    preset: "static",
    prerender: {
      crawlLinks: true,
      ignore: [
        // route => route.startsWith('/modules'),
      ],
      autoSubfolderIndex: false,
      routes: ["/", "/docs"],
    },
  },
  typescript: {
    strict: false,
  },
  hooks: {
    // 'content:file:afterParse': async ({ file, content }) => {
    //   if (file.id === 'index/index.yml') {
    //     // @ts-expect-error -- TODO: fix this
    //     for (const tab of content.hero.tabs) {
    //       tab.content = await parseMdc(tab.content)
    //     }
    //     // @ts-expect-error -- TODO: fix this
    //     delete content.meta.body
    //   }
    // }
  },
  eslint: {
    config: {
      stylistic: {
        commaDangle: "never",
      },
    },
  },
  icon: {
    customCollections: [
      {
        prefix: "custom",
        dir: resolve("./app/assets/icons"),
      },
    ],
    clientBundle: {
      scan: true,
      includeCustomCollections: true,
    },
    provider: "iconify",
  },
  image: {
    format: ["webp", "png", "jpeg", "jpg", "svg"],
    // provider: 'cloudflare',
    // cloudflare: {
    //   baseURL: 'https://yabe-webfont.jooo.si'
    // },
    // ipx: {
    //   baseURL: 'https://ipx.nuxt.com'
    // }
  },
  ogImage: { zeroRuntime: true },
  llms: {
    domain: "https://yabe-webfont.jooo.si",
    title: "Yabe Webfont Docs",
    description:
      "Yabe Webfont is a GDPR-friendly font plugin for WordPress. Import and self-host Google Fonts, Adobe Fonts, and custom fonts, and use them across your favorite visual builders and themes.",
    full: {
      title: "Yabe Webfont Docs",
      description:
        "The complete Yabe Webfont documentation written in Markdown (MDC syntax).",
    },
  },
  sitemap: {
    zeroRuntime: true,
    exclude: [],
    defaults: {
      lastmod: new Date().toISOString(),
      priority: 0.5,
      changefreq: "weekly",
    },
  },
  // turnstile: {
  //   siteKey: '0x4AAAAAAAP2vNBsTBT3ucZi'
  // },
  twoslash: {
    floatingVueOptions: {
      classMarkdown: "prose prose-primary dark:prose-invert",
    },
    // Skip Twoslash in dev to improve performance. Turn this on when you want to explicitly test twoslash in dev.
    enableInDev: false,
    // Do not throw when twoslash fails, the typecheck should be down in github.com/nuxt/nuxt's CI
    throws: false,
  },
  umami: {
    id: "39c25348-6c00-404d-8d46-bc83f76ef3a5",
    host: "https://umami.siagian.dev",
    autoTrack: true,
    ignoreLocalhost: true,
    useDirective: true,
    // proxy: 'direct',
  },
  vite: {
    experimental: {
      // enableNativePlugin: true
    },
    plugins: [yaml()],
  },
});
