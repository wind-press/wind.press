<script setup lang="ts">
import { joinURL } from 'ufo'

definePageMeta({
  heroBackground: '-z-10'
})

const [{ data: page }, { data: officialModules }, { data: sponsorGroups }] = await Promise.all([
  useAsyncData('index', () => queryCollection('index').first()),
  useFetch<{ modules: Module[], stats: Stats }>('https://api.nuxt.com/modules', {
    key: 'official-modules',
    transform: res => res.modules
      .filter(module => module.type === 'official')
      .sort((a, b) => b.stats.stars - a.stats.stars)
  }),
  useFetch('https://api.nuxt.com/sponsors', {
    key: 'top-sponsors',
    transform: sponsors => Object.entries(sponsors)
      .filter(([tier]) => ['diamond', 'platinum', 'gold'].includes(tier))
      .map(([tier, sponsors]) => ({
        tier,
        sponsors: sponsors.map(s => ({
          sponsorName: s.sponsorName,
          sponsorLogo: s.sponsorLogo,
          sponsorUrl: s.sponsorUrl
        }))
      }))
  })
])

const stats = useStats()

const videoModalOpen = ref(false)

const site = useSiteConfig()
const title = 'WindPress: The Tailwind CSS integration plugin for WordPress'
useSeoMeta({
  title,
  titleTemplate: '%s'
})

if (import.meta.server) {
  const description = 'Create high-quality web applications with Nuxt, the open source framework that makes full-stack development with Vue.js intuitive.'
  useSeoMeta({
    ogTitle: title,
    description: description,
    ogDescription: description,
    ogImage: joinURL(site.url, '/new-social.jpg'),
    twitterImage: joinURL(site.url, '/new-social.jpg')
  })
}

// const tabs = computed(() => page.value?.hero.tabs.map(tab => ({
//   label: tab.title,
//   icon: tab.icon,
//   slot: tab.title.toLowerCase(),
//   content: tab.content
// })))

const activeBundlerIndex = ref(0)

const groupedFoundationItems = computed(() => {
  const result = [] as Array<{
    id: string
    // TODO: make these types better
    item?: any
    items?: any[]
    classes: string
  }>
  const bundlers = {
    id: 'bundler',
    items: [],
    classes: 'rounded-none'
  }

  page.value.foundation.items.forEach((item, index) => {
    if (item.id === 'bundler') {
      bundlers.items.push(item)
    } else {
      const borderClasses = index === 0
        ? 'max-sm:rounded-t-lg max-sm:rounded-b-none sm:rounded-s-lg sm:rounded-e-none'
        : index === page.value.foundation.items.length - 1
          ? 'max-sm:rounded-t-none max-sm:rounded-b-lg sm:rounded-s-none sm:rounded-e-lg'
          : 'rounded-none'

      result.push({
        id: item.id,
        item: item,
        classes: `${borderClasses} ${item.gradient}`
      })
    }
  })

  if (bundlers.items.length > 0) {
    result.splice(1, 0, bundlers)
  }

  return result
})

const isMobile = ref(false)
onMounted(() => {
  isMobile.value = window.innerWidth < 768
})
</script>

