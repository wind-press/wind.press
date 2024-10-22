<script setup lang="ts">
import type { NavItem } from '@nuxt/content'

const navigation = inject<Ref<NavItem[]>>('navigation', ref([]))

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
    label: 'Account',
    to: 'https://rosua.org/checkout/order-history',
    target: '_blank'
  },
  // {
  //   label: 'Pricing',
  //   to: '/pricing'
  // },
  // {
  //   label: 'Blog',
  //   to: '/blog'
  // }
]
</script>

<template>
  <UHeader :links="links">
    <template #logo>
      WindPress
      <UBadge label="Draft" variant="subtle" class="mb-0.5" />
      <!-- <UBadge label="SaaS" variant="subtle" class="mb-0.5" /> -->
    </template>

    <template #right>
      <UButton label="Get WindPress" color="gray" to="https://wordpress.org/plugins/windpress/" target="_blank" />
      <!-- <UButton label="Sign in" color="gray" to="/login" /> -->
      <!-- <UButton label="Sign up" icon="i-heroicons-arrow-right-20-solid" trailing color="black" to="/signup" class="hidden lg:flex" /> -->
    </template>

    <template #panel>
      <UNavigationTree :links="mapContentNavigation(navigation)" default-open />
    </template>
  </UHeader>
</template>
