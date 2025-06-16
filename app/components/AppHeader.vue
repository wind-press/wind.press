<script setup lang="ts">
import type { ContentNavigationItem } from '@nuxt/content'

const navigation = inject<Ref<ContentNavigationItem[]>>('navigation', ref([]))
const logo = useTemplateRef('logo')
const route = useRoute()
const stats = useStats()
const { copy } = useClipboard()
const { headerLinks } = useHeaderLinks()

const mobileNavigation = computed<ContentNavigationItem[]>(() => {
  // Show Migration and Bridge on mobile only when user is reading them
  const docsLink = navigation.value.find(link => link.path === '/docs')
  if (docsLink && !route.path.startsWith('/docs/bridge') && !route.path.startsWith('/docs/migration')) {
    docsLink.children = docsLink.children?.filter(link => !['/docs/bridge', '/docs/migration'].includes(link.path as string)) || []
  }

  return [
    docsLink,
    ...headerLinks.value.slice(1).map(link => ({
      ...link,
      title: link.label,
      path: link.to,
      children: link.children?.map(child => ({
        ...child,
        title: child.label,
        path: child.to
      }))
    } as ContentNavigationItem)),
    // {
    //   title: 'Design Kit',
    //   icon: 'i-lucide-palette',
    //   path: '/design-kit'
    // }
  ].filter((item): item is ContentNavigationItem => Boolean(item))
})

const defaultOpen = computed(() => {
  const topLevelWithChildren = mobileNavigation.value.filter(link => link.children?.length)
  const currentPath = route.path

  return topLevelWithChildren.some(link => link.children?.some(child => currentPath.startsWith(child.path as string)))
})

const logoContextMenuItems = [
  [{
    label: 'Copy logo as SVG',
    icon: 'lucide:copy',
    onSelect() {
      if (logo.value) {
        copy(logo.value.$el.outerHTML, {
          title: 'WindPress logo copied as SVG',
          description: 'You can now paste it into your project',
          icon: 'i-lucide-circle-check',
          color: 'success'
        })
      }
    }
  }],
  // [{
  //   label: 'Browse design kit',
  //   icon: 'i-lucide-shapes',
  //   to: '/design-kit'
  // }]
]
</script>

<template>
  <UHeader>
    <template #left>
      <UContextMenu :items="logoContextMenuItems" size="xs">
        <NuxtLink to="/" class="flex gap-2 items-end" aria-label="Back to home">
          <NuxtLogo ref="logo" class="block w-auto h-6" />
          <span class="text-2xl leading-5 font-bold">WindPress</span>

          <UTooltip v-if="stats?.wp_version" :text="`Latest release: v${stats?.wp_version || 3}`">
            <UBadge variant="subtle" size="sm" class="-mb-[2px] rounded font-semibold text-[12px]/3">
              v{{ stats.wp_version }}
            </UBadge>
          </UTooltip>
        </NuxtLink>
      </UContextMenu>
    </template>

    <UNavigationMenu :items="headerLinks" variant="link" :ui="{ linkLeadingIcon: 'hidden' }" />

    <template #right>
      <UTooltip text="Search" :kbds="['meta', 'K']">
        <UContentSearchButton />
      </UTooltip>

      <!-- <UColorModeButton /> -->
      <ColorModeButton />

      <UTooltip text="GitHub Stars">
        <!-- <UButton
          icon="i-simple-icons-github"
          to="https://go.nuxt.com/github"
          target="_blank"
          variant="ghost"
          color="neutral"
          :label="stats ? formatNumber(stats.stars) : '...'"
          :ui="{
            label: 'hidden sm:inline-flex'
          }"
        >
          <span class="sr-only">Nuxt on GitHub</span>
        </UButton> -->
      </UTooltip>

      <UTooltip text="WindPress @ wordpress.org" class="hidden sm:flex" >
        <UButton
          icon="i-simple-icons-wordpress"
          to="https://wordpress.org/plugins/windpress/"
          target="_blank"
          variant="subtle"
          color="neutral"
          :label="stats ? formatNumber(stats.stars) : '...'"
          :ui="{
            label: 'hidden sm:inline-flex'
          }"
        >
          <span>Get WindPress</span>
        </UButton>
      </UTooltip>
    </template>

    <template #body>
      <UContentNavigation :navigation="mobileNavigation" :default-open="defaultOpen" highlight />
    </template>
  </UHeader>
</template>
