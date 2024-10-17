import { globSync } from 'glob'; // using glob here but any package you are comfortable with works

// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
    site: {
        url: 'https://wind.press',
    },

    extends: ['@nuxt/ui-pro'],

    modules: [
        '@nuxt/content',
        '@nuxt/ui',
        '@nuxt/image',
        'nuxt-umami',
        '@nuxt/fonts',
        '@nuxt/eslint',
        '@nuxthq/studio',
        '@nuxtjs/tailwindcss',
        'nuxt-shiki',
        '@nuxt/icon',
        '@vueuse/nuxt',
        'nuxt-og-image',
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

    nitro: {
        prerender: {
            routes: [
                '/',
                '/docs',
                '/sitemap.xml',
                ...globSync('./content/**/*.md*')
                    .map(path => path
                        .slice(7, -3)
                        .replace(/\d+\./g, '')
                        .replace(/\\/g, '/')
                        .replace(/index/g, '/')
                    ),
            ],
            crawlLinks: true
        },
        experimental: {
            wasm: true,
        },
    },

    routeRules: {
        '/api/search.json': { prerender: true },
        '/docs': { redirect: '/docs/getting-started', prerender: false },
    },

    content: {
        defaultLocale: 'en',
        locales: ['en', 'es', 'id'],
        // navigation: {
        //     fields: ['author', 'publishedAt']
        // }
    },

    devtools: {
        enabled: true
    },

    future: {
        compatibilityVersion: 4
    },

    typescript: {
        strict: false
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
            script: []
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

    colorMode: {
        preference: 'system', // default value of $colorMode.preference
        fallback: 'dark', // fallback value if not system preference found
    },

    css: [
        './assets/css/main.css',
    ],

    ui: {
        safelistColors: ['sky', 'mint', 'rose', 'amber', 'violet', 'emerald', 'fuchsia', 'indigo', 'lime', 'orange', 'pink', 'purple', 'red', 'teal', 'yellow', 'green', 'blue', 'cyan', 'gray', 'white', 'black'],
    },

    icon: {},

    vite: {
        plugins: []
    },

    compatibilityDate: '2024-07-11'
})