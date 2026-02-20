<script setup lang="ts">
type MarkdownNode = {
  type?: string
  value?: string
  tag?: string
  props?: Record<string, unknown>
  children?: MarkdownNode[]
}

type ChangelogVersion = {
  title: string
  date: string
  link: string
  changes: Record<string, string[]>
}

type ChangelogData = {
  versions: ChangelogVersion[]
  lastUpdated: string
  source: 'github'
}

const CHANGELOG_SOURCE_URL = 'https://raw.githubusercontent.com/wind-press/windpress/main/CHANGELOG.md'

const changelog = ref<ChangelogData | null>(null)
const pending = ref(true)
const error = ref<unknown>(null)

const nodeToHTML = (nodes: MarkdownNode[] = []) => {
  return nodes.map((node) => {
    if (node.type === 'text') {
      return node.value ?? ''
    }

    if (node.type !== 'element' || !node.tag) {
      return ''
    }

    const props = node.props ?? {}
    const attrs = Object.entries(props)
      .filter(([key, value]) => key !== 'id' && value !== undefined && value !== null)
      .map(([key, value]) => {
        if (Array.isArray(value)) {
          return `${key}="${value.join(' ')}"`
        }

        return `${key}="${String(value)}"`
      })
      .join(' ')
    const attrsString = attrs ? ` ${attrs}` : ''
    const childrenHTML = nodeToHTML(node.children)

    if (['br', 'hr', 'img'].includes(node.tag)) {
      return `<${node.tag}${attrsString} />`
    }

    return `<${node.tag}${attrsString}>${childrenHTML}</${node.tag}>`
  }).join('')
}

const parseChangelog = async (rawMarkdown: string): Promise<ChangelogData> => {
  const { parseMarkdown } = await import('@nuxtjs/mdc/runtime')
  const parsedMarkdown = await parseMarkdown(rawMarkdown)

  const linkReferences: Record<string, string> = {}
  for (const line of rawMarkdown.split('\n')) {
    if (!line.startsWith('[')) {
      continue
    }

    const keyEnd = line.indexOf(']:')
    if (keyEnd <= 1) {
      continue
    }

    const key = line.slice(1, keyEnd).trim()
    const link = line.slice(keyEnd + 2).trim()

    if (key && link) {
      linkReferences[key] = link
    }
  }

  const versions: ChangelogVersion[] = []
  const children = (parsedMarkdown.body?.children ?? []) as MarkdownNode[]
  let currentVersion: ChangelogVersion | null = null
  let currentChangeType = ''

  for (const node of children) {
    if (node.type === 'element' && node.tag === 'h2') {
      if (currentVersion) {
        versions.push(currentVersion)
      }

      const h2Children = node.children ?? []
      let title = ''
      let fullText = ''
      let linkHref = ''

      for (const child of h2Children) {
        if (child.type === 'text') {
          fullText += child.value ?? ''
        } else if (child.type === 'element' && child.tag === 'span' && child.children) {
          for (const spanChild of child.children) {
            if (spanChild.type === 'text') {
              title = spanChild.value ?? ''
            }
          }
        } else if (child.type === 'element' && child.tag === 'a' && child.props?.href) {
          linkHref = String(child.props.href)

          if (!title && child.children) {
            for (const anchorChild of child.children) {
              if (anchorChild.type === 'text') {
                title = anchorChild.value ?? ''
              }
            }
          }
        }
      }

      const titleDateSeparator = fullText.indexOf(' - ')

      if (titleDateSeparator === -1) {
        const versionTitle = title || fullText.trim()
        const linkKey = versionTitle.toLowerCase()

        currentVersion = {
          title: versionTitle,
          date: '',
          link: linkHref || linkReferences[linkKey] || linkReferences[versionTitle] || 'https://github.com/wind-press/windpress/compare/HEAD',
          changes: {}
        }
      } else {
        const parsedTitle = fullText.slice(0, titleDateSeparator).trim()
        const parsedDate = fullText.slice(titleDateSeparator + 3).trim()
        const versionTitle = title || parsedTitle

        currentVersion = {
          title: versionTitle,
          date: parsedDate,
          link: linkHref || linkReferences[versionTitle] || `https://github.com/wind-press/windpress/releases/tag/${versionTitle}`,
          changes: {}
        }
      }

      continue
    }

    if (node.type === 'element' && node.tag === 'h3' && currentVersion) {
      currentChangeType = ''

      for (const child of node.children ?? []) {
        if (child.type === 'text') {
          currentChangeType = (child.value ?? '').trim()
          break
        }
      }

      if (currentChangeType && !currentVersion.changes[currentChangeType]) {
        currentVersion.changes[currentChangeType] = []
      }

      continue
    }

    if (node.type === 'element' && node.tag === 'ul' && currentVersion && currentChangeType) {
      for (const listItem of node.children ?? []) {
        if (listItem.type !== 'element' || listItem.tag !== 'li') {
          continue
        }

        const htmlString = nodeToHTML(listItem.children).trim()
        if (htmlString) {
          currentVersion.changes[currentChangeType].push(htmlString)
        }
      }
    }
  }

  if (currentVersion) {
    versions.push(currentVersion)
  }

  return {
    versions,
    lastUpdated: new Date().toISOString(),
    source: 'github'
  }
}

const fetchChangelog = async () => {
  pending.value = true
  error.value = null

  try {
    const rawMarkdown = await $fetch<string>(CHANGELOG_SOURCE_URL, {
      responseType: 'text'
    })

    changelog.value = await parseChangelog(rawMarkdown)
  } catch (err) {
    error.value = err
  } finally {
    pending.value = false
  }
}

const refreshChangelog = async () => {
  await fetchChangelog()
}

onMounted(() => {
  void fetchChangelog()
})

const formatDate = (dateString: string) => {
  if (!dateString) {
    return 'Unknown'
  }

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
              <p class="mb-3">We're having trouble fetching the latest changelog from GitHub. You can:</p>
              <div class="flex flex-wrap gap-2">
                <UButton variant="outline" size="xs" @click="refreshChangelog()" :loading="pending" icon="i-lucide-refresh-cw">
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
              <UButton variant="ghost" size="sm" icon="i-lucide-refresh-cw" @click="refreshChangelog()" :loading="pending">
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
