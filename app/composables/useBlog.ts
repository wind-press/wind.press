import type { BlogArticle } from '~/types'

export const useBlog = () => {
  const articles = useState<BlogArticle[]>('articles', () => [])

  async function fetchList() {
    if (articles.value.length) {
      return
    }

    try {
      const { data: posts } = await useAsyncData('posts', async () => {
        return queryCollection('blog')
          .where('extension', '=', 'md')
          .order('date', 'DESC')
          .all()
      })

      articles.value = posts.value?.filter(article => article.path !== '/blog') || []
    } catch (e) {
      articles.value = []
      return e
    }
  }

  return {
    articles,
    fetchList
  }
}
