<script setup lang="ts">
// @ts-expect-error yaml is not typed
import pricing from '../../content/pricing.yml';
// @ts-expect-error yaml is not typed
import faq from '../../content/faq.yml';

definePageMeta({
  heroBackground: '-z-10'
})

const [
  { data: page },
] = await Promise.all([
  useAsyncData('index', () => queryCollection('index').first()),
])

const stats = useStats()

const title = 'Yabe Webfont: The GDPR-friendly font plugin for WordPress'
useSeoMeta({
  title,
  titleTemplate: '%s'
})

if (import.meta.server) {
  const description = 'Self-host Google Fonts, embed Adobe Fonts, and manage custom fonts in WordPress — with first-class visual builder integration.'
  useSeoMeta({
    ogTitle: title,
    description: description,
    ogDescription: description
  })

  defineOgImage('Home', {
    title,
    description
  })
}
</script>

<template>
  <div v-if="page">
    <UPageHero :description="page.hero.description" :links="page.hero.links" :ui="{ title: 'text-4xl sm:text-5xl', root: 'mx-auto max-w-7xl', description: 'sm:text-lg', container: 'py-24 sm:py-32 lg:py-40' }">
      <template #title>
        The <span class="gdpr-highlight rounded-md px-1.5">GDPR-friendly</span> font plugin for WordPress
      </template>
      <div class="absolute inset-0 landing-grid z-[-1] [mask-image:radial-gradient(100%_100%_at_top_right,white,transparent)]"></div>
      <SkyBg />

      <template #description>
        {{ page.hero.description }}

        <div class="mt-12 mb-8">
          <div class="feature__container grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-6 auto-cols-auto gap-x-4 gap-y-8 sm:gap-11">
            <div class="feature__item">
              <div class="feature__icon">
                <UIcon name="i-custom-feature-gdpr" />
              </div>
              <div class="feature__title"> GDPR / DSGVO friendly </div>
            </div>
            <div class="feature__item">
              <div class="feature__icon">
                <UIcon name="i-custom-feature-google-fonts" />
              </div>
              <div class="feature__title"> Google Fonts self-host </div>
            </div>
            <div class="feature__item">
              <div class="feature__icon text-rose-600">
                <Icon name="fa6-regular:face-smile-wink" />
              </div>
              <div class="feature__title"> Easy to use and intuitive </div>
            </div>
            <div class="feature__item">
              <div class="feature__icon text-sky-500">
                <Icon name="fa6-solid:feather" />
              </div>
              <div class="feature__title"> Lightweight and fast </div>
            </div>
            <div class="feature__item">
              <div class="feature__icon text-gray-900">
                <Icon name="simple-icons:adobefonts" />
              </div>
              <div class="feature__title"> Adobe Fonts support </div>
            </div>
            <div class="feature__item">
              <div class="feature__icon text-[#4e6fc3]">
                <Icon name="fa6-brands:wordpress" />
              </div>
              <div class="feature__title"> Visual builders integration </div>
            </div>
          </div>
        </div>
      </template>

      <div class="badges__list grid grid-cols-1 sm:grid-cols-2 md:grid-cols-4 gap-4 *:ring-1 *:ring-gray-200 dark:*:ring-gray-800 *:bg-gray-100/50 dark:*:bg-gray-800/50">
        <div class="badges__item text-center">
          <div class="text-2xl flex items-center">
            <Icon name="fa6-solid:hand-holding-dollar" class="text-[#29b077]" />
          </div>
          <div class="text-xs text-left">
            <strong>Free</strong>
            Forever
          </div>
        </div>

        <div class="badges__item">
          <div class="text-2xl flex items-center">
            <Icon name="fa6-solid:headset" />
          </div>
          <div class="text-xs text-left">
            <div class="flex text-[#feb82c] mb-1">
              <Icon v-for="i in 5" name="fa6-solid:star" />
            </div>
            <strong>Support</strong> Quality
          </div>
        </div>

        <div class="badges__item">
          <div class="text-2xl flex items-center text-rose-600">
            <UIcon name="i-ph-smiley-fill" class="text-rose-600" />
          </div>
          <div class="text-xs text-left">
            <strong>{{ stats ? `${(stats.edd_happyCustomers || 0) + (stats.wp_active_installs || 0)}+` : '—' }} Happy</strong>
            Users
          </div>
        </div>

        <div class="badges__item">
          <div class="text-2xl flex items-center text-[#4e6fc3]">
            <Icon name="fa6-brands:wordpress" />
          </div>
          <div class="text-xs text-left">
            <strong>{{ stats?.wp_downloaded ? `${stats.wp_downloaded}+` : '—' }} Download</strong>
            & Counting
          </div>
        </div>
      </div>

    </UPageHero>

    <div v-if="page.as_seen_on" class="mb-8">
      <div class="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8">
        <div class="rounded-2xl bg-(--ui-bg-muted) border border-(--ui-border) p-8 sm:p-10">
          <p class="text-center text-lg font-semibold text-(--ui-text) mb-8">As seen on</p>
          <div class="grid grid-cols-2 sm:grid-cols-4 gap-8 items-center justify-items-center">
            <a v-for="item in page.as_seen_on" :key="item.name" :href="item.url" target="_blank" rel="noopener noreferrer" class="flex items-center justify-center opacity-60 hover:opacity-100 transition-opacity">
              <img :src="item.logo" :alt="item.name" class="max-h-14 max-w-full" />
            </a>
          </div>
        </div>
      </div>
    </div>

    <UPageSection>
      <div id="features" class="flex flex-col items-center">
        <div class="px-2 py-1 mb-2 rounded-lg bg-sky-100 text-sm text-sky-500 dark:bg-sky-600 dark:text-sky-100 text-center uppercase font-bold tracking-widest">
          Fully-featured
        </div>
        <h2 class="mb-6 text-3xl font-semibold text-center">
          Made for Designers and Developers
        </h2>
        <div class="flex justify-center text-center">
          <p class="max-w-[60%] text-lg leading-relaxed">
            At Yabe Webfont, we created an <strong>advanced, easy-to-use, and intuitive font plugin</strong> for WordPress to streamline your workflow.
          </p>
        </div>

        <div class="bento flex flex-col sm:grid sm:grid-cols-12 sm:grid-rows-12 gap-4 w-full mt-16">
          <div class="relative col-span-12 row-span-3 p-5 rounded-lg overflow-hidden hover:shadow-md bg-gradient-to-br from-blue-700/20 from-5% via-blue-500/10 via-50% to-blue-100/10">
            <div class="flex text-blue-900 dark:text-blue-300 items-center gap-4">
              <div class="text-lg font-semibold">GDPR / DSGVO friendly</div>
            </div>
            <div class="mt-4 leading-relaxed">
              <div class="sm:w-1/3">
                Import and self-host Google Fonts files from your server through your WordPress admin page. Fewer reasons to worry about GDPR / DSGVO compliance.
              </div>
            </div>
            <img class="absolute hidden sm:block sm:top-[-50px] sm:right-[-80px] rounded-lg scale-75" src="/tutorial/feature-gdpr-friendly.png" alt="GDPR friendly" />
          </div>

          <div class="relative col-span-3 row-span-3 p-5 rounded-lg overflow-hidden hover:shadow-md bg-gradient-to-br from-lime-700/20 from-5% via-lime-500/10 via-50% to-lime-100/10">
            <div class="flex text-lime-900 dark:text-lime-300 items-center gap-4">
              <div class="text-lg font-semibold">Multiple file formats</div>
            </div>
            <div class="mt-4 leading-relaxed">
              Choose between WOFF2, WOFF, and TTF per font to support the browsers you care about.
            </div>
            <img class="absolute hidden sm:block sm:bottom-[-86px] sm:right-[-120px] rounded-lg scale-[1.8] [clip-path:inset(95px_0px_0px_5px)]" src="/tutorial/feature-fine-tuning.png" alt="Multiple file formats" />
          </div>

          <div class="relative col-span-6 row-span-6 p-5 rounded-lg overflow-hidden hover:shadow-md bg-gradient-to-br from-sky-700/20 from-5% via-sky-500/10 via-50% to-sky-100/10">
            <div class="flex text-sky-900 dark:text-sky-300 items-center gap-4">
              <div class="text-lg font-semibold">Visual builder integrations</div>
            </div>
            <div class="mt-4 leading-relaxed">
              Yabe Webfont integrates with the most popular visual builders and themes. Use your custom fonts natively in the editor.
            </div>
            <div v-if="page.visual_builders" class="p-2 w-full grid grid-cols-2 sm:grid-cols-4 auto-rows-auto place-items-center mt-4">
              <component v-for="builder in page.visual_builders" :key="builder.name" :is="builder.url ? 'a' : 'div'" :href="builder.url || undefined" class="flex w-full rounded-lg aspect-[4/3] items-center justify-center hover:bg-blue-200/70 dark:hover:bg-gray-500/30">
                <div class="p-5">
                  <div class="flex flex-col items-center text-center">
                    <UIcon v-if="builder.icon?.startsWith('i-custom-')" :name="builder.icon" class="size-10" />
                    <img v-else-if="builder.icon?.startsWith('/')" :src="builder.icon" :alt="builder.name" class="w-10 h-10" />
                    <Icon v-else :name="builder.icon" class="size-10" />
                    <div class="pt-2">
                      {{ builder.name }}
                      <UBadge v-if="builder.pro" label="Pro" variant="subtle" />
                    </div>
                  </div>
                </div>
              </component>
            </div>
          </div>

          <div class="relative col-span-3 row-span-3 p-5 rounded-lg overflow-hidden hover:shadow-md bg-gradient-to-br from-amber-700/20 from-5% via-amber-500/10 via-50% to-amber-100/10">
            <div class="flex text-amber-900 dark:text-amber-300 items-center gap-4">
              <div class="text-lg font-semibold">Google Fonts ready</div>
            </div>
            <div class="mt-4 leading-relaxed">
              Import any of the 1,500+ Google Fonts families to your server in one click.
            </div>
            <img class="absolute hidden sm:block sm:bottom-[-30px] sm:right-[-30px] rounded-lg" src="/tutorial/feature-google-fonts-adobe-fonts.png" alt="Google Fonts ready" />
          </div>

          <div class="relative col-span-3 row-span-6 p-5 rounded-lg overflow-hidden hover:shadow-md bg-gradient-to-br from-purple-700/20 from-5% via-purple-500/10 via-50% to-purple-100/10">
            <div class="flex text-purple-900 dark:text-purple-300 items-center gap-4">
              <div class="text-lg font-semibold">Lightweight and blazingly fast</div>
            </div>
            <div class="mt-4 leading-relaxed">
              Built on the WordPress REST API with a modern JavaScript UI and front-page cache. Small footprint, instant response.
            </div>
            <img class="absolute hidden sm:block sm:bottom-[80px] sm:right-[-150px] rounded-lg scale-150" src="/tutorial/feature-lightweight-blazingly-fast.png" alt="Lightweight and fast" />
          </div>

          <div class="relative col-span-3 row-span-3 p-5 rounded-lg overflow-hidden hover:shadow-md bg-gradient-to-br from-indigo-700/20 from-5% via-indigo-500/10 via-50% to-indigo-100/10">
            <div class="flex text-indigo-900 dark:text-indigo-300 items-center gap-4">
              <div class="text-lg font-semibold">Adobe Fonts support</div>
            </div>
            <div class="mt-4 leading-relaxed">
              Sync your Adobe Fonts Web Projects and use them across your whole site.
            </div>
            <Icon name="simple-icons:adobefonts" class="absolute hidden sm:block sm:bottom-[10px] sm:right-[10px] size-48 opacity-10" />
          </div>

          <div class="relative col-span-9 row-span-3 p-5 rounded-lg overflow-hidden hover:shadow-md bg-gradient-to-br from-teal-700/20 from-5% via-teal-500/10 via-50% to-teal-100/10">
            <div class="flex text-teal-900 dark:text-teal-300 items-center gap-4">
              <div class="text-lg font-semibold">Variable Fonts</div>
            </div>
            <div class="mt-4 leading-relaxed">
              A single font file can hold multiple stylistic variations — Yabe Webfont supports them natively.
            </div>
            <img class="absolute hidden sm:block sm:bottom-[-620px] sm:right-[-120px] rounded-lg scale-80" src="/tutorial/feature-variable-fonts.png" alt="Variable Fonts" />
          </div>
        </div>
      </div>
    </UPageSection>

    <UPageSection :title="page.features.title" :description="page.features.description">
      <UPageGrid>
        <UPageCard v-for="(item, index) in page.features.items" :key="index" v-bind="item" spotlight spotlight-color="primary" :ui="{ leadingIcon: 'size-6 text-(--ui-text)' }" />
      </UPageGrid>
    </UPageSection>

    <UPageSection v-if="page.testimonials" id="testimonials" :headline="page.testimonials.headline" :title="page.testimonials.title" :description="page.testimonials.description" :ui="{
      root: 'bg-gradient-to-b border-t border-(--ui-border) from-(--ui-bg-muted) dark:from-(--ui-bg-muted)/40 to-(--ui-bg)',
      title: 'text-left',
      headline: 'justify-start',
      description: 'text-left',
      links: 'justify-start'
    }">
      <div class="mx-auto mt-8 flow-root max-w-2xl sm:mt-12 lg:mx-0 lg:max-w-none">
        <div class="-mt-8 sm:-mx-4 sm:columns-2 sm:text-[0] lg:columns-3">
          <div v-for="(item, index) in page.testimonials.items" :key="`${item.author?.name || 'review'}-${index}`" class="pt-8 sm:inline-block sm:w-full sm:px-4">
            <figure class="group rounded-2xl bg-gray-50 p-6 text-sm/6 ring-1 ring-inset ring-gray-200/70 transition-all duration-200 hover:bg-white hover:shadow-lg hover:shadow-gray-900/5 hover:ring-indigo-200 dark:bg-white/5 dark:ring-white/10 dark:hover:bg-white/10 dark:hover:ring-indigo-400/40">
              <blockquote class="text-gray-900 dark:text-gray-100">
                <div v-if="item.quote_title" class="mb-3 text-xs font-semibold uppercase tracking-[0.12em] text-gray-600 transition-colors group-hover:text-indigo-600 dark:text-gray-300 dark:group-hover:text-indigo-300">
                  {{ item.quote_title }}
                </div>
                <p>"{{ item.quote }}"</p>
              </blockquote>

              <a v-if="item.author?.to" :href="item.author.to" :target="item.author.target || '_blank'" rel="noopener noreferrer" class="mt-6 flex items-center gap-x-3">
                <img :src="item.author.avatar.src" :alt="item.author.name" class="size-10 rounded-full bg-gray-50 dark:bg-gray-800 object-cover" />
                <div>
                  <div class="font-semibold text-gray-900 transition-colors group-hover:text-indigo-700 dark:text-white dark:group-hover:text-indigo-300">
                    {{ item.author.name }}
                  </div>
                  <div class="text-xs text-gray-600 dark:text-gray-400">
                    {{ item.author.description || 'WordPress reviewer' }}
                  </div>
                </div>
              </a>

              <div v-else class="mt-6 flex items-center gap-x-3">
                <img :src="item.author.avatar.src" :alt="item.author.name" class="size-10 rounded-full bg-gray-50 dark:bg-gray-800 object-cover" />
                <div>
                  <div class="font-semibold text-gray-900 transition-colors group-hover:text-indigo-700 dark:text-white dark:group-hover:text-indigo-300">
                    {{ item.author.name }}
                  </div>
                  <div class="text-xs text-gray-600 dark:text-gray-400">
                    {{ item.author.description || 'WordPress reviewer' }}
                  </div>
                </div>
              </div>
            </figure>
          </div>
        </div>
      </div>

      <div class="text-center mt-8">
        <a href="https://wordpress.org/support/plugin/yabe-webfont/reviews/?filter=5/#new-post" target="_blank" class="underline text-(--ui-text-muted) text-sm">
          Tell us what you think about Yabe Webfont
        </a>
      </div>
    </UPageSection>

    <UPageSection id="pricing" v-bind="pricing" :title="pricing.hero.title" headline="" :description="pricing.hero.description" :ui="{
      root: 'border-y border-(--ui-border)',
    }">
      <div class="flex flex-col bg-(--ui-bg) gap-8 lg:gap-0">
        <UPricingPlans compact>
          <UPricingPlan v-for="(plan, index) in pricing.plans" :key="index" :title="plan.title" :description="plan.description" :price="plan.price" :discount="plan.discount" :billing-period="plan.billing_period" :billing-cycle="plan.billing_cycle" :variant="plan.highlight ? 'soft' : 'outline'" :class="['lg:rounded-none', { 'border-2 lg:border lg:border-x-0 border-(--ui-primary) lg:border-(--ui-border)': plan.highlight }]" :features="plan.features" :button="plan.button" :badge="plan.badge">
            <template #features>
              <li v-for="(feature, featIdx) in plan.features" :key="featIdx" class="flex items-center gap-2 min-w-0">
                <UIcon :name="typeof feature === 'string' ? 'i-lucide-circle-check' : feature.icon" :class="[
                  'size-5 text-(--ui-primary) shrink-0',
                  typeof feature === 'string' ? '' : feature.icon_class
                ]" />
                <MDC :value="typeof feature === 'string' ? feature : feature.title" unwrap="p" :class="[
                  'text-sm truncate text-(--ui-text-toned)',
                  typeof feature === 'string' ? '' : feature.title_class
                ]" />
              </li>
            </template>

          </UPricingPlan>
        </UPricingPlans>
      </div>

      <div class="badges__list grid grid-cols-1 sm:grid-cols-2 md:grid-cols-4 gap-4 *:ring-1 *:ring-gray-200 dark:*:ring-gray-800 *:bg-gray-100/50 dark:*:bg-gray-800/50">
        <div class="badges__item text-center">
          <div class="text-2xl flex items-center">
            <Icon name="ri:exchange-dollar-fill" class="text-[#29b077]" />
          </div>
          <div class="text-xs text-left">
            <strong>14-day Money</strong>
            <br> Back Guarantee
          </div>
        </div>

        <div class="badges__item">
          <div class="text-2xl flex items-center">
            <Icon name="fa6-solid:headset" />
          </div>
          <div class="text-xs text-left">
            <div class="flex text-[#feb82c] mb-1">
              <Icon v-for="i in 5" name="fa6-solid:star" />
            </div>
            <strong>Support</strong> Quality
          </div>
        </div>

        <div class="badges__item">
          <div class="text-2xl flex items-center text-rose-600">
            <UIcon name="i-ph-smiley-fill" class="text-rose-600" />
          </div>
          <div class="text-xs text-left">
            <strong>{{ stats ? `${(stats.edd_happyCustomers || 0) + (stats.wp_active_installs || 0)}+` : '—' }} Happy</strong>
            Users
          </div>
        </div>

        <div class="badges__item">
          <div class="text-2xl flex items-center text-[#4e6fc3]">
            <Icon name="fa6-brands:wordpress" />
          </div>
          <div class="text-xs text-left">
            <strong>{{ stats?.edd_activeSites ? `${stats.edd_activeSites}+` : '—' }} Active</strong>
            Sites
          </div>
        </div>
      </div>
    </UPageSection>

    <UPageSection id="faq" v-bind="faq.faq" class="scroll-mt-(--ui-header-height)" :ui="{ container: 'relative' }">
      <div aria-hidden="true" class="hidden lg:block absolute z-[-1] border-x border-(--ui-border) inset-0 mx-4 sm:mx-6 lg:mx-8" />

      <UPageAccordion trailing-icon="lucide:plus" :items="(faq.faq.items as any[])" :ui="{
        item: 'border-none',
        trigger: 'mb-2 border-0 group px-4 transform-gpu rounded-lg bg-elevated/60 will-change-transform hover:bg-muted/50',
        trailingIcon: 'group-data-[state=closed]:rotate-0 group-data-[state=open]:rotate-135'
      }" class="max-w-4xl mx-auto">
        <template #body="{ item: _item }">
          <MDC :value="_item.content" unwrap="p" class="px-4" />
        </template>
      </UPageAccordion>

    </UPageSection>
  </div>
