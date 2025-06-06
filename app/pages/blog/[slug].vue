<script setup lang="ts">
import { kebabCase } from 'scule'

const colorMode = useColorMode()

definePageMeta({
  heroBackground: 'opacity-30 -z-10'
})

const route = useRoute()
const { copy } = useClipboard()

const [{ data: article }, { data: surround }] = await Promise.all([
  useAsyncData(kebabCase(route.path), () => queryCollection('blog').path(route.path).first()),
  useAsyncData(`${kebabCase(route.path)}-surround`, () => {
    return queryCollectionItemSurroundings('blog', route.path, {
      fields: ['description']
    }).order('date', 'DESC')
  })
])

if (!article.value) {
  throw createError({ statusCode: 404, statusMessage: 'Article not found', fatal: true })
}

const title = article.value.seo?.title || article.value.title
const description = article.value.seo?.description || article.value.description

useSeoMeta({
  titleTemplate: '%s · WindPress Blog',
  title,
  description,
  ogDescription: description,
  ogTitle: `${title} · WindPress Blog`
})

if (article.value.image) {
  defineOgImage({ url: article.value.image })
} else {
  defineOgImageComponent('Docs', {
    headline: 'Blog',
    title,
    description
  })
}

const authorTwitter = article.value.authors?.[0]?.twitter

const socialLinks = computed(() => !article.value
  ? []
  : [
    // {
    //   label: 'LinkedIn',
    //   icon: 'i-simple-icons-linkedin',
    //   to: `https://www.linkedin.com/sharing/share-offsite/?url=https://nuxt.com${article.value.path}`
    // },
    {
      label: 'X',
      icon: 'i-simple-icons-x',
      to: `https://x.com/intent/tweet?text=${encodeURIComponent(`${article.value.title}${authorTwitter ? ` by @${article.value.authors[0]!.twitter}` : ''}\n\n`)}https://wind.press${article.value.path}`
    }
  ])

function copyLink() {
  copy(`https://wind.press${article.value?.path || '/'}`, { title: 'Link copied to clipboard', icon: 'i-lucide-copy-check' })
}

const links = [
  {
    icon: 'i-ph-pencil-simple',
    label: 'Edit this article',
    to: `https://github.com/wind-press/wind.press/edit/main/content/${article.value.stem}`,
    target: '_blank'
  }, {
    icon: 'i-ph-star',
    label: 'Star on GitHub',
    to: '/go/github',
    target: '_blank'
  }, {
    icon: 'i-ph-hand-heart',
    label: 'Become a Sponsor',
    to: '/go/sponsor',
    target: '_blank'
  }
]
</script>

<template>
  <UContainer>
    <UPage v-if="article">
      <UPageHeader :title="article.title" :description="article.description" :ui="{ headline: 'flex flex-col gap-y-8 items-start' }">
        <template #headline>
          <UBreadcrumb :items="[{ label: 'Blog', icon: 'i-lucide-newspaper', to: '/blog' }, { label: article.title }]" class="max-w-full" />
          <div class="flex items-center space-x-2">
            <span>
              {{ article.category }}
            </span>
            <span class="text-(--ui-text-muted)">&middot;&nbsp;&nbsp;<time>{{ formatDateByLocale('en', article.date) }}</time></span>
          </div>
        </template>

        <div class="mt-4 flex flex-wrap items-center gap-6">
          <UUser v-for="(author, index) in article.authors" :key="index" v-bind="author" :description="author.to ? `@${author.to.split('/').pop()}` : undefined" />
        </div>
      </UPageHeader>

      <UPage class="lg:gap-24">
        <UPageBody>
          <ContentRenderer v-if="article.body" :value="article" />

          <div class="flex items-center justify-between mt-12 not-prose">
            <ULink to="/blog" class="text-(--ui-primary)">
              ← Back to blog
            </ULink>
            <div class="flex justify-end items-center gap-1.5">
              <UButton icon="i-lucide-link" variant="ghost" color="neutral" @click="copyLink">
                <span class="sr-only">Copy URL</span>
                Copy URL
              </UButton>
              <UButton v-for="(link, index) in socialLinks" :key="index" v-bind="link" variant="ghost" color="neutral" target="_blank">
                <span class="sr-only">Nuxt on {{ link.label }}</span>
              </UButton>
            </div>
          </div>

          <USeparator v-if="surround?.length" />

          <UContentSurround :surround="surround" />

          <Utterances repo="wind-press/wind.press" issue-term="pathname" :theme="`github-${colorMode.value}`" />

        </UPageBody>

        <template #right>
          <UContentToc v-if="article.body && article.body.toc" :links="article.body.toc.links" title="Table of Contents" highlight>
            <template #bottom>
              <div class="hidden lg:block space-y-6">
                <UPageLinks title="Links" :links="links" />
                <USeparator type="dashed" />
                <SocialLinks />
                <Ads />
              </div>
            </template>
          </UContentToc>
        </template>
      </UPage>
    </UPage>
  </UContainer>
</template>
