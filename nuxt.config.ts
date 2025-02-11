import { globSync } from 'glob'; // using glob here but any package you are comfortable with works

// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
    site: {
        url: 'https://wind.press',
        name: 'WindPress',
        description: 'Tailwind CSS integration for WordPress',
        locale: 'en',
    },

    extends: ['@nuxt/ui-pro'],

    modules: [
        // '@nuxthub/core',
        '@nuxt/content',
        '@nuxt/ui',
        '@nuxt/image',
        '@nuxt/fonts',
        '@nuxt/eslint',
        '@vueuse/nuxt',
        '@nuxthq/studio',
        'nuxt-shiki',
        'nuxt-og-image',
        '@nuxtjs/seo',
        'nuxt-svgo',
        'nuxt-umami',
    ],

    hooks: {
        // Define `@nuxt/ui` components as global to use them in `.md` (feel free to add those you need)
        'components:extend': (components) => {
            components.forEach(c => c.global = true)
        },
    },

    nitro: {
        prerender: {
            autoSubfolderIndex: false, // Temp while waiting for hosting on Nuxt Hub, https://nuxt.com/deploy/cloudflare#route-matching
            crawlLinks: true,
            routes: [
                '/',
                '/docs',
                // '/sitemap.xml', // Temp while waiting for hosting on Nuxt Hub
                // '/blog/rss.xml', // Temp while waiting for hosting on Nuxt Hub
                ...globSync('./content/**/*.md*')
                    .map(path => path
                        .slice(7, -3)
                        .replace(/\d+\./g, '')
                        .replace(/index/g, '/')
                        .replace(/\\/g, '/')
                        .replace(/\/+/g, '/')
                        .replace(/\/$/, '')
                    ),
                '/go',
            ]
        },
        experimental: {
            wasm: true,
        },
    },

    // hub: {
    //     database: true,
    //     blob: true
    // },

    routeRules: {
        '/api/search.json': { prerender: true },
        // '/blog/rss.xml': { prerender: true },
        // '/sitemap.xml': { prerender: true },
        '/docs': { redirect: '/docs/getting-started', prerender: false },
        '/go': { redirect: '/', prerender: false },


        // Redirects 
        '/pricing': { redirect: '/#pricing', prerender: false },
        '/go/github': { redirect: 'https://github.com/wind-press/windpress', prerender: false },
        '/go/discord': { redirect: 'https://discord.gg/fjsB83XdFw', prerender: false },
        '/go/facebook': { redirect: 'https://www.facebook.com/groups/1142662969627943', prerender: false },
        '/go/bricks': { redirect: 'https://bricksbuilder.io/', prerender: false },
        '/go/breakdance': { redirect: 'https://breakdance.com/ref/165/', prerender: false },
        '/go/builderius': { redirect: 'https://builderius.io/?referral=afdfca82c8', prerender: false },
        '/go/livecanvas': { redirect: 'https://livecanvas.com/?ref=4008', prerender: false },
        '/go/ads': { redirect: 'https://ko-fi.com/post/WindPress-Bronze-Sponsor-U7U8143WIP', prerender: false },
        '/go/sponsor': { redirect: 'https://ko-fi.com/Q5Q75XSF7?utm_source=windpress_website', prerender: false },
    },

    content: {
        defaultLocale: 'en',
        locales: ['en', 'es', 'id'],
        // navigation: {
        //     fields: ['author', 'publishedAt']
        // },
        highlight: {
            langs: [
                'html',
                'css',
                'postcss',
                'js',
                'php',
            ]
        }
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
        host: 'https://umami.siagian.dev',
        autoTrack: true,
        ignoreLocalhost: true,
        useDirective: true,
        // proxy: 'direct',
    },

    tailwindcss: {
        cssPath: ['./assets/css/tailwind.css', { injectPosition: 'last' }],
        viewer: false,
        exposeConfig: true,
    },

    colorMode: {
        preference: 'system', // default value of $colorMode.preference
        fallback: 'light', // fallback value if not system preference found
    },

    css: [
        './assets/css/main.css',
    ],

    ui: {
        safelistColors: ['sky', 'mint', 'rose', 'amber', 'violet', 'emerald', 'fuchsia', 'indigo', 'lime', 'orange', 'pink', 'purple', 'red', 'teal', 'yellow', 'green', 'blue', 'cyan', 'gray', 'white', 'black'],
    },

    icon: {
        provider: 'iconify', // SSG
        serverBundle: false, // SSG
        // serverBundle: {
        //     remote: 'jsdelivr', // 'unpkg' or 'github-raw', or a custom function
        // },
        clientBundle: {
            // scan all components in the project and include icons 
            scan: true,
        },
    },

    vite: {
        plugins: []
    },

    compatibilityDate: '2024-07-11'
})