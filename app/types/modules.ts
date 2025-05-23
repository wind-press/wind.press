import type { ParsedContentFile } from '@nuxt/content'

export interface Stats {
  maintainers: number
  contributors: number
  modules: number
}

export interface ModuleUser {
  name: string
  github: string
  twitter?: string
  bluesky?: string
}

export interface Module {
  name: string
  description: string
  repo: string
  npm: string
  icon: string
  github: string
  website: string
  learn_more: string
  category: string
  type: string
  sponsor: boolean
  // tags: string[]
  compatibility: { nuxt: string, requires: { bridge: boolean } }
  stats: {
    version: string
    downloads: number
    stars: number
    publishedAt: number
    createdAt: number
  }
  maintainers: {
    name: string
    github: string
    twitter?: string
    bluesky?: string
  }[]
  contributors: {
    id: number
    username: string
    contributions: number
  }[]
  readme?: ParsedContentFile
}
