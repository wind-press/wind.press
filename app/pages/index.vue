<script setup lang="ts">
import Pricing from './pricing.vue';

const { data: page } = await useAsyncData('index', () => queryContent('/').findOne())
if (!page.value) {
  throw createError({ statusCode: 404, statusMessage: 'Page not found', fatal: true })
}

const wp_v12 = inject<Ref<{ version: string; active_installs: number; donate_link: string; download_link: string }>>('wp_v12', ref({ version: '', active_installs: 0, donate_link: '', download_link: 'https://downloads.wordpress.org/plugin/windpress.zip' }));
const edd = inject<Ref<{ happyCustomers: number; activeSites: number; totalDownloads: number }>>('edd', ref({ happyCustomers: 0, activeSites: 0, totalDownloads: 0 }));

useSeoMeta({
  titleTemplate: '',
  title: page.value.title,
  ogTitle: page.value.title,
  description: page.value.description,
  ogDescription: page.value.description
});

page.value.hero.links.forEach(link => {
  if (link.to === 'https://downloads.wordpress.org/plugin/windpress.zip') {
    link.to = wp_v12.value.download_link;
  }
});
</script>

<template>
  <div v-if="page">
    <ULandingHero :title="page.hero.title" :description="page.hero.description" :links="page.hero.links">
      <div class="absolute inset-0 landing-grid z-[-1] [mask-image:radial-gradient(100%_100%_at_top_right,white,transparent)]" />

      <template #headline>
        <UBadge v-if="page.hero.headline" variant="subtle" size="lg" class="relative rounded-full font-semibold">
          <NuxtLink :to="page.hero.headline.to" target="_blank" class="focus:outline-none" tabindex="-1">
            <span class="absolute inset-0" aria-hidden="true" />
          </NuxtLink>

          {{ page.hero.headline.label }}

          <UIcon v-if="page.hero.headline.icon" :name="page.hero.headline.icon" class="ml-1 w-4 h-4 pointer-events-none" />
        </UBadge>
      </template>

      <template #description>
        {{ page.hero.description }}

        <div class="mt-12 mb-8">
          <div class="feature__container grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-6 auto-cols-auto gap-x-4 gap-y-8 sm:gap-16">
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
            <strong>{{ edd.happyCustomers + wp_v12.active_installs }}+ Happy</strong>
            Users
          </div>
        </div>

        <div class="badges__item">
          <div class="text-2xl flex items-center text-[#4e6fc3]">
            <Icon name="fa6-brands:wordpress" />
          </div>
          <div class="text-xs text-left">
            <strong>{{ edd.totalDownloads }}+ Download</strong>
            <br> & Counting
          </div>
        </div>
      </div>

    </ULandingHero>

    <ULandingSection class="!py-0">
      <div class="bg-gray-900/5 dark:bg-white/5 ring-1 ring-inset ring-gray-900/10 dark:ring-white/10 rounded-xl lg:-m-4 p-4">
        <img src="https://ps.w.org/windpress/assets/screenshot-4.png" alt="WindPress screenshot" class="w-full h-auto rounded-lg border border-dashed border-gray-900/10 dark:border-white/10" />
      </div>
    </ULandingSection>

    <ULandingSection>
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
            <img class="absolute sm:top-[-50px] sm:right-[-220px] rounded-lg scale-75 sm:pt-0 pt-5" src="/img/pages/landing/feature-bento-wizard-color.png" alt="feature: wizard colors" />
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
                Start using the future version of Tailwind CSS today. WindPress is ready for the upcoming Tailwind CSS v4.0
              </div>
            </div>
            <img class="absolute hidden sm:block sm:bottom-[-30px] sm:right-[-30px]" src="https://tailwindcss.com/_next/static/media/tailwindcss-mark.3c5441fc7a190fb1800d4a5c7f07ba4b1345a9c8.svg" alt="Tailwind CSS version 4.x ready" />
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
              <div class="p-2 w-full grid grid-cols-2 sm:grid-cols-4 auto-rows-auto place-items-center">
                <a v-for="builder in page.visual_buildes" :href="builder.url" class="flex w-full rounded-lg aspect-square items-center justify-center hover:bg-blue-200 dark:hover:bg-gray-500/30">
                  <div class="p-5">
                    <div class="flex flex-col items-center text-center">
                      <component v-if="builder.icon.endsWith('.svg')" :is="defineAsyncComponent(() => import(`~/assets/icons/${builder.icon.slice(0, -4)}.svg`))" :fontControlled="false" filled :alt="builder.name" class="w-10 h-10" />
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
            <img class="absolute sm:right-[-70px] sm:bottom-[-60px] rounded-lg sm:pt-0 pt-5" src="/img/pages/landing/feature-bento-customizable.png" alt="feature: customizable" />
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
            <img class="absolute hidden sm:block sm:top-[50%] sm:left-[60%] rounded-lg sm:pt-0 pt-5 scale-[2]" src="/img/pages/landing/feature-bento-compile.png" alt="feature: compile" />
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
            <img class="absolute sm:right-[-70px] sm:bottom-[-65px] scale-75 sm:pt-0 pt-5" src="/img/pages/landing/feature-bento-wp-rest-api.png" alt="feature: lightweight and blazingly fast" />
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
            <img class="absolute hidden sm:block sm:right-[-80px] sm:bottom-[-20px] scale-150 rounded-lg sm:pt-0 pt-5" src="/img/pages/landing/feature-bento-simple-file-system.png" alt="feature: autocomplete" />
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
            <img class="absolute sm:right-[-40px] sm:bottom-[-20px] rounded-lg sm:pt-0 pt-5 scale-75" src="/img/pages/landing/feature-bento-javascript-package.png" alt="feature: javascript package" />
          </div>
        </div>
      </div>




    </ULandingSection>

    <!-- <ULandingSection v-for="(section, index) in page.sections" :key="index" :title="section.title" :description="section.description" :align="section.align" :features="section.features">
      <ImagePlaceholder />
    </ULandingSection> -->

    <ULandingSection :title="page.features.title" :description="page.features.description">
      <UPageGrid>
        <ULandingCard v-for="(item, index) in page.features.items" :key="index" v-bind="item" />
      </UPageGrid>
    </ULandingSection>

    <ULandingSection :headline="page.testimonials.headline" :title="page.testimonials.title" :description="page.testimonials.description">
      <UPageColumns class="xl:columns-4">
        <div v-for="(testimonial, index) in page.testimonials.items" :key="index" class="break-inside-avoid">
          <ULandingTestimonial v-bind="testimonial" class="bg-gray-100/50 dark:bg-gray-800/50">
            <template #quote>
              <span class="font-bold">{{ testimonial.quote_title ? `${testimonial.quote_title} — ` : '' }}</span>{{ testimonial.quote }}
            </template>
          </ULandingTestimonial>
        </div>
      </UPageColumns>
      <div class="text-center">
        <a href="https://wordpress.org/support/plugin/windpress/reviews/?filter=5/#new-post" target="_blank" class="underline">
          Tell us what you think about WindPress
        </a>
      </div>
    </ULandingSection>
  </div>

  <Pricing />
</template>

<style scoped>
.landing-grid {
  background-size: 100px 100px;
  background-image:
    linear-gradient(to right, rgb(var(--color-gray-200)) 1px, transparent 1px),
    linear-gradient(to bottom, rgb(var(--color-gray-200)) 1px, transparent 1px);
}

.dark {
  .landing-grid {
    background-image:
      linear-gradient(to right, rgb(var(--color-gray-800)) 1px, transparent 1px),
      linear-gradient(to bottom, rgb(var(--color-gray-800)) 1px, transparent 1px);
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
