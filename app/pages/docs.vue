<script setup lang="ts">
import type { NavItem } from '@nuxt/content'

const navigation = inject<Ref<NavItem[]>>('navigation', ref([]))

const links = computed(() => navigation.value.find(item => item._path === '/docs')?.children ?? [])

definePageMeta({
  heroBackground: 'opacity-30 z-20'
})

</script>

<template>
  <UContainer>
    <UPage class="nuxt-content">
      <template #left>
        <UAside>
          <template #top>
            <UContentSearchButton
              class="rounded-md"
              size="sm"
            />
          </template>

          <UNavigationTree :links="mapContentNavigation(links)" />
        </UAside>
      </template>

      <NuxtPage />
    </UPage>
  </UContainer>
</template>
