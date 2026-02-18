<script setup lang="ts">
import { joinURL } from 'ufo'

const title = 'Move from Winden to WindPress'
const description = 'Winden has been sunset. WindPress is the modern replacement built by the original team.'
const { url } = useSiteConfig()

useSeoMeta({
  title,
  description,
  ogTitle: title,
  ogDescription: description,
  ogImage: joinURL(url, '/pro/og-image.png')
})
</script>

<template>
  <UMain>
    <UPageHero
      headline="Move your project from Winden"
      title="Switch to WindPress"
      description="Winden is no longer maintained. WindPress is the next generation built by the original team."
      :ui="{ container: 'relative overflow-hidden', wrapper: 'lg:px-12', description: 'text-pretty' }"
    >
      <SkyBg />

      <div aria-hidden="true" class="hidden lg:block absolute z-[-1] border-x border-(--ui-border) inset-0 mx-4 sm:mx-6 lg:mx-8" />

      <div class="px-4 py-10 lg:border border-(--ui-border) bg-(--ui-bg)">
        <div class="max-w-2xl mx-auto space-y-5">
          <p>
            If you are migrating from Winden and need help with pricing or onboarding,
            please contact support and include your Winden license details.
          </p>

          <div class="flex flex-wrap gap-3">
            <UButton to="/#pricing">
              View pricing
            </UButton>

            <UButton to="/go/ticket" variant="subtle" color="neutral" icon="i-lucide-life-buoy">
              Contact support
            </UButton>
          </div>
        </div>
      </div>
    </UPageHero>
  </UMain>
</template>
