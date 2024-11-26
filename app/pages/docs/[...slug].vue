<script setup lang="ts">
import { withoutTrailingSlash } from 'ufo'
import lightGallery from 'lightgallery';
import lgThumbnail from 'lightgallery/plugins/thumbnail';
import lgZoom from 'lightgallery/plugins/zoom';

const route = useRoute()

const { data: page } = await useAsyncData(route.path, () => queryContent(route.path).findOne())
if (!page.value) {
  throw createError({ statusCode: 404, statusMessage: 'Page not found', fatal: true })
}

const { data: surround } = await useAsyncData(`${route.path}-surround`, () => queryContent('/docs')
  .where({ _extension: 'md', navigation: { $ne: false } })
  .only(['title', 'description', '_path'])
  .findSurround(withoutTrailingSlash(route.path))
  , { default: () => [] })

useSeoMeta({
  title: page.value.title,
  ogTitle: page.value.title,
  description: page.value.description,
  ogDescription: page.value.description
})

defineOgImage({
  component: 'Saas',
  title: page.value.title,
  description: page.value.description
})

const headline = computed(() => findPageHeadline(page.value!))

onMounted(() => {
  lightGallery(document.querySelector('.nuxt-content__body'), {
    plugins: [lgZoom, lgThumbnail],
    selector: 'img',
    exThumbImage: 'src',
    supportLegacyBrowser: false,
    showZoomInOutIcons: true,
    licenseKey: 'your_license_key',
  })
})

const communityLinks = computed(() => [{
  icon: 'i-ph-pen',
  label: 'Edit this page',
  to: `https://github.com/wind-press/wind.press/edit/main/content/${page?.value?._file?.split('/').slice(1).join('/')}`,
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
}])
</script>

<template>
  <UPage v-if="page">
    <UPageHeader :title="page.title" :description="page.description" :links="page.links" :headline="headline" />

    <UPageBody prose>
      <ContentRenderer v-if="page.body" :value="page" class="nuxt-content__body" />

      <hr v-if="surround?.length">

      <UContentSurround :surround="surround" />
    </UPageBody>

    <template v-if="page.toc !== false" #right>
      <UContentToc :links="page.body?.toc?.links" :ui="{ /*wrapper: ''*/ }">
        <template #bottom>
          <div class="hidden lg:block space-y-6" :class="{ '!mt-6': page.body?.toc?.links?.length }">
            <UDivider v-if="page.body?.toc?.links?.length" type="dashed" />

            <UPageLinks title="Community" :links="communityLinks" />

            <UDivider type="dashed" />
            <SocialLinks />

            <Ads />
          </div>
        </template>
      </UContentToc>
    </template>
  </UPage>
</template>

<style>
.nuxt-content {
  .iconify {
    /* display: inline-block; */
    /* height: 1em; */
    /* width: 1em; */
    /* overflow: visible; */
    vertical-align: -.125em;
  }

  img {
    cursor: pointer;
  }
}
</style>