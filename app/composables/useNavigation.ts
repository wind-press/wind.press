import { createSharedComposable } from '@vueuse/core'

function _useHeaderLinks() {
  const route = useRoute()
  const headerLinks = computed(() => {
    return [
      {
        label: 'Docs',
        icon: 'i-lucide-book-marked',
        to: '/docs',
        search: false,
        active: route.path.startsWith('/docs'),
        children: [
          {
            label: 'Getting Started',
            description: 'Introduction and installation guide.',
            icon: 'i-lucide-rocket',
            to: '/docs/getting-started',
            active: route.path.startsWith('/docs/getting-started')
          },
          {
            label: 'Fonts',
            description: 'Add custom fonts, self-host Google Fonts, and embed Adobe Fonts.',
            icon: 'i-lucide-type',
            to: '/docs/fonts',
            active: route.path.startsWith('/docs/fonts')
          },
          {
            label: 'Integrations',
            description: 'Use your fonts in page builders, themes, and editors.',
            icon: 'i-lucide-puzzle',
            to: '/docs/integrations',
            active: route.path.startsWith('/docs/integrations')
          },
          {
            label: 'Misc',
            description: 'Tools and extras for managing custom fonts.',
            icon: 'i-lucide-wrench',
            to: '/docs/misc',
            active: route.path.startsWith('/docs/misc')
          },
        ]
      },
      {
        label: 'Tools',
        icon: 'i-lucide-wand-sparkles',
        search: false,
        active: route.path.startsWith('/docs/misc/convert-ttf-woff2') || route.path.startsWith('/docs/misc/static-fonts-to-variable'),
        children: [
          {
            label: 'Font Converter',
            description: 'Convert TTF to WOFF2 in your browser.',
            icon: 'i-lucide-file-down',
            to: '/docs/misc/convert-ttf-woff2',
            active: route.path === '/docs/misc/convert-ttf-woff2'
          },
          {
            label: 'Static to Variable',
            description: 'Build a variable TTF from static font masters.',
            icon: 'i-lucide-sliders-horizontal',
            to: '/docs/misc/static-fonts-to-variable',
            active: route.path === '/docs/misc/static-fonts-to-variable'
          }
        ]
      },
      {
        label: 'Resources',
        icon: 'i-lucide-library',
        search: false,
        active: false,
        children: [
          {
            label: 'Blog',
            description: 'Releases, updates, and announcements.',
            icon: 'i-lucide-newspaper',
            to: '/blog'
          },
          {
            label: 'Changelog',
            description: 'Stay up-to-date with the latest changes.',
            icon: 'i-lucide-list-check',
            to: '/changelog'
          }
        ]
      },
      {
        label: 'Support',
        icon: 'i-lucide-life-buoy',
        search: false,
        children: [
          {
            label: 'Customer Support',
            to: '/go/ticket',
            description: 'Send us a support ticket.',
            icon: 'i-lucide-life-buoy'
          },
          {
            label: 'Community',
            to: '/go/facebook',
            description: 'Join our Facebook group.',
            icon: 'i-simple-icons-facebook'
          },
          {
            label: 'Sponsors',
            to: '/go/sponsor',
            description: 'Help us sustain Yabe Webfont development.',
            icon: 'i-ph-hand-heart'
          }
        ]
      },
    ]
  })
  return { headerLinks }
}

export const useHeaderLinks = import.meta.client ? createSharedComposable(_useHeaderLinks) : _useHeaderLinks

const footerLinks = [
  {
    label: 'Product',
    children: [
      {
        label: 'Features',
        to: '/#features',
      },
      {
        label: 'Pricing',
        to: '/#pricing'
      }
    ]
  },
  {
    label: 'Resources',
    children: [
      {
        label: 'Documentation',
        to: '/docs'
      },
      {
        label: 'Getting Started',
        to: '/docs/getting-started'
      },
      {
        label: 'Integrations',
        to: '/docs/integrations'
      },
    ]
  },
  {
    label: 'Support',
    children: [
      {
        label: 'Customer support',
        to: '/go/ticket',
        target: '_blank'
      },
      {
        label: 'Community',
        to: '/go/facebook',
        target: '_blank'
      },
      {
        label: 'Account',
        to: '/go/account',
        target: '_blank'
      },
    ]
  },
]

