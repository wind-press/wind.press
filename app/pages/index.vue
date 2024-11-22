<script setup lang="ts">
const { data: page } = await useAsyncData('index', () => queryContent('/').findOne())
if (!page.value) {
  throw createError({ statusCode: 404, statusMessage: 'Page not found', fatal: true })
}

useSeoMeta({
  titleTemplate: '',
  title: page.value.title,
  ogTitle: page.value.title,
  description: page.value.description,
  ogDescription: page.value.description
})
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

          <!-- <div class="feature__container grid-cols:2 grid-cols:3@sm grid-cols:6@lg grid-auto-cols:auto gap-x:15 gap-y:30 gap:64@sm"> -->
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
              <div class="feature__title"> Tailwind CSS 4.x ready </div>
            </div>
            <div class="feature__item">
              <div class="feature__icon fg:gray-40 round">
                <div class="flex 
                  -ml-6 [&>:not([hidden])~:not([hidden])]:mr-0 [&>:not([hidden])~:not([hidden])]:-ml-4 [&>*]:inline-block [&>*]:w-10 [&>*]:h-10 
                ">
                  <!-- <i class="fa-solid fa-cloud-slash"></i> -->
                  <img src="https://upload.wikimedia.org/wikipedia/commons/e/e1/Google_Chrome_icon_%28February_2022%29.svg" alt="Google Chromo logo">
                  <img src="https://upload.wikimedia.org/wikipedia/commons/5/52/Safari_browser_logo.svg" alt="Apple Safari logo">
                  <img src="https://upload.wikimedia.org/wikipedia/commons/9/98/Microsoft_Edge_logo_%282019%29.svg" alt="Microsoft Edge logo">
                  <img src="https://upload.wikimedia.org/wikipedia/commons/a/a0/Firefox_logo%2C_2019.svg" alt="Mozilla Firefox logo">
                </div>
              </div>
              <div class="feature__title"> Compile in the browser; No server is needed </div>
            </div>
            <div class="feature__item">
              <div class="feature__icon fg:crimson-80 round"><i class="fa-light fa-face-smile-hearts"></i></div>
              <div class="feature__title"> Easy to use and intuitive </div>
            </div>
            <div class="feature__item">
              <div class="feature__icon fg:sky-70"><i class="fa-light fa-feather"></i></div>
              <div class="feature__title"> Lightweight and blazingly fast </div>
            </div>
            <div class="feature__item">
              <div class="feature__icon fg:#4e6fc3"><i class="fa-brands fa-wordpress"></i></div>
              <div class="feature__title"> Visual builders integration </div>
            </div>
            <div class="feature__item">
              <div class="feature__icon fg:lime-70 round"><i class="fa-solid fa-play"></i></div>
              <div class="feature__title"> Zero-configuration yet customizable </div>
            </div>
          </div>
        </div>


        <!-- <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-6 auto-cols-auto gap-x-4 gap-y-8 sm:gap-16">
          <div class="feature__item">
            <div class="feature__icon">
              <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 54 33" class="w-12 h-12">
                <g clip-path="url(#clip0)">
                  <path fill="#38bdf8" fill-rule="evenodd" d="M27 0c-7.2 0-11.7 3.6-13.5 10.8 2.7-3.6 5.85-4.95 9.45-4.05 2.054.513 3.522 2.004 5.147 3.653C30.744 13.09 33.808 16.2 40.5 16.2c7.2 0 11.7-3.6 13.5-10.8-2.7 3.6-5.85 4.95-9.45 4.05-2.054-.513-3.522-2.004-5.147-3.653C36.756 3.11 33.692 0 27 0zM13.5 16.2C6.3 16.2 1.8 19.8 0 27c2.7-3.6 5.85-4.95 9.45-4.05 2.054.514 3.522 2.004 5.147 3.653C17.244 29.29 20.308 32.4 27 32.4c7.2 0 11.7-3.6 13.5-10.8-2.7 3.6-5.85 4.95-9.45 4.05-2.054-.513-3.522-2.004-5.147-3.653C23.256 19.31 20.192 16.2 13.5 16.2z" clip-rule="evenodd"></path>
                </g>
                <defs>
                  <clipPath id="clip0">
                    <path fill="#fff" d="M0 0h54v32.4H0z"></path>
                  </clipPath>
                </defs>
              </svg>
            </div>
            <div class="feature__title text-center mt-2 text-gray-800">Tailwind CSS 4.x ready</div>
          </div>

          <div class="feature__item">
            <div class="feature__icon bg-gray-400 rounded-full flex justify-center items-center">
              <div class="flex -ml-9 [&>:not([hidden])~:not([hidden])]:inline-block [&>:not([hidden])~:not([hidden])]:w-10 [&>:not([hidden])~:not([hidden])]:h-10">
                <i class="fa-solid fa-cloud-slash"></i>
                <img src="https://upload.wikimedia.org/wikipedia/commons/e/e1/Google_Chrome_icon_%28February_2022%29.svg" alt="Google Chrome logo">
                <img src="https://upload.wikimedia.org/wikipedia/commons/5/52/Safari_browser_logo.svg" alt="Apple Safari logo">
                <img src="https://upload.wikimedia.org/wikipedia/commons/9/98/Microsoft_Edge_logo_%282019%29.svg" alt="Microsoft Edge logo">
                <img src="https://upload.wikimedia.org/wikipedia/commons/a/a0/Firefox_logo%2C_2019.svg" alt="Mozilla Firefox logo">
              </div>
            </div>
            <div class="feature__title text-center mt-2 text-gray-800">Compile in the browser; No server is needed</div>
          </div>

          <div class="feature__item">
            <div class="feature__icon bg-red-500 rounded-full flex justify-center items-center">
              <i class="fa-light fa-face-smile-hearts"></i>
            </div>
            <div class="feature__title text-center mt-2 text-gray-800">Easy to use and intuitive</div>
          </div>

          <div class="feature__item">
            <div class="feature__icon bg-sky-500 rounded-full flex justify-center items-center">
              <i class="fa-light fa-feather"></i>
            </div>
            <div class="feature__title text-center mt-2 text-gray-800">Lightweight and blazingly fast</div>
          </div>

          <div class="feature__item">
            <div class="feature__icon bg-blue-600 rounded-full flex justify-center items-center">
              <i class="fa-brands fa-wordpress"></i>
            </div>
            <div class="feature__title text-center mt-2 text-gray-800">Visual builders integration</div>
          </div>

          <div class="feature__item">
            <div class="feature__icon bg-lime-500 rounded-full flex justify-center items-center">
              <i class="fa-solid fa-play"></i>
            </div>
            <div class="feature__title text-center mt-2 text-gray-800">Zero-configuration yet customizable</div>
          </div>
        </div> -->


      </template>

    </ULandingHero>

    <ULandingSection class="!pt-0">
      <ImagePlaceholder />
    </ULandingSection>

    <ULandingSection v-for="(section, index) in page.sections" :key="index" :title="section.title" :description="section.description" :align="section.align" :features="section.features">
      <ImagePlaceholder />
    </ULandingSection>

    <ULandingSection :title="page.features.title" :description="page.features.description">
      <UPageGrid>
        <ULandingCard v-for="(item, index) in page.features.items" :key="index" v-bind="item" />
      </UPageGrid>
    </ULandingSection>

    <ULandingSection :headline="page.testimonials.headline" :title="page.testimonials.title" :description="page.testimonials.description">
      <UPageColumns class="xl:columns-4">
        <div v-for="(testimonial, index) in page.testimonials.items" :key="index" class="break-inside-avoid">
          <ULandingTestimonial v-bind="testimonial" class="bg-gray-100/50 dark:bg-gray-800/50" />
        </div>
      </UPageColumns>
    </ULandingSection>

    <ULandingSection>
      <ULandingCTA v-bind="page.cta" class="bg-gray-100/50 dark:bg-gray-800/50" />
    </ULandingSection>
  </div>
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
</style>
