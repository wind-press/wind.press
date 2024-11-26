import { Feed } from 'feed'
import { joinURL } from 'ufo'
import { serverQueryContent } from '#content/server'

export default defineEventHandler(async (event) => {
    const baseUrl = 'https://wind.press'
    const siteUrl = joinURL(baseUrl, 'blog')
    const feed = new Feed({
        title: 'The WindPress Blog',
        description: 'News and updates about WindPress.',
        id: siteUrl,
        link: siteUrl,
        language: 'en',
        image: joinURL(baseUrl, 'favicon.svg'),
        favicon: joinURL(baseUrl, 'favicon.svg'),
        copyright: `Copyright © 2024-${new Date().getFullYear()} WindPress All Rights Reserved`,
        feedLinks: {
            rss: `${siteUrl}/rss.xml`
        }
    })

    const posts = await serverQueryContent(event, '/blog')
        .sort({ date: -1 })
        .where({ _partial: false, _draft: false, _type: 'markdown' })
        .find()

    for (const post of posts) {
        feed.addItem({
            link: joinURL(baseUrl, post._path),
            image: joinURL(baseUrl, post.image.src),
            title: post.title,
            date: new Date(post.date),
            description: post.description,
            author: post.authors,
            category: post.category
        })
    }

    appendHeader(event, 'Content-Type', 'application/xml')
    return feed.rss2()
})