export const useFooterLinks = () => ({ footerLinks })

const _useNavigation = () => {
  const nuxtApp = useNuxtApp()
  const searchTerm = ref<string>('')

  const { headerLinks } = useHeaderLinks()
  const { footerLinks } = useFooterLinks()

  const searchLinks = computed(() => [
    // {
    //   label: 'Ask AI',
    //   icon: 'i-lucide-wand',
    //   to: 'javascript:void(0);',
    //   onSelect: () => nuxtApp.$kapa?.openModal()
    // },
    ...headerLinks.value.map((link) => {
      // Remove `/docs` and `/enterprise` links from command palette
      if (link.search === false) {
        return {
          label: link.label,
          icon: link.icon,
          children: link.children
        }
      }
      return link
    }).filter((link): link is NonNullable<typeof link> => Boolean(link))
  ])

  type SearchGroup = {
    id: string
    label: string
    icon?: string
    items: Array<{
      id: string
      label: string
      suffix?: string
      icon?: string
      avatar?: {
        src?: string
        ui?: {
          root: string
        }
      }
      to: string
      onSelect?: () => Promise<void>
    }>
  }

  const searchGroups = computed<SearchGroup[]>(() => {
    const aiGroup: SearchGroup = {
      id: 'ask-ai-search',
      label: 'AI',
      icon: 'i-lucide-wand',
      items: []
    }

    const modulesGroup: SearchGroup = {
      id: 'modules-search',
      label: 'Modules',
      items: []
    }

    const hostingGroup: SearchGroup = {
      id: 'hosting-search',
      label: 'Hosting',
      items: []
    }

    const groups = [aiGroup, modulesGroup, hostingGroup]

    if (!searchTerm.value) {
      return groups
    }

    aiGroup.items = [
      {
        id: `ask-ai-${searchTerm.value}`,
        label: `Ask AI about "${searchTerm.value}"`,
        icon: 'i-lucide-wand',
        to: 'javascript:void(0);',
        onSelect() {
          return nuxtApp.$kapa.openModal(searchTerm.value)
        }
      }
    ]

    const loadModules = async () => {
      const { modules, fetchList } = useModules()
      if (!modules.value.length) {
        await fetchList()
      }

      modulesGroup.items = modules.value
        .filter(module => ['name', 'npm', 'repo'].map(field => module[field as keyof typeof module]).filter(Boolean).some(value => typeof value === 'string' && value.search(searchTextRegExp(searchTerm.value)) !== -1))
        .map(module => ({
          id: `module-${module.name}`,
          label: module.npm,
          suffix: module.description,
          avatar: {
            src: moduleImage(module.icon),
            ui: {
              root: 'rounded-none bg-transparent'
            }
          },
          to: `/modules/${module.name}`
        }))
    }

    const loadHosting = async () => {
      const { providers, fetchList } = useHostingProviders()
      if (!providers.value.length) {
        await fetchList()
      }

      hostingGroup.items = providers.value
        .filter(hosting => ['title'].map(field => hosting[field as keyof typeof hosting]).filter(Boolean).some(value => typeof value === 'string' && value.search(searchTextRegExp(searchTerm.value)) !== -1))
        .map(hosting => ({
          id: `hosting-${hosting.path}`,
          label: hosting.title,
          suffix: hosting.description,
          icon: hosting.logoIcon,
          avatar: hosting.logoSrc
            ? {
              src: hosting.logoSrc,
              ui: {
                root: 'rounded-none bg-transparent'
              }
            }
            : undefined,
          to: hosting.path
        }))
    }

    onMounted(() => {
      Promise.all([
        // loadModules(),
        // loadHosting()
      ]).catch(error => console.error('Error loading search results:', error))
    })

    return groups
  })

  return {
    searchTerm,
    headerLinks,
    footerLinks,
    searchLinks,
    searchGroups
  }
}

export const useNavigation = import.meta.client ? createSharedComposable(_useNavigation) : _useNavigation
