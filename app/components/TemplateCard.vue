<script lang="ts" setup>
import type { TemplatesCollectionItem } from '@nuxt/content'

defineProps<{
  template: TemplatesCollectionItem
  index: number
}>()
</script>

<template>
  <UPageCard
    :description="template.description"
    :ui="{
      container: 'p-0 sm:p-0',
      body: 'p-4 h-[105px]',
      header: 'relative mb-0 aspect-video',
      title: 'flex items-center w-full',
      description: 'line-clamp-2',
      footer: 'mt-0 px-4 pb-4 w-full'
    }"
    class="overflow-hidden"
  >
    <template #header>
      <NuxtImg
        :src="template.screenshotUrl.startsWith('https') ? template.screenshotUrl : `/assets/templates/${template.screenshotUrl}.png`"
        class="object-contain object-top w-full h-full xl:hidden"
        :alt="template.name"
        width="600"
        height="338"
        format="webp"
        :modifiers="{ pos: 'top' }"
        :loading="index > 3 ? 'lazy' : undefined"
      />
      <NuxtImg
        :src="template.screenshotUrl.startsWith('https') ? template.screenshotUrl : `/assets/templates/${template.screenshotUrl}.png`"
        class="object-cover object-top hidden xl:block"
        :alt="template.name"
        width="600"
        height="338"
        format="webp"
        :modifiers="{ pos: 'top' }"
        :loading="index > 3 ? 'lazy' : undefined"
      />
    </template>
    <template #title>
      <div class="w-full grid grid-cols-[1fr_auto] items-center gap-2">
        <p class="truncate m-0">
          {{ template.name }}
        </p>
        <div class="flex shrink-0 gap-1">
          <UBadge
            v-if="template.badge === 'Premium'"
            :label="template.badge"
            color="info"
            variant="subtle"
            size="sm"
            class="rounded-full"
          />
          <UBadge
            v-else-if="template.badge === 'Freemium'"
            :label="template.badge"
            color="success"
            variant="subtle"
            size="sm"
            class="rounded-full"
          />
        </div>
      </div>
    </template>
    <template #footer>
      <UButtonGroup class="w-full grid grid-cols-[repeat(auto-fit,minmax(50px,1fr))] </UButtonGroup>gap-2">
        <UButton
          v-if="template.demo"
          label="Demo"
          icon="i-lucide-laptop"
          :to="template.demo"
          target="_blank"
          size="sm"
          color="neutral"
          variant="subtle"
          class="justify-center"
        />
        <UButton
          v-if="template.repo"
          label="GitHub"
          icon="i-simple-icons-github"
          :to="`https://github.com/${template.repo}`"
          target="_blank"
          size="sm"
          color="neutral"
          variant="subtle"
          class="justify-center"
        />
        <UButton
          v-if="template.purchase"
          target="_blank"
          :to="template.purchase"
          color="neutral"
          label="Purchase"
          variant="subtle"
          icon="i-lucide-credit-card"
          size="sm"
          class="justify-center"
        />
        <UButton
          v-if="template.site"
          target="_blank"
          :to="template.site"
          color="neutral"
          label="Website"
          variant="subtle"
          icon="i-lucide-globe"
          size="sm"
          class="justify-center"
        />
      </UButtonGroup>
    </template>
  </UPageCard>
</template>
