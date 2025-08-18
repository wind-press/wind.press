function escapeKey(key: string | string[]) {
  return String(key).replace(/\W/g, "");
}

export default defineEventHandler(async () => {
  try {
    const cacheKey = `nitro:handlers:changelog:${escapeKey('/api/changelog')}.json`
    
    const cache = await useStorage('cache').getItem(cacheKey)
    if (!cache) {
      throw createError({
        statusCode: 404,
        statusMessage: 'Cache not found'
      })
    }
    
    const cacheAge = Date.now() - cache.mtime
    if (cacheAge < 60 * 1000) {
      throw createError({
        statusCode: 400,
        statusMessage: 'Cache is too fresh to invalidate'
      })
    }

    await useStorage('cache').removeItem(cacheKey)

    return {
      success: true,
      message: 'Changelog cache invalidated successfully',
      timestamp: new Date().toISOString()
    }
  } catch (error: any) {
    return {
      success: false,
      message: error.message || 'Failed to invalidate changelog cache',
      timestamp: new Date().toISOString()
    }
  }
})