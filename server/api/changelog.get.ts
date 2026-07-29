import { parseMarkdown } from '@nuxtjs/mdc/runtime'

function nodeToHTML(nodes: any[]): string {
  return nodes.map(node => {
    if (node.type === 'text') {
      return node.value
    } else if (node.type === 'element') {
      const tag = node.tag
      const props = node.props || {}
      const children = node.children || []

      const attrs = Object.entries(props)
        .filter(([key]) => key !== 'id')
        .map(([key, value]) => {
          if (Array.isArray(value)) {
            return `${key}="${value.join(' ')}"`
          }
          return `${key}="${value}"`
        })
        .join(' ')

      const attrsString = attrs ? ` ${attrs}` : ''
      const childrenHTML = nodeToHTML(children)

      if (['br', 'hr', 'img'].includes(tag)) {
        return `<${tag}${attrsString} />`
      }

      return `<${tag}${attrsString}>${childrenHTML}</${tag}>`
    }
    return ''
  }).join('')
}

export default cachedEventHandler(async (event) => {
  try {
    const response = await fetch('https://raw.githubusercontent.com/orgrosua/yabe-webfont/master/CHANGELOG.md')

    if (!response.ok) {
      throw new Error(`GitHub API error: ${response.status}`)
    }

    const rawMarkdown = await response.text()
    const parsedMarkdown = await parseMarkdown(rawMarkdown)

    const linkReferences: Record<string, string> = {}
    const linkRefRegex = /^\[([^\]]+)\]:\s*(.+)$/gm
    let match
    while ((match = linkRefRegex.exec(rawMarkdown)) !== null) {
      if (match[1] && match[2]) {
        linkReferences[match[1]] = match[2]
      }
    }

    let versions: Array<{
      title: string
      date: string
      link: string
      changes: Record<string, string[]>
    }> = []

    const children = (parsedMarkdown.body?.children ?? []) as any[]
    let currentVersion: any = null
    let currentChangeType = ''

    for (let i = 0; i < children.length; i++) {
      const node = children[i] as any

      if (node.type === 'element' && node.tag === 'h2') {
        if (currentVersion) {
          versions.push(currentVersion)
        }

        const h2Children = node.children || []
        let title = ''
        let fullText = ''
        let linkHref = ''

        for (const child of h2Children) {
          if (child.type === 'text') {
            fullText += child.value
          } else if (child.type === 'element' && child.tag === 'span' && child.children) {
            for (const spanChild of child.children) {
              if (spanChild.type === 'text') {
                title = spanChild.value
              }
            }
          } else if (child.type === 'element' && child.tag === 'a' && child.props?.href) {
            linkHref = child.props.href
            if (!title && child.children) {
              for (const anchorChild of child.children) {
                if (anchorChild.type === 'text') {
                  title = anchorChild.value
                }
              }
            }
          }
        }

        const parsed = fullText.match(/^(.+?)\s*-\s*(.+)$/)

        if (!parsed) {
          const versionTitle = title || fullText.trim()
          currentVersion = {
            title: versionTitle,
            date: '',
            link: linkHref || linkReferences[versionTitle] || 'https://github.com/orgrosua/yabe-webfont/compare/HEAD',
            changes: {}
          }
        } else {
          const versionTitle = title || (parsed![1] ?? '').trim()
          currentVersion = {
            title: versionTitle,
            date: (parsed![2] ?? '').trim(),
            link: linkHref || linkReferences[versionTitle] || `https://github.com/orgrosua/yabe-webfont/releases/tag/${versionTitle}`,
            changes: {}
          }
        }
      } else if (node.type === 'element' && node.tag === 'h3' && currentVersion) {
        currentChangeType = ''
        for (const child of node.children || []) {
          if (child.type === 'text') {
            currentChangeType = child.value.trim()
            break
          }
        }
        if (currentChangeType && !currentVersion.changes[currentChangeType]) {
          currentVersion.changes[currentChangeType] = []
        }
      } else if (node.type === 'element' && node.tag === 'ul' && currentVersion && currentChangeType) {
        const listItems = node.children || []
        for (const li of listItems) {
          if (li.type === 'element' && li.tag === 'li') {
            const htmlString = nodeToHTML(li.children || [])
            if (htmlString.trim()) {
              currentVersion.changes[currentChangeType].push(htmlString.trim())
            }
          }
        }
      }
    }

    if (currentVersion) {
      versions.push(currentVersion)
    }

    setHeader(event, 'X-Cache', 'MISS')
    setHeader(event, 'Cache-Control', 'public, max-age=3600')

    return {
      versions,
      lastUpdated: new Date().toISOString(),
      source: 'github'
    }
  } catch (error) {
    throw createError({
      statusCode: 503,
      statusMessage: 'Changelog unavailable'
    })
  }
}, {
  maxAge: 60 * 60 * 24,
  name: 'changelog',
  getKey: (event) => 'changelog',
  swr: true
})
