export interface Stats {
  wp_downloaded: number
  wp_version: string
  wp_active_installs: number
  wp_donate_link: string
  wp_download_link: string
  edd_happyCustomers: number
  edd_activeSites: number
  edd_totalDownloads: number
}

export const useStats = () => {
  return useState<Stats | null>('stats', () => null)
}
