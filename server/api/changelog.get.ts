import { parseMarkdown } from '@nuxtjs/mdc/runtime'

// Helper function to convert AST nodes to HTML
function nodeToHTML(nodes: any[]): string {
  return nodes.map(node => {
    if (node.type === 'text') {
      return node.value
    } else if (node.type === 'element') {
      const tag = node.tag
      const props = node.props || {}
      const children = node.children || []
      
      // Build attributes string
      const attrs = Object.entries(props)
        .filter(([key]) => key !== 'id') // Skip auto-generated IDs
        .map(([key, value]) => {
          if (Array.isArray(value)) {
            return `${key}="${value.join(' ')}"`
          }
          return `${key}="${value}"`
        })
        .join(' ')
      
      const attrsString = attrs ? ` ${attrs}` : ''
      const childrenHTML = nodeToHTML(children)
      
      // Self-closing tags
      if (['br', 'hr', 'img'].includes(tag)) {
        return `<${tag}${attrsString} />`
      }
      
      return `<${tag}${attrsString}>${childrenHTML}</${tag}>`
    }
    return ''
  }).join('')
}

export default defineCachedEventHandler(async (event) => {
  try {
    const response = await fetch('https://raw.githubusercontent.com/wind-press/windpress/main/CHANGELOG.md')

    if (!response.ok) {
      throw new Error(`GitHub API error: ${response.status}`)
    }

    const rawMarkdown = await response.text()
    const parsedMarkdown = await parseMarkdown(rawMarkdown)

    // Extract link references from the markdown for reference-style links
    const linkReferences: Record<string, string> = {}
    const linkRefRegex = /^\[([^\]]+)\]:\s*(.+)$/gm
    let match
    while ((match = linkRefRegex.exec(rawMarkdown)) !== null) {
      linkReferences[match[1]] = match[2]
    }

    let versions: Array<{
      title: string, // from the heading 2 (before the `-`)
      date: string, // from heading 2 (after the `-`)
      link: string, // from the heading 2 (the link)
      changes: Record<
        string, // from the heading 3 (the section title)
        string[] // from the list items under the section
      >
    }> = [];

    // Parse the markdown AST
    const children = parsedMarkdown.body.children
    let currentVersion: any = null
    let currentChangeType: string = ''

    for (let i = 0; i < children.length; i++) {
      const node = children[i]

      // Handle H2 headings (version titles)
      if (node.type === 'element' && node.tag === 'h2') {
        // Save previous version if exists
        if (currentVersion) {
          versions.push(currentVersion)
        }

        // Extract version info from H2
        const h2Children = node.children || []
        let title = ''
        let fullText = ''
        let linkHref = ''
        
        // Extract text content and links from all children
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
            // Extract link from anchor element
            linkHref = child.props.href
            // Also extract text content from anchor if title is not found in span
            if (!title && child.children) {
              for (const anchorChild of child.children) {
                if (anchorChild.type === 'text') {
                  title = anchorChild.value
                }
              }
            }
          }
        }

        // Parse title and date from full text
        const match = fullText.match(/^(.+?)\s*-\s*(.+)$/)


        if (!match) {
          // Handle other special cases
          const versionTitle = title || fullText.trim()
          const linkKey = versionTitle.toLowerCase() // Reference keys are typically lowercase
          currentVersion = {
            title: versionTitle,
            date: '',
            link: linkHref || linkReferences[linkKey] || linkReferences[versionTitle] || `https://github.com/wind-press/windpress/compare/HEAD`,
            changes: {}
          }
        } else {
          const [, , date] = match
          const versionTitle = title || match[1].trim()
          currentVersion = {
            title: versionTitle,
            date: date.trim(),
            link: linkHref || linkReferences[versionTitle] || `https://github.com/wind-press/windpress/releases/tag/${versionTitle}`,
            changes: {}
          }
        }
      }

      // Handle H3 headings (change types)
      else if (node.type === 'element' && node.tag === 'h3' && currentVersion) {
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
      }

      // Handle UL lists (changes under each type)
      else if (node.type === 'element' && node.tag === 'ul' && currentVersion && currentChangeType) {
        const listItems = node.children || []
        for (const li of listItems) {
          if (li.type === 'element' && li.tag === 'li') {
            // Convert AST node to HTML string
            const htmlString = nodeToHTML(li.children || [])
            if (htmlString.trim()) {
              currentVersion.changes[currentChangeType].push(htmlString.trim())
            }
          }
        }
      }
    }

    // Don't forget to add the last version
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
  getKey: () => '/api/changelog'
})