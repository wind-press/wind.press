<script setup lang="ts">
definePageMeta({
  heroBackground: 'opacity-80 -z-10'
})

const [{ data: page }, { data: showcases }] = await Promise.all([
  useAsyncData('showcase-landing', () => queryCollection('landing').path('/showcase').first()),
  useAsyncData('showcase', () => queryCollection('showcase').all())
])

const fallbackPage = {
  title: 'Showcase',
  description: 'The awesome websites built with WindPress.',
  links: [{
    label: 'List a website',
    to: 'https://github.com/wind-press/wind.press/discussions/new?category=show-and-tell',
    icon: 'i-lucide-circle-plus',
    target: '_blank'
  }]
}

const showcasePage = computed(() => page.value || fallbackPage)

const title = page.value?.head?.title || page.value?.title || 'WindPress Showcase'
const description = page.value?.head?.description || page.value?.description || fallbackPage.description

useSeoMeta({
  titleTemplate: '%s',
  title,
  description,
  ogDescription: description,
  ogTitle: title
})
defineOgImageComponent('Docs', {
  title,
  description
})

function getHostname(url: string) {
  try {
    return new URL(url).hostname.replace(/^www\./, '')
  } catch {
    return url
      .replace(/^https?:\/\//, '')
      .replace(/^www\./, '')
      .split('/')[0]
  }
}

const showcaseCards = computed(() => {
  return (showcases.value || []).map((showcase, index) => {
    const hostname = getHostname(showcase.url)
    const author = showcase.author || showcase.owner

    return {
      ...showcase,
      index,
      author,
      siteName: showcase.name || hostname
    }
  })
})
</script>

<template>
  <UContainer>
    <UPageHero
      :title="showcasePage.title"
      :description="showcasePage.description"
      :links="showcasePage.links"
      :ui="{ container: '!pb-10 sm:!pb-12', links: 'gap-1.5 max-w-2xl mx-auto' }"
    />

    <UPage>
      <UPageBody>
        <section class="relative isolate overflow-hidden rounded-3xl border border-(--ui-border) bg-(--ui-bg-elevated)/40 p-5 sm:p-8">
          <div aria-hidden="true" class="pointer-events-none absolute -right-16 -top-20 size-64 rounded-full bg-(--ui-primary)/10 blur-3xl" />
          <div aria-hidden="true" class="pointer-events-none absolute -bottom-24 -left-16 size-72 rounded-full bg-(--ui-color-info-500)/10 blur-3xl" />

          <div class="relative flex flex-wrap items-end justify-between gap-4">
            <div>
              <p class="text-xs font-semibold uppercase tracking-[0.14em] text-(--ui-text-toned)">
                Community websites
              </p>
              <h2 class="mt-2 text-xl font-semibold text-(--ui-text-highlighted) sm:text-2xl">
                Websites built with WindPress
              </h2>
              <p class="mt-2 text-sm text-(--ui-text-muted)">
                Real projects shipped with WindPress. Explore their layouts, details, and ideas.
              </p>
            </div>

            <UButton
              v-if="showcasePage.links?.[0]"
              :to="showcasePage.links[0].to"
              :label="showcasePage.links[0].label"
              :target="showcasePage.links[0].target"
              icon="i-lucide-arrow-up-right"
              color="neutral"
              variant="outline"
              size="sm"
              class="rounded-full"
            />
          </div>

          <div class="relative mt-8 grid grid-cols-1 gap-x-6 gap-y-8 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
            <article
              v-for="showcase in showcaseCards"
              :key="showcase.slug"
              class="group relative"
            >
              <ULink
                :to="showcase.url"
                target="_blank"
                rel="noopener noreferrer"
                class="relative block overflow-hidden rounded-2xl border border-(--ui-border) bg-(--ui-bg-muted)"
              >
                <NuxtImg
                  :src="showcase.screenshotUrl?.startsWith('https') ? showcase.screenshotUrl : `/assets/showcase/${showcase.slug}.webp`"
                  :alt="`Screenshot of ${showcase.siteName}`"
                  :loading="showcase.index < 4 ? 'eager' : 'lazy'"
                  class="aspect-[4/3] w-full object-cover object-top transition duration-500 group-hover:scale-[1.02]"
                  width="488"
                  height="366"
                />
                <div aria-hidden="true" class="absolute inset-0 bg-gradient-to-t from-black/60 via-black/20 to-transparent opacity-0 transition-opacity duration-300 group-hover:opacity-100" />
                <div class="absolute inset-x-4 bottom-4 flex translate-y-3 items-center justify-center rounded-md bg-white/80 px-3 py-2 text-sm font-medium text-gray-900 opacity-0 backdrop-blur-sm transition duration-300 group-hover:translate-y-0 group-hover:opacity-100">
                  View website
                </div>
              </ULink>

              <div class="mt-4 flex items-start justify-between gap-4">
                <div class="min-w-0">
                  <h3 class="truncate text-base font-medium text-(--ui-text-highlighted)">
                    <ULink
                      :to="showcase.url"
                      target="_blank"
                      rel="noopener noreferrer"
                      class="transition-colors hover:text-(--ui-primary)"
                    >
                      {{ showcase.siteName }}
                    </ULink>
                  </h3>
                  <p v-if="showcase.author" class="mt-1 truncate text-sm text-(--ui-text-muted)">
                    by {{ showcase.author }}
                  </p>
                </div>
                <UIcon name="i-lucide-arrow-up-right" class="mt-0.5 size-4 shrink-0 text-(--ui-text-toned) transition-colors group-hover:text-(--ui-primary)" />
              </div>
            </article>
          </div>
        </section>
      </UPageBody>
    </UPage>
  </UContainer>
</template>
