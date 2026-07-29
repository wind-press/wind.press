import type { Stats } from '~/composables/useStats'

export default defineNuxtPlugin(async () => {
  const stats = useStats()

  if (import.meta.server || import.meta.client) {
    stats.value = await fetchStats()
  }
  onNuxtReady(async () => {
    if (!stats.value || !stats.value.wp_version) {
      stats.value = await fetchStats()
    }
  })
})

async function fetchStats(): Promise<Stats> {
  const stats: Stats = {
    wp_downloaded: 0,
    wp_version: '',
    wp_active_installs: 0,
    wp_donate_link: '',
    wp_download_link: '',
    edd_happyCustomers: 0,
    edd_activeSites: 0,
    edd_totalDownloads: 0,
  }

  await fetch('https://api.wordpress.org/plugins/info/1.2/?action=plugin_information&slug=yabe-webfont')
    .then(response => response.json())
    .then((data: any) => {
      stats.wp_version = data.version
      stats.wp_donate_link = data.donate_link
      stats.wp_download_link = data.download_link
      stats.wp_active_installs = data.active_installs
      stats.edd_activeSites = data.active_installs
    })
    .catch(() => null)

  await fetch('https://api.wordpress.org/plugins/info/1.0/yabe-webfont.json')
    .then(response => response.json())
    .then((data: any) => {
      stats.wp_downloaded = data.downloaded
      stats.edd_totalDownloads = data.downloaded
    })
    .catch(() => null)

  await fetch('https://jooo.si/wp-json/jooosi/v1/stats/total_sites')
    .then(response => response.json())
    .then((data: any) => {
      const entry = Array.isArray(data) ? data.find((d: any) => d.download_id === 18) : null
      if (entry?.count_sites) {
        stats.edd_activeSites += entry.count_sites
      }
    })
    .catch(() => null)

  return stats
}
