<script setup lang="ts">
const { data: changelog, pending, error, refresh } = await useLazyFetch('/api/changelog')

// Cache invalidation function
const invalidateAndRefresh = async () => {
  try {
    await $fetch('/api/changelog/invalidate', { method: 'POST' })
    reloadNuxtApp()
  } catch (error) {
    console.error('Error invalidating cache:', error)
    reloadNuxtApp()
  }
}

// Format date helper
const formatDate = (dateString: string) => {
  return new Date(dateString).toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'long',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit'
  })
}

// SEO meta
useSeoMeta({
  title: 'Changelog - WindPress',
  description: 'Latest updates and changes to WindPress',
  ogTitle: 'WindPress Changelog',
  ogDescription: 'Latest updates and changes to WindPress',
  ogType: 'website'
})

definePageMeta({
  heroBackground: 'opacity-70 -z-10'
})

defineOgImageComponent('Docs', {
  title: 'Changelog - WindPress',
  description: 'Latest updates and changes to WindPress'
})
</script>

<template>
  <UContainer>
    <UPageHero title="Changelog" description="Stay updated with the latest changes and improvements to WindPress." :ui="{ container: '!pb-16', links: 'gap-1.5 max-w-2xl mx-auto' }">
      <template #links>
        <UButton to="https://github.com/wind-press/windpress/blob/main/CHANGELOG.md" color="neutral" external icon="i-lucide-github" variant="subtle" size="sm" target="_blank">
          View on GitHub
        </UButton>
      </template>
    </UPageHero>

    <UPageBody>
      <UContainer>
        <!-- Loading State -->
        <div v-if="pending" class="flex flex-col items-center justify-center py-24">
          <UIcon name="i-lucide-loader-2" class="animate-spin h-8 w-8 text-primary-500 mb-4" />
          <p class="text-lg font-medium text-gray-600 dark:text-gray-400">Loading changelog...</p>
          <p class="text-sm text-gray-500 dark:text-gray-500 mt-2">Fetching the latest updates from GitHub</p>
        </div>

        <!-- Error State -->
        <UAlert v-else-if="error" color="error" class="mb-8" variant="soft">
          <template #title>Unable to load changelog</template>
          <template #description>
            <div class="mt-2">
              <p class="mb-3">We're having trouble fetching the latest changelog. You can:</p>
              <div class="flex flex-wrap gap-2">
                <UButton variant="outline" size="xs" @click="invalidateAndRefresh()" :loading="pending" icon="i-lucide-refresh-cw">
                  Try Again
                </UButton>
                <UButton variant="link" size="xs" to="https://github.com/wind-press/windpress/blob/main/CHANGELOG.md" external icon="i-lucide-external-link">
                  View on GitHub
                </UButton>
              </div>
            </div>
          </template>
        </UAlert>

        <!-- Content -->
        <div v-else class="max-w-5xl mx-auto">
          <!-- Info bar at the top -->
          <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between mb-8 p-4 bg-gray-50 dark:bg-gray-900 rounded-lg border border-gray-200 dark:border-gray-800">
            <div class="flex items-center gap-2 text-sm text-gray-600 dark:text-gray-400">
              <UIcon name="i-lucide-clock" class="h-4 w-4" />
              <span>Last updated: {{ formatDate(changelog.lastUpdated) }}</span>
            </div>

            <div class="flex items-center gap-2 mt-4 sm:mt-0">
              <UButton variant="ghost" size="sm" icon="i-lucide-refresh-cw" @click="invalidateAndRefresh()" :loading="pending">
                Refresh
              </UButton>

              <UButton to="https://github.com/wind-press/windpress/releases" variant="ghost" size="sm" icon="i-lucide-tag" external target="_blank">
                Releases
              </UButton>
            </div>
          </div>

          <!-- Structured Changelog Versions -->
          <div v-if="changelog?.versions && changelog.versions.length > 0">
            <UChangelogVersions>
              <UChangelogVersion v-for="version in changelog.versions" :key="version.title" :title="version.title" :date="version.date" :ui="{
                title: 'relative text-xl text-pretty font-semibold text-highlighted mb-4',
              }">
                <template #body>
                  <div class="space-y-6">
                    <div v-for="(items, changeType) in version.changes" :key="changeType">
                      <h4 class="text-sm font-semibold text-gray-900 dark:text-gray-100 capitalize tracking-wide mb-3">
                        {{ changeType }}
                      </h4>
                      <ul class="space-y-2
                          [&_a]:text-primary-600 [&_a]:hover:text-primary-700 [&_a]:dark:text-primary-400 [&_a]:dark:hover:text-primary-300 [&_a]:font-medium [&_a]:underline [&_a]:decoration-1 [&_a]:underline-offset-2
                          [&_code]:px-1.5 [&_code]:py-0.5 [&_code]:text-sm [&_code]:font-mono [&_code]:font-medium [&_code]:rounded-md [&_code]:inline-block [&_code]:border [&_code]:border-muted [&_code]:text-highlighted [&_code]:bg-muted
                      ">
                        <li v-for="(item, itemIndex) in items" :key="itemIndex" class="flex items-start gap-3 text-sm leading-relaxed">
                          <span class="mt-1.5 h-1.5 w-1.5 rounded-full bg-gray-400 dark:bg-gray-500 flex-shrink-0"></span>
                          <div v-html="item" class="
                            [&_span:first-child]:font-medium
                            [&_span:first-child]:inline-flex
                            [&_span:first-child]:items-center
                            [&_span:first-child]:text-xs
                            [&_span:first-child]:px-2
                            [&_span:first-child]:py-1
                            [&_span:first-child]:gap-1
                            [&_span:first-child]:rounded-md
                            [&_span:first-child]:bg-important/10
                            [&_span:first-child]:text-important
                            [&_span:first-child]:ring
                            [&_span:first-child]:ring-inset
                            [&_span:first-child]:ring-important/25
                          "></div>
                        </li>
                      </ul>
                    </div>
                  </div>
                  <div class="mt-6 flex items-center justify-between">

                    <UButton v-if="version.link" :to="version.link" variant="ghost" size="sm" icon="i-lucide-external-link" target="_blank">
                      View on GitHub
                    </UButton>
                  </div>
                </template>
              </UChangelogVersion>
            </UChangelogVersions>
          </div>

          <!-- No versions available fallback -->
          <div v-else-if="changelog && (!changelog.versions || changelog.versions.length === 0)" class="text-gray-600 dark:text-gray-400 text-center py-12">
            No changelog versions available.
          </div>

          <!-- No content available -->
          <div v-else class="text-gray-600 dark:text-gray-400 text-center py-12">
            No changelog content available.
          </div>
        </div>
      </UContainer>
    </UPageBody>
  </UContainer>
</template>