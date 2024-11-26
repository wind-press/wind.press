<script setup lang="ts">
import { withoutTrailingSlash, joinURL } from 'ufo'
import type { BlogPost } from '~/types'

const route = useRoute()
const { copy } = useCopyToClipboard()

definePageMeta({
  heroBackground: 'opacity-40 -z-10'
})

const { data: post } = await useAsyncData(route.path, () => queryContent<BlogPost>(route.path).findOne())
if (!post.value) {
  throw createError({ statusCode: 404, statusMessage: 'Post not found', fatal: true })
}

const { data: surround } = await useAsyncData(`${route.path}-surround`, () => queryContent('/blog')
  .where({ _extension: 'md' })
  .without(['body', 'excerpt'])
  .sort({ date: -1 })
  .findSurround(withoutTrailingSlash(route.path))
  , { default: () => [] })

const title = post.value.head?.title || post.value.title
const description = post.value.head?.description || post.value.description

useSeoMeta({
  title,
  ogTitle: title,
  description,
  ogDescription: description
})

if (post.value.image?.src) {
  const site = useSiteConfig()

  useSeoMeta({
    ogImage: joinURL(site.url, post.value.image.src),
    twitterImage: joinURL(site.url, post.value.image.src)
  })
} else {
  defineOgImage({
    component: 'Saas',
    title,
    description,
    headline: 'Blog'
  })
}

const authorTwitter = post.value.authors?.[0]?.twitter
const socialLinks = computed(() => [{
  icon: 'i-simple-icons-linkedin',
  to: `https://www.linkedin.com/sharing/share-offsite/?url=https://wind.press${post.value._path}`
}, {
  icon: 'i-simple-icons-twitter',
  to: `https://twitter.com/intent/tweet?text=${encodeURIComponent(`${post.value.title}${authorTwitter ? ` by @${post.value.authors[0].twitter}` : ''}\n\n`)}https://wind.press${post.value._path}`
}, {
  icon: 'i-simple-icons-facebook',
  to: `https://www.facebook.com/sharer/sharer.php?u=https://wind.press${post.value._path}`
}
]);

function copyLink() {
  copy(`https://wind.press${post.value._path}`, { title: 'Copied to clipboard' })
}

const links = [
  {
    icon: 'i-ph-pen',
    label: 'Edit this article',
    to: `https://github.com/wind-press/wind.press/edit/main/content/${post.value._file}`,
    target: '_blank'
  }, {
    icon: 'i-ph-chat-centered-text',
    label: 'Discussions',
    to: 'https://github.com/wind-press/windpress/discussions',
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
  <UContainer v-if="post">
    <UPageHeader :title="post.title" :description="post.description" :ui="{ headline: 'flex flex-col gap-y-8 items-start' }">
      <template #headline>
        <UBreadcrumb :links="[{ label: 'Blog', icon: 'i-ph-newspaper', to: '/blog' }, { label: post.title }]" :ui="{ wrapper: 'max-w-full' }" />
        <div class="flex items-center space-x-2">
          <span>
            {{ post.category ?? 'Uncategorized' }}
          </span>
          <span class="text-gray-500 dark:text-gray-400">&middot;&nbsp;&nbsp;<time>{{ new Date(post.date).toLocaleDateString('en', { year: 'numeric', month: 'short', day: 'numeric' }) }}</time></span>
        </div>
      </template>

      <UBadge v-bind="post.badge" variant="subtle" class="mt-2" />

      <div class="flex flex-wrap items-center gap-3 mt-4">
        <UButton v-for="(author, index) in post.authors" :key="index" :to="author.to" color="white" target="_blank" size="sm">
          <UAvatar v-bind="author.avatar" :alt="author.name" size="2xs" />
          {{ author.name }}
        </UButton>


      </div>
    </UPageHeader>

    <UPage>
      <UPageBody prose>
        <ContentRenderer v-if="post && post.body" :value="post" />



        <div class="flex items-center justify-between mt-12 not-prose">
          <NuxtLink href="/blog" class="text-primary">
            ← Back to blog
          </NuxtLink>
          <div class="flex justify-end items-center gap-1.5">
            <UButton icon="i-ph-link-simple" v-bind="($ui.button.secondary as any)" @click="copyLink">
              Copy URL
            </UButton>
            <UButton v-for="(link, index) in socialLinks" :key="index" v-bind="link" variant="ghost" color="gray" target="_blank" />
          </div>
        </div>

        <hr v-if="surround?.length">

        <UContentSurround :surround="surround" />
      </UPageBody>

      <template #right>
        <UContentToc v-if="post.body && post.body.toc" :links="post.body.toc.links">
          <template #bottom>
            <div class="hidden lg:block space-y-6">
              <UPageLinks title="Links" :links="links" />
              <UDivider type="dashed" />
              <SocialLinks />
              <Ads />
            </div>
          </template>
        </UContentToc>
      </template>
    </UPage>
  </UContainer>
</template>
