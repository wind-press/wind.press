import { createResolver } from 'nuxt/kit'
import { parseMdc } from './helpers/mdc-parser.mjs'
import yaml from '@rollup/plugin-yaml'

const { resolve } = createResolver(import.meta.url)

// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  modules: [
    '@nuxt/ui-pro',
    'nuxt-content-twoslash',
    '@nuxtjs/sitemap',
    '@nuxt/content',
    '@nuxt/image',
    '@nuxt/eslint',
    '@nuxt/scripts',
    '@vueuse/nuxt',
    'nuxt-og-image',
    'motion-v/nuxt',
    'nuxt-llms',
    '@nuxthub/core',
    'nuxt-umami',
  ],
  $development: {
    site: {
      url: 'http://localhost:3000'
    }
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
    enabled: true
  },
  app: {
    pageTransition: false,
    layoutTransition: false
  },
  css: ['~/assets/css/main.css'],
  colorMode: {
    preference: 'system', // default value of $colorMode.preference
    fallback: 'light', // fallback value if not system preference found
  },
  content: {
    build: {
      markdown: {
        highlight: {
          theme: {
            default: 'material-theme-lighter',
            dark: 'material-theme-palenight'
          },
          langs: [
            'sql',
            'diff',
            'ini',
            'html',
            'css',
            'postcss',
            'js',
            'php',
          ],
        }
      },
    },
    preview: {
      api: 'https://api.nuxt.studio'
    }
  },
  mdc: {
    highlight: {
      noApiRoute: false
    },
  },
  ui: {
    theme: {
      colors: ['primary', 'secondary', 'info', 'success', 'warning', 'error', 'important']
    }
  },
  routeRules: {
    // Pre-render
    '/': { prerender: true },
    '/blog/rss.xml': { prerender: true },
    '/404.html': { prerender: true },
    // Redirects
    '/docs': { redirect: '/docs/getting-started/introduction', prerender: false },
    '/docs/getting-started': { redirect: '/docs/getting-started/introduction', prerender: false },
    '/docs/getting-started/migration': { redirect: '/docs/getting-started/migration/yabe-siul', prerender: false },
    '/docs/guide/concepts': { redirect: '/docs/guide/concepts/simple-file-system', prerender: false },
    '/docs/guide/configuration': { redirect: '/docs/guide/configuration/tailwind-version', prerender: false },
    '/docs/guide/configuration/tw-3': { redirect: '/docs/guide/configuration/tw-3/file-main-css', prerender: false },
    '/docs/guide/configuration/tw-4': { redirect: '/docs/guide/configuration/tw-4/file-main-css', prerender: false },
    '/docs/examples/templates-blocks': { redirect: '/docs/examples/templates-blocks/tailwindplus', prerender: false },
    '/docs/examples/custom-integrations': { redirect: '/docs/examples/custom-integrations/custom-theme', prerender: false },
    '/pricing': { redirect: '/#pricing', prerender: false },

    // Redirects 
    '/go': { redirect: '/', prerender: false },
    '/go/github': { redirect: 'https://github.com/wind-press/windpress', prerender: false },
    '/go/discord': { redirect: 'https://discord.gg/fjsB83XdFw', prerender: false },
    '/go/facebook': { redirect: 'https://www.facebook.com/groups/1142662969627943', prerender: false },
    '/go/bricks': { redirect: 'https://bricksbuilder.io/', prerender: false },
    '/go/breakdance': { redirect: 'https://breakdance.com/ref/165/', prerender: false },
    '/go/builderius': { redirect: 'https://builderius.io/?referral=afdfca82c8', prerender: false },
    '/go/livecanvas': { redirect: 'https://livecanvas.com/?ref=4008', prerender: false },
    '/go/ads': { redirect: 'https://ko-fi.com/post/WindPress-Bronze-Sponsor-U7U8143WIP', prerender: false },
    '/go/sponsor': { redirect: 'https://ko-fi.com/Q5Q75XSF7?utm_source=windpress_website', prerender: false },
    '/go/ticket': { redirect: 'https://rosua.org/support-portal', prerender: false },
  },
  // sourcemap: true,
  future: {
    compatibilityVersion: 4
  },
  compatibilityDate: '2024-07-18',
  nitro: {
    prerender: {
      crawlLinks: true,
      ignore: [
        route => route.startsWith('/modules'),
        route => route.startsWith('/enterprise'),
        route => route.startsWith('/deploy'),
        route => route.startsWith('/video-courses'),
        route => route.startsWith('/design-kit'),
        route => route.startsWith('/team'),
        // route => route.startsWith('/docs'),
      ],
      autoSubfolderIndex: false,
      routes: [
        '/',
        '/docs',
        // '/sitemap.xml', // Temp while waiting for hosting on Nuxt Hub
        // '/blog/rss.xml', // Temp while waiting for hosting on Nuxt Hub
        // ...globSync('./content/**/*.md*')
        //   .map(path => path
        //     .slice(7, -3)
        //     .replace(/\d+\./g, '')
        //     .replace(/index/g, '/')
        //     .replace(/\\/g, '/')
        //     .replace(/\/+/g, '/')
        //     .replace(/\/$/, '')
        //   ),
        // '/go',
      ]
    },
    cloudflare: {
      pages: {
        routes: {
          exclude: [
            // '/docs/*'
          ]
        }
      }
    }
  },
  hub: {
    cache: true,
    database: true,
  },
  typescript: {
    strict: false
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
        commaDangle: 'never'
      }
    }
  },
  icon: {
    customCollections: [{
      prefix: 'custom',
      dir: resolve('./app/assets/icons')
    }],
    clientBundle: {
      scan: true,
      includeCustomCollections: true
    },
    provider: 'iconify'
  },
  image: {
    format: ['webp', 'jpeg', 'jpg', 'png', 'svg'],
    // provider: 'cloudflare',
    // cloudflare: {
    //   baseURL: 'https://wind.press'
    // },
    // ipx: {
    //   baseURL: 'https://ipx.nuxt.com'
    // }
  },
  llms: {
    domain: 'https://wind.press',
    title: 'WindPress Docs',
    description: 'WindPress is a plugin that integrates Tailwind CSS with WordPress. It can be used with page builders, themes, and plugins in any WordPress environment, including shared hosting.',
    full: {
      title: 'WindPress Docs',
      description: 'The complete WindPress documentation and blog posts written in Markdown (MDC syntax).'
    }
  },
  sitemap: {
    exclude: [
      '/modules',
      '/enterprise',
      '/enterprise/*',
      '/deploy',
      '/video-courses',
      '/design-kit',
      '/team',
    ],
    defaults: {
      lastmod: new Date().toISOString(),
      priority: 0.5,
      changefreq: 'weekly'
    }
  },
  // turnstile: {
  //   siteKey: '0x4AAAAAAAP2vNBsTBT3ucZi'
  // },
  twoslash: {
    floatingVueOptions: {
      classMarkdown: 'prose prose-primary dark:prose-invert'
    },
    // Skip Twoslash in dev to improve performance. Turn this on when you want to explicitly test twoslash in dev.
    enableInDev: false,
    // Do not throw when twoslash fails, the typecheck should be down in github.com/nuxt/nuxt's CI
    throws: false
  },
  umami: {
    id: 'de57ce9c-c391-4dbb-bb1a-ba6ba26b36a4',
    host: 'https://umami.siagian.dev',
    autoTrack: true,
    ignoreLocalhost: true,
    useDirective: true,
    // proxy: 'direct',
  },
  vite: {
    plugins: [
      yaml()
    ],
  }
})
