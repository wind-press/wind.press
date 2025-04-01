<script setup lang="ts">
defineProps<{
  quote_title: string
  quote: string
  author: {
    name: string
    description?: string
    avatar?: {
      src: string
    }
    to?: string
    target?: '_blank' | '_self'
  }
  achievements?: Array<{
    label: string
    color: 'primary' | 'secondary' | 'info' | 'success' | 'warning' | 'error' | 'important' | 'neutral'
  }>
}>()
</script>

<template>
  <UPageCard variant="subtle" :ui="{ root: 'h-full flex-grow' }">
    <template #description>
      <div class="font-semibold text-(--ui-text-highlighted) text-base">
        {{ quote_title }}
      </div>
      “{{ quote }}”
    </template>

    <div class="flex flex-col justify-end h-full gap-y-4">
      <div class="flex justify-between gap-x-8 pt-6">
        <div>

          <UUser :to="author.to" target="_blank" :name="author.name" :description="author.description" :avatar="{
            src: author.avatar.src
          }" :ui="{
            name: 'font-semibold text-(--ui-text-highlighted) text-base',
            description: 'text-sm text-(--ui-text-muted)',
          }" />
        </div>

        <!-- <NuxtImg :src="author.company.src" class="h-6 shrink-0 max-w-[140px] grayscale" /> -->
      </div>

      <ul class="flex gap-x-1 flex-wrap gap-2">
        <li v-for="achievement in achievements" :key="achievement.label">
          <UBadge v-bind="achievement" variant="subtle" />
        </li>
      </ul>
    </div>
  </UPageCard>
</template>
