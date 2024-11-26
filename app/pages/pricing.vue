<script setup lang="ts">
import { twJoin } from 'tailwind-merge'

const { data: page } = await useAsyncData('pricing', () => queryContent('/pricing').findOne())
if (!page.value) {
  throw createError({ statusCode: 404, statusMessage: 'Page not found', fatal: true })
}

const wp_v12 = inject<Ref<{ version: string; active_installs: number; donate_link: string; download_link: string }>>('wp_v12', ref({ version: '', active_installs: 0, donate_link: '', download_link: 'https://downloads.wordpress.org/plugin/windpress.zip' }));
const edd = inject<Ref<{ happyCustomers: number; activeSites: number; totalDownloads: number }>>('edd', ref({ happyCustomers: 0, activeSites: 0, totalDownloads: 0 }));


const appConfig = useAppConfig()

const config = computed(() => {
  const base = twJoin(
    'flex-1 gap-6 lg:gap-x-8 xl:gap-x-10 flex flex-col',
    'vertical' === 'horizontal' ? 'lg:grid lg:grid-cols-10' : ''
  )

  const left: string = 'vertical' === 'horizontal' ? 'lg:col-span-7' : ''
  const right: string = 'vertical' === 'horizontal' ? 'flex flex-col lg:items-center justify-center gap-y-6 lg:col-span-3 border-t lg:border-l lg:border-t-0 border-gray-200 dark:border-gray-800 pt-6 lg:pt-0 lg:pl-8 xl:pl-10' : ''

  return {
    wrapper: 'relative flex flex-col self-stretch w-full',
    highlight: 'ring-2 ring-primary dark:ring-primary',
    scale: 'lg:scale-[1.1] lg:z-10',
    rounded: 'rounded-xl',
    header: {
      padding: 'p-6 lg:px-8 xl:px-10'
    },
    body: {
      base,
      padding: 'p-6 lg:p-8 xl:p-10'
    },
    footer: {
      padding: 'p-6 lg:px-8 xl:px-10'
    },
    inner: 'flex items-center gap-3',
    title: 'text-2xl text-gray-900 dark:text-white sm:text-3xl font-semibold truncate',
    description: 'text-sm sm:text-base text-gray-500 dark:text-gray-400 mt-2',
    amount: {
      base: 'flex flex-row items-baseline gap-x-1',
      discount: 'text-gray-500 dark:text-gray-400 line-through text-xl sm:text-2xl font-medium',
      price: 'text-gray-900 dark:text-white text-2xl sm:text-4xl font-semibold',
      cycle: 'text-gray-500 dark:text-gray-400 text-sm/6 font-medium truncate'
    },
    features: {
      vertical: 'space-y-3 text-sm',
      horizontal: 'grid lg:grid-cols-2 text-sm gap-3',
      item: {
        base: 'flex items-center gap-x-3 min-w-0',
        label: 'text-gray-600 dark:text-gray-400 truncate',
        icon: {
          base: 'w-5 h-5 flex-shrink-0 text-primary',
          name: appConfig.ui.icons.check
        }
      }
    },
    divider: 'my-6 lg:my-8',
    left,
    right
  }
})

const { ui, attrs } = useUI('pricing.card', ref({}), config, ref(''), true)
const isOnetime = ref(false)

page.value.plans.forEach((plan) => {
  if (typeof plan.button?.to === 'object') {
    plan.button._to = plan.button.to;
    plan.button.to = plan.button._to[isOnetime.value ? 'onetime' : 'year'];
  }
})

watch(isOnetime, (value) => {
  page.value.plans.forEach((plan) => {
    if (plan.button?._to) {
      plan.button.to = plan.button._to[isOnetime.value ? 'onetime' : 'year'];
    }
  })
}, { immediate: true })

</script>

<template>
  <div v-if="page" id="pricing">
    <UPageHero v-bind="page.hero">
      <template #links>
        <UPricingToggle v-model="isOnetime" left="Yearly" right="One-time" class="w-48" />
      </template>
    </UPageHero>

    <UContainer>
      <UPricingGrid>
        <UPricingCard v-for="(plan, index) in page.plans" :key="index" v-bind="plan" :price="plan.price.onetime || plan.price.year ? isOnetime ? plan.price.onetime : plan.price.year : 'Free'" :cycle="plan.price.year && !isOnetime ? '/year' : undefined">
          <template #features>
            <ul v-if="plan.features?.length" :class="ui.features.vertical">
              <li v-for="(offer, index) of plan.features" :key="index" :class="ui.features.item.base">
                <template v-if="typeof offer === 'object'">
                  <UIcon :name="offer.icon_name" :class="offer.icon_class ?? ui.features.item.icon.base" />
                  <span :class="offer.label_class ?? ui.features.item.label">
                    {{ typeof offer.label_text === 'object' ? offer.label_text[isOnetime ? 'onetime' : 'year'] : offer.label_text }}
                  </span>
                </template>
                <template v-else>
                  <UIcon :name="ui.features.item.icon.name" :class="ui.features.item.icon.base" />
                  <span :class="ui.features.item.label">{{ offer }}</span>
                </template>
              </li>
            </ul>
          </template>
        </UPricingCard>

      </UPricingGrid>
    </UContainer>

    <ULandingSection class="py-8 sm:py-16 lg:py-24">
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
            <strong>{{ edd.activeSites }}+ Active</strong> Sites
          </div>
        </div>
      </div>
    </ULandingSection>

    <!-- 
    <ULandingSection>
      <ULandingLogos>
        <UIcon
          v-for="icon in page.logos.icons"
          :key="icon"
          :name="icon"
          class="w-12 h-12 flex-shrink-0 text-gray-500 dark:text-gray-400"
        />
      </ULandingLogos>
    </ULandingSection> 
    -->

    <ULandingSection :title="page.faq.title" :description="page.faq.description">
      <ULandingFAQ :items="page.faq.items" multiple class="max-w-4xl mx-auto" />
    </ULandingSection>
  </div>
</template>

<style scoped>
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
</style>