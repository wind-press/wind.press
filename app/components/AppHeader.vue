<script setup lang="ts">
import type { NavItem } from '@nuxt/content';
import Logo from './Logo.vue';

const navigation = inject<Ref<NavItem[]>>('navigation', ref([]));

const wp_v12 = inject<Ref<{ version: string; active_installs: number; donate_link: string; download_link: string }>>('wp_v12', ref({ version: '', active_installs: 0, donate_link: '', download_link: '' }));

const { metaSymbol } = useShortcuts();

const links = [
  {
    label: 'Home',
    to: '/'
  },
  {
    label: 'Docs',
    to: '/docs'
  },
  {
    label: 'Blog',
    to: '/blog'
  },
  // {
  //   label: 'Pricing',
  //   to: '/pricing'
  // },
  // {
  //   label: 'Account',
  //   to: 'https://rosua.org/checkout/order-history',
  //   target: '_blank'
  // },
  {
    label: 'Account',
    to: '/login',
  },
]
</script>

<template>
  <UHeader :links="links">
    <template #logo>
      <Logo class="w-auto h-6 fill-black dark:fill-white self-center" />
      WindPress
      <UBadge :label="`v${wp_v12.version}`" variant="subtle" class="mb-0.5 hidden sm:flex" />
    </template>

    <template #right>

      <UTooltip text="Search" :shortcuts="[metaSymbol, 'K']" class="">
        <UContentSearchButton :label="null" />
      </UTooltip>

      <UTooltip :text="$colorMode.preference === 'dark' ? 'Switch to light mode' : 'Switch to dark mode'">
        <UColorModeButton />
      </UTooltip>

      <UButton label="Get WindPress" color="gray" to="https://wordpress.org/plugins/windpress/" target="_blank" class="hidden sm:flex" />
      <!-- <UButton label="Sign in" color="gray" to="/login" /> -->
      <!-- <UButton label="Sign up" icon="i-heroicons-arrow-right-20-solid" trailing color="black" to="/signup" class="hidden lg:flex" /> -->
    </template>

    <template #panel>
      <UNavigationTree :links="mapContentNavigation(navigation)" default-open />
    </template>
  </UHeader>
</template>
