import type { Stats } from '~/composables/useStats'

export default defineNuxtPlugin(async () => {
  const stats = useStats()

  if (import.meta.server) {
    // stats.value = await $fetch<Stats>('https://api.nuxt.com/stats').catch(() => null)
    stats.value = await fetchStats()
  }
  onNuxtReady(async () => {
    if (!stats.value) {
      // stats.value = await $fetch<Stats>('https://api.nuxt.com/stats')
      stats.value = await fetchStats()
    }
  })
})

async function fetchStats() {
  const stats: Stats = {}

  await fetch('https://rosua.org/wp-json/rosua-edd/v1/stats/total_users')
    .then(res => res.json())
    .then(res => {
      stats.edd_happyCustomers = res[0].count_users;
    })
    .catch(() => null);


  await fetch('https://rosua.org/wp-json/rosua-edd/v1/stats/total_sites')
    .then(res => res.json())
    .then(res => {
      // search array of object with download_id = 2250 (WindPress)
      const downloadWindPress = res.find(item => item.download_id === 2250);
      stats.edd_activeSites = downloadWindPress.count_sites;
      stats.edd_totalDownloads = downloadWindPress.count_sites;

      // search array of object with download_id = 3591 (Yabe Siul)
      const downloadYabeSiul = res.find(item => item.download_id === 3591);
      stats.edd_activeSites += downloadYabeSiul.count_sites;
      stats.edd_totalDownloads += downloadYabeSiul.count_sites;
    })
    .catch(() => null);

  await fetch('https://api.wordpress.org/plugins/info/1.2/?action=plugin_information&slug=windpress')
    .then(response => response.json())
    .then(data => {
      stats.edd_activeSites += data.active_installs;
      stats.wp_version = data.version;
      stats.wp_donate_link = data.donate_link;
      stats.wp_download_link = data.download_link;
      stats.wp_active_installs = data.active_installs;
    })
    .catch(() => null);

  await fetch('https://api.wordpress.org/plugins/info/1.0/windpress.json')
    .then(response => response.json())
    .then(data => {
      stats.wp_downloaded = data.downloaded;
      stats.edd_totalDownloads += data.downloaded;
    })
    .catch(() => null);

  return stats
}