</template>

<style scoped>
.landing-grid {
  background-size: 100px 100px;
  background-image:
    linear-gradient(to right, var(--color-gray-200) 1px, transparent 1px),
    linear-gradient(to bottom, var(--color-gray-200) 1px, transparent 1px);
}

.dark {
  .landing-grid {
    background-image:
      linear-gradient(to right, var(--color-gray-800) 1px, transparent 1px),
      linear-gradient(to bottom, var(--color-gray-800) 1px, transparent 1px);
  }
}

.feature__icon {
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 40px;
  width: 64px;
  height: 64px;
  background-color: #f9f8fa;
  border-radius: 50%
}

.dark .feature__icon {
  background-color: rgb(255 255 255 / 0.05);
}

.feature__item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px
}

.feature__icon svg {
  width: 40px;
  height: 40px;
}

.badges__list {
  margin-left: auto;
  margin-right: auto;
  gap: 10px;
}

.badges__item {
  display: flex;
  gap: 8px;
  align-items: center;
  padding-left: 16px;
  padding-right: 16px;
  padding-top: 6px;
  padding-bottom: 6px;
  max-width: 180px;
  border-radius: 1e9em;
  width: 100%;
}

.bento {
  aspect-ratio: 1 / 1.25;
}

.gdpr-highlight {
  animation: gdpr-color-cycle 8s ease-in-out infinite;
}

@keyframes gdpr-color-cycle {
  0%, 100% { background-color: rgb(220 252 231); }
  25% { background-color: rgb(219 234 254); }
  50% { background-color: rgb(237 233 254); }
  75% { background-color: rgb(254 243 199); }
}

.dark .gdpr-highlight {
  animation: gdpr-color-cycle-dark 8s ease-in-out infinite;
}

@keyframes gdpr-color-cycle-dark {
  0%, 100% { background-color: rgb(22 101 52); }
  25% { background-color: rgb(30 58 138); }
  50% { background-color: rgb(55 48 107); }
  75% { background-color: rgb(120 53 15); }
}
</style>
