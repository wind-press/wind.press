<script setup lang="ts">
import type { ParsedContent } from '@nuxt/content'

const { data: navigation } = await useAsyncData('navigation', () => fetchContentNavigation(), { default: () => [] })
const { data: files } = useLazyFetch<ParsedContent[]>('/api/search.json', { default: () => [], server: false })

const edd = ref({
    happyCustomers: 0,
    activeSites: 0,
    totalDownloads: 0,
});

const wp_v10 = ref({
    downloaded: 0,
});
const wp_v12 = ref({
    version: '3.0.x',
    active_installs: 0,
    donate_link: 'https://ko-fi.com/Q5Q75XSF7',
    download_link: 'https://downloads.wordpress.org/plugin/windpress.zip',
});

onBeforeMount(() => {
    fetch('https://rosua.org/wp-json/rosua-edd/v1/stats/total_users')
        .then(res => res.json())
        .then(res => {
            edd.value.happyCustomers += res[0].count_users;
        })
        .catch(err => {
            console.log(err);
        });

    fetch('https://rosua.org/wp-json/rosua-edd/v1/stats/total_sites')
        .then(res => res.json())
        .then(res => {
            // search array of object with download_id = 2250 (WindPress)
            const downloadWindPress = res.find(item => item.download_id === 2250);
            edd.value.activeSites += downloadWindPress.count_sites;
            edd.value.totalDownloads += downloadWindPress.count_sites;
            
            // search array of object with download_id = 3591 (Yabe Siul)
            const downloadYabeSiul = res.find(item => item.download_id === 3591);
            edd.value.activeSites += downloadYabeSiul.count_sites;
            edd.value.totalDownloads += downloadYabeSiul.count_sites;
        })
        .catch(err => {
            console.log(err);
        });

    fetch('https://api.wordpress.org/plugins/info/1.2/?action=plugin_information&slug=windpress')
        .then(response => response.json())
        .then(data => {
            wp_v12.value = data;
            edd.value.activeSites += wp_v12.value.active_installs;
        });
    fetch('https://api.wordpress.org/plugins/info/1.0/windpress.json')
        .then(response => response.json())
        .then(data => {
            wp_v10.value = data;
            edd.value.totalDownloads += wp_v10.value.downloaded;
        });
});

provide('navigation', navigation);
provide('wp_v10', wp_v10);
provide('wp_v12', wp_v12);
provide('edd', edd);
</script>

<template>
  <div>
    <AppHeader />

    <UMain>
      <slot />
    </UMain>

    <AppFooter />

    <ClientOnly>
      <LazyUContentSearch :files="files" :navigation="navigation" />
    </ClientOnly>
  </div>
</template>
