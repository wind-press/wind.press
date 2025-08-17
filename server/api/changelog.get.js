import { parseMarkdown } from '@nuxtjs/mdc/runtime'

function parseChangelogVersions(content) {
  const versions = []
  const lines = content.split('\n')
  
  // First, extract all version links from the bottom of the file
  const versionLinks = {}
  for (const line of lines) {
    const linkMatch = line.match(/^\[([^\]]+)\]:\s*(.+)$/)
    if (linkMatch) {
      const versionKey = linkMatch[1].toLowerCase()
      versionLinks[versionKey] = linkMatch[2].trim()
    }
  }
  
  let currentVersion = null
  let currentSection = null
  let currentItems = []
  let inChangelogContent = false
  
  // Helper function to save current section
  const saveCurrentSection = () => {
    if (currentSection && currentItems.length > 0 && currentVersion) {
      if (!currentVersion.changes[currentSection]) {
        currentVersion.changes[currentSection] = []
      }
      currentVersion.changes[currentSection].push(...currentItems)
      currentItems = []
    }
  }
  
  // Helper function to save current version
  const saveCurrentVersion = () => {
    if (currentVersion) {
      saveCurrentSection() // Save any pending section
      versions.push(currentVersion)
    }
  }
  
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i].trim()
    
    // Skip everything until we reach the first version
    if (!inChangelogContent && !line.match(/^## \[/)) {
      continue
    }
    
    // Match version headers like "## [3.3.56] - 2025-08-13" or "## [Unreleased]"
    const versionMatch = line.match(/^## \[([^\]]+)\](?:\s*-\s*(\d{4}-\d{2}-\d{2}))?/)
    if (versionMatch) {
      inChangelogContent = true
      saveCurrentVersion() // Save previous version if exists
      
      const versionNumber = versionMatch[1]
      const versionDate = versionMatch[2]
      const isUnreleased = versionNumber === 'Unreleased'
      
      // Start new version
      const linkKey = isUnreleased ? 'unreleased' : versionNumber.toLowerCase()
      const versionLink = versionLinks[linkKey]

      currentVersion = {
        title: isUnreleased ? 'Unreleased' : versionNumber,
        badge: isUnreleased ? 'Unreleased' : versionNumber,
        date: versionDate || new Date().toISOString().split('T')[0],
        changes: {},
        description: '',
        link: versionLink || null
      }
      currentSection = null
      currentItems = []
      continue
    }
    
    // Skip processing if we haven't started yet or no current version
    if (!inChangelogContent || !currentVersion) {
      continue
    }
    
    // Match section headers like "### Added", "### Fixed", etc.
    const sectionMatch = line.match(/^### (Added|Changed|Fixed|Removed|Deprecated|Security|Improved)/)
    if (sectionMatch) {
      saveCurrentSection() // Save previous section if exists
      
      currentSection = sectionMatch[1]
      currentItems = []
      continue
    }
    
    // Match list items like "- Some change"
    const itemMatch = line.match(/^[-*] (.+)/)
    if (itemMatch && currentSection) {
      const itemText = itemMatch[1].trim()
      if (itemText) {
        currentItems.push(itemText)
      }
      continue
    }
    
    // If we hit another ## (next version), we'll catch it in the next iteration
  }
  
  // Save the last version
  saveCurrentVersion()
  
  // Clean up data without descriptions to avoid duplication
  return versions.map(version => ({
    ...version,
    description: undefined // Remove description to avoid duplication with body content
  })).filter(version => Object.keys(version.changes).length > 0 || version.badge === 'Unreleased')
}

export default cachedEventHandler(async (event) => {
  console.log('Fetching fresh changelog from GitHub')
  
  try {
    // Fetch from GitHub
    const response = await fetch('https://raw.githubusercontent.com/wind-press/windpress/main/CHANGELOG.md')
    
    if (!response.ok) {
      throw new Error(`GitHub API responded with status: ${response.status}`)
    }
    
    const rawContent = await response.text()
    
    // Parse markdown using MDC
    const parsedContent = await parseMarkdown(rawContent)
    
    // Parse versions for structured display
    const versions = parseChangelogVersions(rawContent)
    
    console.log(`Parsed ${versions.length} versions`)
    console.log('First 3 versions:', versions.slice(0, 3).map(v => ({ title: v.title, changeCount: Object.keys(v.changes).length })))
    
    const data = {
      rawContent,
      parsedContent,
      versions,
      lastUpdated: new Date().toISOString(),
      source: 'github'
    }
    
    setHeader(event, 'X-Cache', 'MISS')
    setHeader(event, 'Cache-Control', 'public, max-age=3600') // Browser cache for 1 hour
    
    return data
    
  } catch (error) {
    console.error('Error fetching changelog:', error.message)
    
    throw createError({
      statusCode: 503,
      statusMessage: 'Changelog temporarily unavailable. Please try again later.'
    })
  }
}, {
  maxAge: 60 * 60 * 24, // Cache for 1 day
  name: 'changelog',
  getKey: (event) => {
    const query = getQuery(event)
    return query.bustCache ? `windpress-changelog-${Date.now()}` : 'windpress-changelog'
  }
})