<script setup lang="ts">
// const { $ads } = useNuxtApp()
const route = useRoute()
const uiPro = useState('uiProAd', () => Math.round(Math.random()))
if (!useNuxtApp().isHydrating) {
  uiPro.value = Math.round(Math.random())
}
</script>

<template>
  <div class="space-y-3">
    <!-- <AdsUIPro v-if="uiPro" /> -->
    <!-- <AdsFallback v-else-if="$ads.adBlocked.value" /> -->
    <!-- <LazyAdsCarbon v-else :key="route.path" /> -->
    <AdsFallback />
  </div>
</template>