<template>
  <div v-if="page">




    <UPageHero :title="page.hero.title" :description="page.hero.description" :links="page.hero.links" :ui="{ title: 'text-4xl sm:text-5xl', root: 'mx-auto max-w-7xl', description: 'sm:text-lg' }">
      <div class="absolute inset-0 landing-grid z-[-1] [mask-image:radial-gradient(100%_100%_at_top_right,white,transparent)]"></div>

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

        <div class="bento flex flex-col sm:grid sm:grid-cols-12 sm:grid-rows-12 gap-4 w-full mt-16 dark:*:bg-gray-800/50 dark:[&_.bento\_\_slot-title]:text-white">
          <div class="bento__slot relative col-span-12 row-span-3 bg-gray-200/50 p-5 rounded-lg overflow-hidden hover:shadow-md">
            <div class="bento__slot-title flex text-gray-900 items-center gap-4">
              <a>
                <div class="text-lg font-semibold">
                  The Wizard <span class="font-light">(In development)</span>
                </div>
              </a>
            </div>
            <div class="bento__slot-description mt-4 leading-relaxed">
              <div class="sm:w-1/4">
                A simple and intuitive visual interface to customize your Tailwind CSS configuration without coding.
              </div>
            </div>
            <img class="absolute sm:top-[-50px] sm:right-[-220px] rounded-lg scale-75 sm:pt-0 pt-5" src="/assets/landing/feature-bento-wizard-color.png" alt="feature: wizard colors" />
          </div>
          <div class="bento__slot relative col-span-3 row-span-3 bg-lime-700/10 p-5 rounded-lg overflow-hidden hover:shadow-md">
            <div class="bento__slot-title flex text-lime-900 items-center gap-4">
              <a href="/docs/configuration/tailwind-version">
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

          <div class="bento__slot relative col-span-6 row-span-6 bg-blue-200/50 p-5 rounded-lg overflow-hidden hover:shadow-md">
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
                <a v-for="builder in page.visual_builders" :href="builder.url" class="flex w-full rounded-lg aspect-square items-center justify-center hover:bg-blue-200 dark:hover:bg-gray-500/30">
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
          <div class="bento__slot relative col-span-3 row-span-3 bg-indigo-200/50 p-5 rounded-lg overflow-hidden hover:shadow-md">
            <div class="bento__slot-title flex text-indigo-900 items-center gap-4">
              <a href="/docs/configuration/file-main-css">
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
            <img class="absolute sm:right-[-70px] sm:bottom-[-60px] rounded-lg sm:pt-0 pt-5" src="/assets/landing/feature-bento-customizable.png" alt="feature: customizable" />
          </div>

          <div class="bento__slot relative col-span-3 row-span-6 bg-yellow-700/10 p-5 rounded-lg overflow-hidden hover:shadow-md">
            <div class="bento__slot-title flex text-yellow-900 items-center gap-4">
              <a href="/docs/advanced/cache#production">
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
            <img class="absolute hidden sm:block sm:top-[50%] sm:left-[60%] rounded-lg sm:pt-0 pt-5 scale-[2]" src="/assets/landing/feature-bento-compile.png" alt="feature: compile" />
            <img class="absolute hidden sm:block z-[-1] w-36 h-36 sm:top-[-40px] sm:right-[-40px] opacity-50" src="https://upload.wikimedia.org/wikipedia/commons/5/52/Safari_browser_logo.svg" alt="Apple Safari logo" />
            <img class="absolute hidden sm:block z-[-1] w-36 h-36 sm:top-[-40px] sm:left-[-40px] opacity-50" src="https://upload.wikimedia.org/wikipedia/commons/9/98/Microsoft_Edge_logo_%282019%29.svg" alt="Microsoft Edge logo" />
            <img class="absolute hidden sm:block z-[-1] w-36 h-36 sm:bottom-[-40px] sm:right-[-40px] opacity-50" src="https://upload.wikimedia.org/wikipedia/commons/a/a0/Firefox_logo%2C_2019.svg" alt="Mozilla Firefox logo" />
            <img class="absolute hidden sm:block z-[-1] w-36 h-36 sm:bottom-[-40px] sm:left-[-40px] opacity-50" src="https://upload.wikimedia.org/wikipedia/commons/e/e1/Google_Chrome_icon_%28February_2022%29.svg" alt="Google Chrome logo" />
          </div>
          <div class="bento__slot relative col-span-3 row-span-3 bg-purple-200/50 p-5 rounded-lg overflow-hidden hover:shadow-md">
            <div class="bento__slot-title flex text-purple-900 items-center gap-4">
              <div class="text-lg font-semibold">Lightweight and blazingly fast</div>
            </div>
            <div class="bento__slot-description mt-4 leading-relaxed">
              <div>
                Built with WordPress REST API and a modern JavaScript framework for a responsive user experience. It has a small footprint and won't slow down your site.
              </div>
            </div>
            <img class="absolute sm:right-[-70px] sm:bottom-[-65px] scale-75 sm:pt-0 pt-5" src="/assets/landing/feature-bento-wp-rest-api.png" alt="feature: lightweight and blazingly fast" />
          </div>
          <div class="bento__slot relative col-span-3 row-span-3 bg-orange-200/50 p-5 rounded-lg overflow-hidden hover:shadow-md">
            <div class="bento__slot-title flex text-orange-900 items-center gap-4">
              <a href="/docs/advanced/simple-file-system">
                <div class="text-lg font-semibold">Simple File System</div>
              </a>
            </div>
            <div class="bento__slot-description mt-4 leading-relaxed">
              <div>
                A virtual file system that helps organize your custom CSS and JavaScript files, functioning like a file manager for your Tailwind CSS project.
              </div>
            </div>
            <img class="absolute hidden sm:block sm:right-[-80px] sm:bottom-[-20px] scale-150 rounded-lg sm:pt-0 pt-5" src="/assets/landing/feature-bento-simple-file-system.png" alt="feature: autocomplete" />
          </div>
          <div class="bento__slot relative col-span-6 row-span-3 bg-[rgba(242,198,222,0.5)] p-5 rounded-lg overflow-hidden hover:shadow-md">
            <div class="bento__slot-title flex text-orange-900 items-center gap-4">
              <a href="/docs/configuration/file-tailwind-config-js/#javascript-package">
                <div class="text-lg font-semibold">JavaScript package support</div>
              </a>
            </div>
            <div class="bento__slot-description mt-4 leading-relaxed">
              <div>
                Add extra functionality to your Tailwind CSS configuration with JavaScript packages through npm. Enrich Tailwind CSS with external plugins.
              </div>
            </div>
            <img class="absolute sm:right-[-40px] sm:bottom-[-20px] rounded-lg sm:pt-0 pt-5 scale-75" src="/assets/landing/feature-bento-javascript-package.png" alt="feature: javascript package" />
          </div>
        </div>
      </div>
    </UPageSection>






    <UPageSection :title="page.features.title" :description="page.features.description">
      <UPageGrid>
        <UPageCard v-for="(item, index) in page.features.items" :key="index" v-bind="item" spotlight spotlight-color="primary" :ui="{ leadingIcon: 'size-6 text-(--ui-text)' }" />
      </UPageGrid>
    </UPageSection>














    <UPageSection v-bind="page.testimonials" :title="page.testimonials.title" :headline="page.testimonials.headline" :description="page.testimonials.description" :ui="{
      root: 'bg-gradient-to-b border-t border-(--ui-border) from-(--ui-bg-muted) dark:from-(--ui-bg-muted)/40 to-(--ui-bg)',
      title: 'text-left',
      headline: 'justify-start',
      description: 'text-left',
      links: 'justify-start'
    }">
      <UCarousel v-slot="{ item }" :items="page.testimonials.items" dots wheel-gestures arrows loop class="min-w-0" :ui="{
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






    <!-- 

    <UPageHero class="relative" orientation="horizontal" :ui="{
      container: '!pb-20 py-24 sm:py-32 lg:py-40',
      title: 'text-5xl sm:text-7xl',
      wrapper: 'lg:min-h-[540px]'
    }">
      <template #headline>
        <NuxtLink :to="page.hero.cta.to">
          <UBadge variant="subtle" size="lg" class="px-3 relative rounded-full font-semibold dark:hover:bg-primary-400/15 dark:hover:ring-primary-700">
            {{ page?.hero.cta.label }}
            <UIcon v-if="page?.hero.cta.icon" :name="page?.hero.cta.icon" class="size-4 pointer-events-none" />
          </UBadge>
        </NuxtLink>
      </template>

      <template #title>
        The Progressive<br><span class="text-(--ui-primary)">Web Framework</span>
      </template>

      <template #description>
        <LazyMDC :value="page?.hero.description" unwrap="p" cache-key="index-hero-description" hydrate-never />
      </template>

      <template #links>
        <div class="flex flex-col gap-4">
          <div class="flex items-center flex-wrap gap-2">
            <UButton to="/docs/getting-started/installation" size="xl">
              Get started
            </UButton>
            <UButton size="xl" color="neutral" variant="subtle" trailing-icon="i-lucide-play-circle" @click="videoModalOpen = true">
              Nuxt in 100 seconds
            </UButton>
          </div>
          <UInputCopy value="npm create nuxt@latest" label="npm create nuxt@latest" size="xl" />
        </div>

        <UModal v-model:open="videoModalOpen" :ui="{ content: 'sm:max-w-4xl lg:max-w-5xl aspect-[16/9]' }">
          <template #content>
            <div class="p-3 h-full">
              <iframe width="100%" height="100%" src="https://www.youtube-nocookie.com/embed/dCxSsr5xuL8" title="Nuxt in 100 Seconds by Fireship" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" allowfullscreen />
            </div>
          </template>
        </UModal>
      </template>

      <UPageCard class="overflow-auto lg:absolute [@media(min-width:2400px)]:relative lg:-mt-16 [@media(min-width:2400px)]:mt-8 right-0 [@media(min-width:2400px)]:right-auto w-screen lg:w-[calc(50%-2rem)] [@media(min-width:2400px)]:w-full max-w-[800px] [@media(min-width:2400px)]:mx-auto rounded-none lg:rounded-l-[calc(var(--ui-radius)*4)] [@media(min-width:2400px)]:rounded-[calc(var(--ui-radius)*4)] -mx-4 sm:-mx-6 lg:mx-0" variant="subtle" :ui="{ container: 'sm:pt-4.5 lg:pr-0 [@media(min-width:2400px)]:px-6 w-full' }">
        <UTabs :items="tabs" :unmount-on-hide="false" :ui="{
          list: 'px-0 bg-transparent lg:pr-4 overflow-x-auto',
          trigger: 'group data-[state=active]:text-(--ui-text-highlighted)',
          indicator: 'bg-(--ui-bg)',
          leadingIcon: 'group-data-[state=active]:text-(--ui-primary) size-4 hidden sm:inline-flex',
          content: 'lg:h-[450px] bg-(--ui-bg) [@media(min-width:2400px)]:border-e [@media(min-width:2400px)]:border-(--ui-border) [@media(min-width:2400px)]:rounded-l-[calc(var(--ui-radius)*1.5)] transition-opacity duration-500 data-[state=inactive]:opacity-0 opacity-100'
        }">
          <template v-for="(tab, index) of tabs" :key="index" #[tab.slot]="{ item }">
            <LazyMDC :value="item.content" :cache-key="`index-hero-tab-${index}`" hydrate-on-idle />
          </template>
        </UTabs>
      </UPageCard>
    </UPageHero>
    <UPageSection :ui="{ container: '!pt-0' }">
      <UPageLogos :marquee="isMobile" :title="page?.logos.title" :ui="{ title: 'text-left text-(--ui-text-muted) font-medium text-lg', logos: 'mt-4' }">
        <Motion v-for="(company, index) in page?.logos.companies" :key="company.alt" as-child :initial="{ opacity: 0, transform: 'translateY(20px)' }" :while-in-view="{ opacity: 1, transform: 'translateY(0)' }" :transition="{ delay: 0.4 + 0.2 * index }" :in-view-options="{ once: true }">
          <div class="opacity-0">
            <UColorModeImage :key="company.alt" :light="company.light" :dark="company.dark" :alt="`${company.alt} logo`" loading="lazy" :height="company.height" :width="company.width" class="h-6 shrink-0 max-w-[140px]" />
          </div>
        </Motion>
      </UPageLogos>
    </UPageSection>
    <UPageSection :title="page?.features.title" :description="page?.features.description" :ui="{
      title: 'text-left',
      description: 'text-left',
      root: 'bg-gradient-to-b border-t border-(--ui-border) from-(--ui-bg-muted) dark:from-(--ui-bg-muted)/40 to-(--ui-bg)',
      features: 'xl:grid-cols-4 lg:gap-10'
    }">
      <template #features>
        <Motion v-for="(feature, index) in page.features.features" :key="feature.title" as="li" :initial="{ opacity: 0, transform: 'translateY(10px)' }" :while-in-view="{ opacity: 1, transform: 'translateY(0)' }" :transition="{ delay: 0.1 * index }" :in-view-options="{ once: true }">
          <UPageFeature v-bind="feature" orientation="vertical" />
        </Motion>
        <Motion as="li" :initial="{ opacity: 0, transform: 'translateY(10px)' }" :while-in-view="{ opacity: 1, transform: 'translateY(0)' }" :transition="{ delay: 0.1 * page.features.features.length }" :in-view-options="{ once: true }" class="flex flex-col justify-center gap-4 p-4 bg-(--ui-bg-muted)/50 h-full">
          <span class="text-lg font-semibold">
            {{ page.features.cta.title }}
          </span>
          <div>
            <UButton :to="page.features.cta.to" :label="page.features.cta.label" trailing :icon="page.features.cta.icon" />
          </div>
        </Motion>
      </template>
    </UPageSection>

    <UPageSection :ui="{
      root: 'bg-gradient-to-b border-t border-(--ui-border) from-(--ui-bg-muted) dark:from-(--ui-bg-muted)/40 to-(--ui-bg)'
    }">
      <template #title>
        <LazyMDC :value="page.foundation.title" unwrap="p" cache-key="index-foundation-title" hydrate-never />
      </template>
      <template #description>
        <LazyMDC :value="page.foundation.description" unwrap="p" cache-key="index-foundation-description" hydrate-never />
      </template>

      <div class="grid grid-cols-1 sm:grid-cols-3">
        <template v-for="(group, groupIndex) in groupedFoundationItems" :key="groupIndex">
          <UPageCard v-if="group.id !== 'bundler'" :title="group.item.title" :description="group.item.description" class="h-full" :ui="{
            root: `${group.classes} ring-0 border border-(--ui-border) ${groupIndex === 0 ? 'sm:border-r-0 max-sm:border-b-0' : groupIndex === groupedFoundationItems.length - 1 ? 'sm:border-l-0 max-sm:border-t-0' : 'max-sm:border-y-0'}`,
            title: 'text-lg font-semibold'
          }">
            <template #leading>
              <UIcon :name="group.item.logo" class="size-6" />
            </template>
            <ULink :to="group.item.link.to" :style="{ color: group.item.color }">
              {{ group.item.link.label }}
            </ULink>
          </UPageCard>

          <UPageCard v-else :title="group.items[activeBundlerIndex].title" :description="group.items[activeBundlerIndex].description" class="h-full ring-0 border border-(--ui-border)" :ui="{
            root: `${group.classes} ${group.items[activeBundlerIndex].gradient}`,
            title: 'text-lg font-semibold'
          }">
            <template #leading>
              <div class="flex items-center space-x-2">
                <div v-for="(bundler, bIndex) in group.items" :key="bIndex" class="size-7 justify-center inline-flex items-end">
                  <UIcon :name="bundler.logo" class="cursor-pointer transition-all duration-150 ease-in-out" :class="bIndex === activeBundlerIndex
                    ? 'size-7 opacity-100'
                    : 'size-6 opacity-50 grayscale hover:size-7'" @click="activeBundlerIndex = bIndex" />
                </div>
              </div>
            </template>
            <ULink :to="group.items[activeBundlerIndex].link.to" :style="{ color: group.items[activeBundlerIndex].color }">
              {{ group.items[activeBundlerIndex].link.label }}
            </ULink>
          </UPageCard>
        </template>
      </div>
    </UPageSection>
    <UPageCTA :description="page.testimonial.quote" variant="subtle" class="rounded-none" :ui="{
      container: 'sm:py-12 lg:py-12 sm:gap-8',
      description: '!text-base text-balance before:content-[open-quote] before:text-5xl lg:before:text-7xl before:inline-block before:text-(--ui-text-dimmed) before:absolute before:-ml-6 lg:before:-ml-10 before:-mt-2 lg:before:-mt-4 after:content-[close-quote] after:text-5xl lg:after:text-7xl after:inline-block after:text-(--ui-text-dimmed) after:absolute after:mt-1 lg:after:mt-0 after:ml-1 lg:after:ml-2'
    }">
      <UUser v-bind="page.testimonial.author" size="xl" class="justify-center" />
    </UPageCTA>
    <UPageSection :title="page.stats.title" :description="page.stats.description" class="relative" :ui="{
      root: 'bg-gradient-to-b border-t border-(--ui-border) from-(--ui-bg-muted) dark:from-(--ui-bg-muted)/40 to-(--ui-bg)'
    }">
      <div class="flex flex-col md:flex-row gap-4">
        <div class="md:w-1/4 flex flex-col gap-4">
          <UPageCard class="flex-1" variant="subtle" to="https://npm.chart.dev/nuxt">
            <div class="flex items-center gap-3">
              <div class="rounded-[calc(var(--ui-radius)*2)] bg-(--ui-bg) p-2 flex items-center justify-center border border-(--ui-border)">
                <UIcon name="i-simple-icons-npm" class="text-red-500 size-6" />
              </div>
              <div class="flex flex-col">
                <span class="font-semibold text-lg text-(--ui-text-highlighted)">
                  {{ formatNumber(stats.monthlyDownloads) }}
                </span>
                <p class="text-sm">
                  Monthly downloads
                </p>
              </div>
            </div>
          </UPageCard>

          <UPageCard class="flex-1" variant="subtle" to="https://go.nuxt.com/github">
            <div class="flex items-center gap-2">
              <div class="rounded-lg bg-(--ui-bg) p-2 flex items-center justify-center border border-(--ui-border)">
                <UIcon name="i-simple-icons-github" class="size-6" />
              </div>
              <div class="flex flex-col">
                <span class="font-semibold text-lg text-(--ui-text-highlighted)">
                  {{ formatNumber(stats.stars) }}
                </span>
                <p class="text-sm">
                  GitHub Stars
                </p>
              </div>
            </div>
          </UPageCard>
        </div>

        <div class="md:w-1/2">
          <UPageCard class="h-full" variant="subtle" to="https://go.nuxt.com/github">
            <div class="flex flex-col items-center justify-around h-full">
              <span class="text-xl font-semibold">
                {{ page.stats.community.title }}
              </span>
              <p class="text-(--ui-text-muted) text-center">
                {{ page.stats.community.description }}
              </p>
              <UButton class="mt-4 w-fit" v-bind="page.stats.cta" />
            </div>
          </UPageCard>
        </div>

        <div class="md:w-1/4 flex flex-col gap-4">
          <UPageCard class="flex-1" variant="subtle" to="https://go.nuxt.com/x">
            <div class="flex items-center gap-2">
              <div class="rounded-lg bg-(--ui-bg) p-2 flex items-center justify-center border border-(--ui-border)">
                <UIcon name="i-simple-icons-x" class="size-6" />
              </div>
              <div class="flex flex-col">
                <span class="font-medium">
                  {{ page.stats.x }}
                </span>
                <p>Followers</p>
              </div>
            </div>
          </UPageCard>

          <UPageCard class="flex-1" variant="subtle" to="https://go.nuxt.com/discord">
            <div class="flex items-center gap-2">
              <div class="rounded-lg bg-(--ui-bg) p-2 flex items-center justify-center border border-(--ui-border)">
                <UIcon name="i-simple-icons-discord" class="text-indigo-400 size-6" />
              </div>
              <div class="flex flex-col">
                <span class="font-medium">
                  {{ page.stats.discord }}
                </span>
                <p>Members</p>
              </div>
            </div>
          </UPageCard>
        </div>
      </div>
    </UPageSection>

    <UPageSection :description="page.modules.description" :links="page.modules.links" :ui="{
      root: 'bg-gradient-to-b border-t border-(--ui-border) from-(--ui-bg-muted) dark:from-(--ui-bg-muted)/40 to-(--ui-bg)',
      title: 'text-left',
      description: 'text-left',
      links: 'justify-start'
    }">
      <template #title>
        <LazyMDC :value="page.modules.title" unwrap="p" cache-key="index-modules-title" hydrate-never />
      </template>
      <UCarousel v-slot="{ item }" dots wheel-gestures arrows :items="officialModules" class="min-w-0" :ui="{
        container: 'ms-0',
        item: 'min-w-0 shrink-0 sm:basis-1/3 p-2',
        arrows: 'hidden 2xl:block'
      }">
        <ModuleItem :module="item" :show-badge="false" class="min-h-[180px]" />
      </UCarousel>
    </UPageSection>

    <UPageSection :title="page.deploy.title" :description="page.deploy.description" :links="page.deploy.links" orientation="horizontal" :ui="{
      root: 'bg-gradient-to-b border-t border-(--ui-border) from-(--ui-bg-muted) dark:from-(--ui-bg-muted)/40 to-(--ui-bg)'
    }">
      <NuxtImg src="/assets/landing/deploy.svg" width="512" height="439" :alt="page.deploy.title" class="mx-auto max-w-lg sm:w-full w-full" loading="lazy" />
    </UPageSection>

    <UPageSection :title="page.support.title" :description="page.support.description" :links="page.support.links" orientation="horizontal" class="relative" :ui="{
      root: 'bg-gradient-to-b border-t border-(--ui-border) from-(--ui-bg-muted) dark:from-(--ui-bg-muted)/40 to-20% to-(--ui-bg)',
      title: 'text-left',
      description: 'text-left',
      links: 'justify-start'
    }">
      <template #title>
        <LazyMDC :value="page.support.title" unwrap="p" cache-key="index-support-title" hydrate-never />
      </template>
      <template #description>
        <LazyMDC :value="page.support.description" unwrap="p" cache-key="index-support-description" hydrate-never />

        <UPageLogos :ui="{ logos: 'mt-6' }" marquee>
          <NuxtImg v-for="company in page.support.companies" :key="company.alt" v-bind="company" loading="lazy" class="h-8 max-w-[70px] object-contain filter invert dark:invert-0 opacity-50" :alt="`${company.alt} logo`" />
        </UPageLogos>
      </template>

      <UPageCard variant="subtle" :ui="{ container: 'gap-y-8 sm:p-8' }">
        <UPageFeature v-for="(feature, index) in page.support.features" :key="index" v-bind="feature" :ui="{
          root: 'lg:items-center lg:gap-3',
          leadingIcon: 'text-(--ui-text-highlighted)',
          leading: 'bg-(--ui-bg) p-1 lg:p-2.5 rounded-(--ui-radius) border border-(--ui-border)',
          description: 'mt-0'
        }" />
      </UPageCard>
    </UPageSection>
    <UPageSection :title="page.contributors.title" :description="page.contributors.description" :links="page.contributors.links" orientation="horizontal" reverse :ui="{
      root: 'bg-gradient-to-b border-t border-(--ui-border) from-(--ui-bg-muted) dark:from-(--ui-bg-muted)/40 to-(--ui-bg)'
    }">
      <HomeSectionContributors />
    </UPageSection>
    <UPageSection :title="page.sponsors.title" :description="page.sponsors.description" :links="page.sponsors.links" class="relative" :ui="{
      root: 'bg-gradient-to-b border-t border-(--ui-border) from-(--ui-bg-muted) dark:from-(--ui-bg-muted)/40 to-(--ui-bg)',
      container: 'py-12 sm:py-16 lg:py-20'
    }">
      <div class="flex flex-col items-center">
        <template v-for="({ tier, sponsors }) of sponsorGroups" :key="tier">
          <div class="w-full mb-24">
            <UBadge color="neutral" variant="subtle" class="capitalize mb-2">
              {{ tier }} sponsors
            </UBadge>

            <div class="w-full border border-(--ui-border) rounded-lg">
              <table class="w-full">
                <tbody>
                  <template v-for="(_, rowIndex) in Math.ceil(sponsors.length / 3)" :key="rowIndex">
                    <tr>
                      <template v-for="colIndex in 3" :key="colIndex">
                        <td v-if="(rowIndex * 3) + colIndex - 1 < sponsors.length" class="border-b border-r border-(--ui-border) p-0 w-1/3 h-[120px]" :class="{
                          'border-r-0': colIndex === 3,
                          'border-b-0': rowIndex === Math.ceil(sponsors.length / 3) - 1
                        }">
                          <NuxtLink :to="sponsors[(rowIndex * 3) + colIndex - 1].sponsorUrl" target="_blank" class="flex items-center gap-2 justify-center h-full hover:bg-(--ui-bg-muted)/50 transition-colors">
                            <NuxtImg :src="sponsors[(rowIndex * 3) + colIndex - 1].sponsorLogo" :alt="`${sponsors[(rowIndex * 3) + colIndex - 1].sponsorName} logo`" loading="lazy" class="h-10 max-w-[140px] object-contain rounded-[calc(var(--ui-radius)*2)]" height="40" width="40" />
                            <span class="text-base hidden sm:block font-semibold">{{ sponsors[(rowIndex * 3) + colIndex - 1].sponsorName }}</span>
                          </NuxtLink>
                        </td>
                        <td v-else class="border-b border-r border-(--ui-border) p-0 w-1/3 h-[120px]" :class="{
                          'border-r-0': colIndex === 3,
                          'border-b-0': rowIndex === Math.ceil(sponsors.length / 3) - 1
                        }">
                          <div class="h-full" />
                        </td>
                      </template>
                    </tr>
                  </template>
                </tbody>
              </table>
            </div>
          </div>
        </template>
      </div>
    </UPageSection>
 -->




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