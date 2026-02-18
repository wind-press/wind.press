<script setup lang="ts">
import { joinURL } from 'ufo'

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

const site = useSiteConfig()
const title = 'WindPress: The Tailwind CSS integration plugin for WordPress'
useSeoMeta({
  title,
  titleTemplate: '%s'
})

if (import.meta.server) {
  const description = 'Use Tailwind CSS within the block editor, page builders, plugins, and themes — no build step is required'
  useSeoMeta({
    ogTitle: title,
    description: description,
    ogDescription: description,
    ogImage: joinURL(site.url, '/new-social.png'),
    twitterImage: joinURL(site.url, '/new-social.png')
  })
}

const isMobile = ref(false)
onMounted(() => {
  isMobile.value = window.innerWidth < 768
})
</script>

<template>
  <div v-if="page">
    <UPageHero :title="page.hero.title" :description="page.hero.description" :links="page.hero.links" :ui="{ title: 'text-4xl sm:text-5xl', root: 'mx-auto max-w-7xl', description: 'sm:text-lg', container: 'py-24 sm:py-32 lg:py-40' }">
      <div class="absolute inset-0 landing-grid z-[-1] [mask-image:radial-gradient(100%_100%_at_top_right,white,transparent)]"></div>
      <SkyBg />

      <template #headline>
        <UBadge v-if="page.hero.cta" variant="subtle" size="lg" class="relative rounded-full font-semibold">
          <NuxtLink :to="page.hero.cta.to" target="_blank" class="focus:outline-none" tabindex="-1">
            <span class="absolute inset-0" aria-hidden="true"></span>
          </NuxtLink>

          {{ page.hero.cta.label }}

          <UIcon v-if="page.hero.cta.icon" :name="page.hero.cta.icon" class="ml-1 w-4 h-4 pointer-events-none" />
        </UBadge>
      </template>

      <template #description>
        {{ page.hero.description }}

        <div class="mt-12 mb-8">
          <div class="feature__container grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-6 auto-cols-auto gap-x-4 gap-y-8 sm:gap-11">
            <div class="feature__item">
              <div class="feature__icon"><svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 54 33" class="">
                  <g clip-path="url(#prefix__clip0)">
                    <path fill="#38bdf8" fill-rule="evenodd" d="M27 0c-7.2 0-11.7 3.6-13.5 10.8 2.7-3.6 5.85-4.95 9.45-4.05 2.054.513 3.522 2.004 5.147 3.653C30.744 13.09 33.808 16.2 40.5 16.2c7.2 0 11.7-3.6 13.5-10.8-2.7 3.6-5.85 4.95-9.45 4.05-2.054-.513-3.522-2.004-5.147-3.653C36.756 3.11 33.692 0 27 0zM13.5 16.2C6.3 16.2 1.8 19.8 0 27c2.7-3.6 5.85-4.95 9.45-4.05 2.054.514 3.522 2.004 5.147 3.653C17.244 29.29 20.308 32.4 27 32.4c7.2 0 11.7-3.6 13.5-10.8-2.7 3.6-5.85 4.95-9.45 4.05-2.054-.513-3.522-2.004-5.147-3.653C23.256 19.31 20.192 16.2 13.5 16.2z" clip-rule="evenodd"></path>
                  </g>
                  <defs>
                    <clipPath id="prefix__clip0">
                      <path fill="#fff" d="M0 0h54v32.4H0z"></path>
                    </clipPath>
                  </defs>
                </svg></div>
              <div class="feature__title"> Tailwind CSS <span class="font-bold font-mono">3.x</span> and <span class="font-bold font-mono">4.x</span> ready </div>
            </div>
            <div class="feature__item">
              <div class="feature__icon fg:gray-40 round">
                <div class="flex -ml-6 [&>:not([hidden])~:not([hidden])]:mr-0 [&>:not([hidden])~:not([hidden])]:-ml-4 [&>*]:inline-block [&>*]:w-10 [&>*]:h-10">
                  <img src="https://upload.wikimedia.org/wikipedia/commons/e/e1/Google_Chrome_icon_%28February_2022%29.svg" alt="Google Chromo logo">
                  <img src="https://upload.wikimedia.org/wikipedia/commons/5/52/Safari_browser_logo.svg" alt="Apple Safari logo">
                  <img src="https://upload.wikimedia.org/wikipedia/commons/9/98/Microsoft_Edge_logo_%282019%29.svg" alt="Microsoft Edge logo">
                  <img src="https://upload.wikimedia.org/wikipedia/commons/a/a0/Firefox_logo%2C_2019.svg" alt="Mozilla Firefox logo">
                </div>
              </div>
              <div class="feature__title"> Run in the browser; No server is needed </div>
            </div>
            <div class="feature__item">
              <div class="feature__icon fg:crimson-80 text-rose-600">
                <Icon name="fa6-regular:face-smile-wink" />
              </div>
              <div class="feature__title"> Easy to use and intuitive </div>
            </div>
            <div class="feature__item">
              <div class="feature__icon text-sky-600">
                <Icon name="fa6-solid:feather-pointed" />
              </div>
              <div class="feature__title"> Lightweight and blazingly fast </div>
            </div>
            <div class="feature__item">
              <div class="feature__icon text-[#4e6fc3]">
                <Icon name="fa6-brands:wordpress" />
              </div>
              <div class="feature__title"> Visual builders integration </div>
            </div>
            <div class="feature__item">
              <div class="feature__icon text-lime-600">
                <Icon name="fa6-solid:play" />
              </div>
              <div class="feature__title"> Zero-configuration yet customizable </div>
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
            <Icon name="fa6-solid:face-smile" />
          </div>
          <div class="text-xs text-left">
            <strong>{{ stats.edd_happyCustomers + stats.wp_active_installs }}+ Happy</strong>
            Users
          </div>
        </div>

        <div class="badges__item">
          <div class="text-2xl flex items-center text-[#4e6fc3]">
            <Icon name="fa6-brands:wordpress" />
          </div>
          <div class="text-xs text-left">
            <strong>{{ stats.edd_totalDownloads }}+ Download</strong>
            <br> & Counting
          </div>
        </div>
      </div>

    </UPageHero>

    <UPageSection :ui="{ container: '!py-0' }">
      <div class="bg-gray-900/5 dark:bg-white/5 ring-1 ring-inset ring-gray-900/10 dark:ring-white/10 rounded-xl lg:-m-4 p-4">
        <img src="https://ps.w.org/windpress/assets/screenshot-4.png" alt="WindPress screenshot" class="w-full h-auto rounded-lg border border-dashed border-gray-900/10 dark:border-white/10" />
      </div>
    </UPageSection>

    <UPageSection>
      <div id="features" class="block-features flex flex-col items-center">
        <div class="px-2 py-1 mb-2 rounded-lg bg-sky-100 text-sm text-sky-500 dark:bg-sky-600 dark:text-sky-100 text-center uppercase font-bold tracking-widest">
          Fully-featured
        </div>
        <h2 class="mb-6 text-3xl font-semibold text-center">
          Made for Designers and Developers
        </h2>
        <div class="flex justify-center text-center">
          <p class="max-w-[60%] text-lg leading-relaxed">
            At WindPress, we created an <strong>easy-to-use and intuitive Tailwind CSS integration</strong> for WordPress to streamline your workflow.
          </p>
        </div>

        <div class="bento flex flex-col sm:grid sm:grid-cols-12 sm:grid-rows-12 gap-4 w-full mt-16 dark:*:!bg-none dark:*:!bg-gray-800/50 dark:[&_.bento\_\_slot-title]:text-white">
          <div class="bento__slot relative col-span-12 row-span-3 p-5 rounded-lg overflow-hidden hover:shadow-md bg-gradient-to-br from-gray-700/20 from-5% via-gray-500/10 via-50% to-gray-100/10">
            <div class="bento__slot-title flex text-gray-900 items-center gap-4">
              <a>
                <div class="text-lg font-semibold">
                  The Wizard
                </div>
              </a>
            </div>
            <div class="bento__slot-description mt-4 leading-relaxed">
              <div class="sm:w-1/4">
                A simple and intuitive visual interface to customize your Tailwind CSS configuration without coding.
              </div>
            </div>
            <img class="absolute hidden sm:block sm:top-[-50px] sm:right-[-220px] rounded-lg scale-75 sm:pt-0 pt-5" src="/assets/landing/feature-bento-wizard-color.webp" alt="feature: wizard colors" />
          </div>
          <div class="bento__slot relative col-span-3 row-span-3 p-5 rounded-lg overflow-hidden hover:shadow-md bg-gradient-to-br from-lime-700/20 from-5% via-lime-500/10 via-50% to-lime-100/10">
            <div class="bento__slot-title flex text-lime-900 items-center gap-4">
              <a href="/docs/guide/configuration/tailwind-version">
                <div class="text-lg font-semibold">
                  Tailwind CSS version 3.x and 4.x ready
                </div>
              </a>
            </div>
            <div class="bento__slot-description mt-4 leading-relaxed">
              <div>
                WindPress supports both Tailwind CSS v3.x and v4.x. The flexibility to choose the Tailwind CSS version that meet your WordPress project requirements.
              </div>
            </div>
            <img class="absolute hidden sm:block sm:bottom-[-30px] sm:right-[-30px]" src="/assets/landing/tailwindcss-mark.svg" alt="Tailwind CSS version 3.x and 4.x ready" />
          </div>
          <div id="integrations" class="bento__slot relative col-span-6 row-span-6 p-5 rounded-lg overflow-hidden hover:shadow-md bg-gradient-to-br from-sky-700/20 from-5% via-sky-500/10 via-50% to-sky-100/10">
            <div class="bento__slot-title flex text-sky-900 items-center gap-4">
              <div class="text-lg font-semibold">Visual builder integrations</div>
            </div>
            <div class="bento__slot-description mt-4 leading-relaxed">
              <div>
                WindPress integrates with the most popular visual builders to make using Tailwind CSS in your WordPress website easier. More integrations are coming soon.
              </div>
            </div>
            <div>
              <div v-if="page.visual_builders" class="p-2 w-full grid grid-cols-2 sm:grid-cols-4 auto-rows-auto place-items-center">
                <a v-for="builder in page.visual_builders" :href="builder.url" class="flex w-full rounded-lg aspect-square items-center justify-center hover:bg-blue-200/70 dark:hover:bg-gray-500/30">
                  <div class="p-5">
                    <div class="flex flex-col items-center text-center">
                      <UIcon v-if="builder.icon?.startsWith('i-custom-')" :name="builder.icon" class="size-10" />
                      <img v-else :src="builder.icon" :alt="builder.name" class="w-10 h-10" />
                      <div class="pt-2">
                        {{ builder.name }}
                        <UBadge v-if="builder.pro" label="Pro" variant="subtle" class="" />
                      </div>
                    </div>
                  </div>
                </a>
              </div>
            </div>
          </div>
          <div class="bento__slot relative col-span-3 row-span-3 p-5 rounded-lg overflow-hidden hover:shadow-md bg-gradient-to-br from-indigo-700/20 from-5% via-indigo-500/10 via-50% to-indigo-100/10">
            <div class="bento__slot-title flex text-indigo-900 items-center gap-4">
              <a href="/docs/guide/configuration/tw-4/file-main-css">
                <div class="text-lg font-semibold">
                  Zero configuration yet fully customizable
                </div>
              </a>
            </div>
            <div class="bento__slot-description mt-4 leading-relaxed">
              <div>
                Start using it instantly without any configuration, and customize it as needed.
              </div>
            </div>
            <img class="absolute hidden sm:block sm:right-[-70px] sm:bottom-[-25px] rounded-lg sm:pt-0 pt-5 scale-[1.3]" src="/assets/landing/feature-bento-customizable.webp" alt="feature: customizable" />
          </div>

          <div class="bento__slot relative col-span-3 row-span-6 p-5 rounded-lg overflow-hidden hover:shadow-md bg-gradient-to-br from-teal-700/20 from-5% via-teal-500/10 via-50% to-teal-100/10">
            <div class="bento__slot-title flex text-teal-900 items-center gap-4">
              <a href="/docs/guide/concepts/cache#production">
                <div class="text-lg font-semibold">
                  Compile in the browser; No server is required
                </div>
              </a>
            </div>
            <div class="bento__slot-description mt-4 leading-relaxed">
              <div>
                Build a single static final CSS without a server. Use Tailwind CSS on any WordPress server, even shared hosting. All processing is done on your browser, and no data transfer occurs.
              </div>
            </div>
            <img class="absolute hidden sm:block sm:top-[50%] sm:left-[20%] rounded-lg sm:pt-0 pt-5 scale-[1.25]" src="/assets/landing/feature-bento-compile.webp" alt="feature: compile" />
            <img class="absolute hidden sm:block z-[-1] w-36 h-36 sm:top-[-40px] sm:right-[-40px] opacity-30 dark:opacity-60" src="https://upload.wikimedia.org/wikipedia/commons/5/52/Safari_browser_logo.svg" alt="Apple Safari logo" />
            <img class="absolute hidden sm:block z-[-1] w-36 h-36 sm:top-[-40px] sm:left-[-40px] opacity-30 dark:opacity-60" src="https://upload.wikimedia.org/wikipedia/commons/9/98/Microsoft_Edge_logo_%282019%29.svg" alt="Microsoft Edge logo" />
            <img class="absolute hidden sm:block z-[-1] w-36 h-36 sm:bottom-[-40px] sm:right-[-40px] opacity-30 dark:opacity-60" src="https://upload.wikimedia.org/wikipedia/commons/a/a0/Firefox_logo%2C_2019.svg" alt="Mozilla Firefox logo" />
            <img class="absolute hidden sm:block z-[-1] w-36 h-36 sm:bottom-[-40px] sm:left-[-40px] opacity-30 dark:opacity-60" src="https://upload.wikimedia.org/wikipedia/commons/e/e1/Google_Chrome_icon_%28February_2022%29.svg" alt="Google Chrome logo" />
          </div>
          <div class="bento__slot relative col-span-3 row-span-3 p-5 rounded-lg overflow-hidden hover:shadow-md bg-gradient-to-br from-purple-700/20 from-5% via-purple-500/10 via-50% to-purple-100/10">
            <div class="bento__slot-title flex text-purple-900 items-center gap-4">
              <div class="text-lg font-semibold">Lightweight and blazingly fast</div>
            </div>
            <div class="bento__slot-description mt-4 leading-relaxed">
              <div>
                Built with WordPress REST API and a modern JavaScript framework for a responsive user experience. It has a small footprint and won't slow down your site.
              </div>
            </div>
            <img class="absolute sm:right-[-70px] sm:bottom-[-65px] scale-75 sm:pt-0 pt-5" src="/assets/landing/feature-bento-wp-rest-api.webp" alt="feature: lightweight and blazingly fast" />
          </div>

          <div class="bento__slot relative col-span-3 row-span-3 p-5 rounded-lg overflow-hidden hover:shadow-md bg-gradient-to-br from-yellow-700/20 from-5% via-yellow-500/10 via-50% to-yellow-100/10">
            <div class="bento__slot-title flex text-yellow-900 items-center gap-4">
              <a href="/docs/guide/configuration/tw-3/file-tailwind-config-js#javascript-package">
                <div class="text-lg font-semibold">JavaScript package support</div>
              </a>
            </div>
            <div class="bento__slot-description mt-4 leading-relaxed">
              <div>
                Add extra functionality to your Tailwind CSS configuration with JavaScript packages through npm. Enrich Tailwind CSS with external plugins.
              </div>
            </div>
            <img class="absolute sm:right-[0px] sm:bottom-[0px] rounded-lg sm:pt-0 pt-5 scale-[.9]" src="/assets/landing/feature-bento-javascript-package.webp" alt="feature: javascript package" />
          </div>

          <div class="bento__slot relative col-span-6 row-span-3 p-5 rounded-lg overflow-hidden hover:shadow-md bg-gradient-to-br from-rose-700/20 from-5% via-rose-500/10 via-50% to-rose-100/10">
            <div class="bento__slot-title flex text-rose-900 items-center gap-4">
              <a href="/docs/guide/concepts/simple-file-system">
                <div class="text-lg font-semibold">Simple File System</div>
              </a>
            </div>
            <div class="bento__slot-description mt-4 leading-relaxed">
              <div>
                A virtual file system that helps organize your custom CSS and JavaScript files, functioning like a file manager for your Tailwind CSS project.
              </div>
            </div>
            <img class="absolute hidden sm:block sm:left-[30px] sm:bottom-[-90px] scale-[.95] rounded-lg sm:pt-0 pt-5" src="/assets/landing/feature-bento-simple-file-system.webp" alt="feature: autocomplete" />
          </div>
        </div>
      </div>
    </UPageSection>

    <UPageSection :title="page.features.title" :description="page.features.description">
      <UPageGrid>
        <UPageCard v-for="(item, index) in page.features.items" :key="index" v-bind="item" spotlight spotlight-color="primary" :ui="{ leadingIcon: 'size-6 text-(--ui-text)' }" />
      </UPageGrid>
    </UPageSection>

    <UPageSection id="testimonials" v-bind="page.testimonials" :title="page.testimonials.title" :headline="page.testimonials.headline" :description="page.testimonials.description" :ui="{
      root: 'bg-gradient-to-b border-t border-(--ui-border) from-(--ui-bg-muted) dark:from-(--ui-bg-muted)/40 to-(--ui-bg)',
      title: 'text-left',
      headline: 'justify-start',
      description: 'text-left',
      links: 'justify-start'
    }">
      <UCarousel v-slot="{ item }" :items="page.testimonials.items" dots wheel-gestures arrows loop autoplay class="min-w-0" :ui="{
        container: 'ms-0 items-stretch',
        item: 'min-w-0 shrink-0 sm:basis-1/2 p-2',
        arrows: 'hidden 2xl:block',
        root: 'flex flex-col gap-12 sm:gap-0'
      }">
        <LandingTestimonialCard v-bind="item" />
      </UCarousel>

      <div class="text-center mt-6 sm:mt-0">
        <a href="https://wordpress.org/support/plugin/windpress/reviews/?filter=5/#new-post" target="_blank" class="underline text-(--ui-text-muted)">
          Tell us what you think about WindPress
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
        <!-- <div class="badges__item text-center">
          <div class="text-2xl flex items-center">
            <Icon name="fa6-solid:hand-holding-dollar" class="text-[#29b077]" />
          </div>
          <div class="text-xs text-left">
            <strong>Free</strong>
            Forever
          </div>
        </div> -->

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
            <Icon name="fa6-solid:face-smile" />
          </div>
          <div class="text-xs text-left">
            <strong>{{ stats.edd_happyCustomers + stats.wp_active_installs }}+ Happy</strong>
            Users
          </div>
        </div>

        <div class="badges__item">
          <div class="text-2xl flex items-center text-[#4e6fc3]">
            <Icon name="fa6-brands:wordpress" />
          </div>
          <div class="text-xs text-left">
            <strong>{{ stats.edd_activeSites }}+ Active</strong> Sites
          </div>
        </div>
      </div>
    </UPageSection>

    <UPageSection id="faq" v-bind="faq.faq" class="scroll-mt-(--ui-header-height)" :ui="{ container: 'relative' }">
      <div aria-hidden="true" class="hidden lg:block absolute z-[-1] border-x border-(--ui-border) inset-0 mx-4 sm:mx-6 lg:mx-8" />
      <!-- <UPageAccordion multiple :items="(faq.faq.items as any[])" class="max-w-4xl mx-auto">
        <template #body="{ item, index }">
          <MDC :value="item.content" unwrap="p" :cache-key="`pro-pricing-faq-${index}-content`" />
        </template>
      </UPageAccordion> -->

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
  /* border: 1px solid rgba(0, 0, 0, 0.2); */
  max-width: 180px;
  border-radius: 1e9em;
  width: 100%;
}

.bento {
  aspect-ratio: 1 / 1.25;
}

.aspect-square {
  aspect-ratio: 1 / 1;
}
</style>