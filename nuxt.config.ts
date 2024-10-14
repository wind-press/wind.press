import { globSync } from 'glob'; // using glob here but any package you are comfortable with works
// import wasm from '@rollup/plugin-wasm';
// import {
//   transformerNotationDiff,
//   transformerMetaWordHighlight,
//   // ...
// } from '@shikijs/transformers';

// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  site: {
    url: 'https://wind.press',
  },

  extends: ['@nuxt/ui-pro'],

  modules: [
    '@nuxt/content',
    '@nuxt/eslint',
    '@nuxt/fonts',
    '@nuxt/image',
    '@nuxt/ui',
    '@nuxthq/studio',
    '@vueuse/nuxt',
    'nuxt-og-image',
    'nuxt-umami',
    '@nuxtjs/tailwindcss',
    'nuxt-shiki',
  ],

  hooks: {
    // Define `@nuxt/ui` components as global to use them in `.md` (feel free to add those you need)
    'components:extend': (components) => {
      const globals = components
      // .filter(c => [
      //   'UButton',
      //   'Card',
      // ].includes(c.pascalName))

      globals.forEach(c => c.global = true)
    },
  },

  colorMode: {
    disableTransition: true
  },

  nitro: {
    prerender: {
      routes: [
        '/',
        '/sitemap.xml',
        '/docs',
        ...globSync('./content/**/*.md*')
          .map(path => path
            .slice(7, -3)
            .replace(/\d+\./g, '')
            .replace(/\\/g, '/')
            .replace(/index/g, '')
          )
      ],
      crawlLinks: true
    },
    experimental: {
      wasm: true,
    },
  },

  routeRules: {
    '/api/search.json': { prerender: true },
    '/docs': { redirect: '/docs/getting-started', prerender: false }
  },

  devtools: {
    enabled: true
  },

  typescript: {
    strict: false
  },

  future: {
    compatibilityVersion: 4
  },

  eslint: {
    config: {
      stylistic: {
        commaDangle: 'never',
        braceStyle: '1tbs'
      }
    }
  },

  app: {
    head: {
      script: [
        {
          src: 'https://kit.fontawesome.com/ee89d90c81.js',
          crossorigin: 'anonymous'
        },
      ]
    }
  },

  umami: {
    id: 'de57ce9c-c391-4dbb-bb1a-ba6ba26b36a4',
    host: 'https://umami.rosua.org',
    autoTrack: true,
    ignoreLocalhost: true,
    useDirective: true,
    // proxy: 'direct',
  },

  tailwindcss: {
    cssPath: ['./assets/css/tailwind.css', { injectPosition: 'last' }],
    viewer: false,
    exposeConfig: true
  },

  css: [
    './assets/css/main.css',
  ],

  ui: {
    safelistColors: ['sky', 'mint', 'rose', 'amber', 'violet', 'emerald', 'fuchsia', 'indigo', 'lime', 'orange', 'pink', 'purple', 'red', 'teal', 'yellow', 'green', 'blue', 'cyan', 'gray', 'white', 'black'],
  },

  // vite: {
  //   plugins: [wasm()],
  // },

  compatibilityDate: '2024-07-11'
})