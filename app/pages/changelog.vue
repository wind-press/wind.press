<script setup lang="ts">
const { data: changelog, pending, error, refresh } = await useLazyFetch('/api/changelog')

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

// Parse changelog items to handle links and formatting
const parseChangelogItem = (item: string) => {
  let parsed = item

  // Convert GitHub issue/PR links like [#58](https://github.com/wind-press/windpress/issues/58)
  // parsed = parsed.replace(/\[#(\d+)\]\(([^)]+)\)/g, '<a href="$2" class="text-primary-600 hover:text-primary-700 dark:text-primary-400 dark:hover:text-primary-300 font-medium underline decoration-1 underline-offset-2" target="_blank" rel="noopener">#$1</a>')

  // Convert other markdown links [text](url)
  // parsed = parsed.replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2" class="text-primary-600 hover:text-primary-700 dark:text-primary-400 dark:hover:text-primary-300 font-medium underline decoration-1 underline-offset-2" target="_blank" rel="noopener">$1</a>')

  // Convert inline code `code`
  // parsed = parsed.replace(/`([^`]+)`/g, '<code class="bg-gray-100 dark:bg-gray-800 text-gray-900 dark:text-gray-100 px-1.5 py-0.5 rounded text-xs font-mono border border-gray-200 dark:border-gray-700">$1</code>')

  // Convert bold text **text**
  // parsed = parsed.replace(/\*\*(.*?)\*\*/g, '<strong class="font-semibold text-gray-900 dark:text-gray-100">$1</strong>')

  // Convert version tags like [TW4] or [Bricks]
  // parsed = parsed.replace(/\[([A-Z][A-Z0-9]*)\]/g, '<span class="inline-flex items-center px-2 py-0.5 rounded-md text-xs font-medium bg-blue-50 text-blue-700 dark:bg-blue-900/20 dark:text-blue-300 border border-blue-200 dark:border-blue-800">$1</span>')

  return parsed
}

// SEO meta
useSeoMeta({
  title: 'Changelog - WindPress',
  description: 'Track all updates and improvements to WindPress. Stay up to date with new features, bug fixes, and enhancements.',
  ogTitle: 'WindPress Changelog',
  ogDescription: 'Latest updates and changes to WindPress',
  ogType: 'website'
})

definePageMeta({
  heroBackground: 'opacity-70 -z-10'
})
</script>

<template>
  <UContainer>
    <UPageHero title="Changelog" description="Track all updates and improvements to WindPress. Stay up to date with new features, bug fixes, and enhancements." orientation="horizontal">
      <template #links>
        <UButton to="https://github.com/wind-press/windpress/blob/main/CHANGELOG.md" color="neutral" external icon="i-lucide-github" variant="subtle" size="sm" target="_blank">
          View on GitHu
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
                <UButton variant="outline" size="xs" @click="refresh()" :loading="pending" icon="i-lucide-refresh-cw">
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
            <div class="flex flex-col sm:flex-row sm:items-center gap-2 sm:gap-4 text-sm text-gray-600 dark:text-gray-400 mb-4 sm:mb-0">
              <div class="flex items-center gap-2">
                <UIcon name="i-lucide-clock" class="h-4 w-4" />
                <span>Updated: {{ formatDate(changelog.lastUpdated) }}</span>
              </div>
              <div class="flex items-center gap-2">
                <UIcon name="i-lucide-database" class="h-4 w-4" />
                <span>Source: {{ changelog.source === 'github' ? 'GitHub' : 'Cache' }}</span>
              </div>
              <div v-if="changelog?.source === 'cache'" class="flex items-center gap-2 text-amber-600 dark:text-amber-400">
                <UIcon name="i-lucide-wifi-off" class="h-4 w-4" />
                <span>Cached content</span>
              </div>
            </div>

            <div class="flex items-center gap-2">
              <UButton variant="ghost" size="sm" icon="i-lucide-refresh-cw" @click="refresh()" :loading="pending">
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
              <UChangelogVersion v-for="version in changelog.versions" :key="version.badge" :title="version.title" :date="version.date" :ui="{
                title: 'relative text-xl text-pretty font-semibold text-highlighted mb-4',
              }">
                <template #body>
                  <div class="space-y-6">
                    <div v-for="(items, changeType) in version.changes" :key="changeType">
                      <h4 class="text-sm font-semibold text-gray-900 dark:text-gray-100 uppercase tracking-wide mb-3">
                        {{ changeType }}
                      </h4>
                      <ul class="space-y-2">
                        <li v-for="(item, itemIndex) in items" :key="itemIndex" class="flex items-start gap-3 text-sm leading-relaxed">
                          <span class="mt-1.5 h-1.5 w-1.5 rounded-full bg-gray-400 dark:bg-gray-500 flex-shrink-0"></span>
                          <span v-html="parseChangelogItem(item)" class="text-gray-700 dark:text-gray-300"></span>
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

          <!-- Fallback to full content if versions parsing failed -->
          <div v-else-if="changelog?.parsedContent" class="prose prose-gray dark:prose-invert max-w-none">
            <ContentRenderer :value="changelog.parsedContent" />
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