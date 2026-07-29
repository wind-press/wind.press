import { defineContentConfig, defineCollection, z } from '@nuxt/content'
import { asSitemapCollection } from '@nuxtjs/sitemap/content'

const Image = z.object({
  src: z.string(),
  alt: z.string(),
  width: z.number().optional(),
  height: z.number().optional()
})

const Link = z.object({
  label: z.string(),
  to: z.string(),
  icon: z.string().optional()
})

const Button = z.object({
  label: z.string(),
  icon: z.string().optional(),
  trailingIcon: z.string().optional(),
  to: z.string().optional(),
  color: z.enum(['primary', 'neutral', 'success', 'warning', 'error', 'info']).optional(),
  size: z.enum(['xs', 'sm', 'md', 'lg', 'xl']).optional(),
  variant: z.enum(['solid', 'outline', 'subtle', 'soft', 'ghost', 'link']).optional(),
  id: z.string().optional(),
  target: z.enum(['_blank', '_self']).optional()
})

const BaseSection = z.object({
  title: z.string(),
  description: z.string()
})

const Author = z.object({
  name: z.string(),
  description: z.string().optional(),
  username: z.string().optional(),
  twitter: z.string().optional(),
  to: z.string().optional(),
  avatar: Image.optional()
})

const PageFeature = z.object({
  title: z.string(),
  description: z.string(),
  icon: z.string().editor({ input: 'icon' }),
  to: z.string().optional(),
  target: z.enum(['_blank', '_self']).optional(),
  soon: z.boolean().optional()
})

const PageSection = BaseSection.extend({
  links: z.array(Button),
  features: z.array(PageFeature)
})

const PageHero = BaseSection.extend({
  head: z.object({
    title: z.string().optional(),
    description: z.string().optional()
  }).optional(),
  headline: z.object({
    label: z.string(),
    to: z.string(),
    icon: z.string().optional().editor({ input: 'icon' })
  }).optional(),
  links: z.array(Button).optional(),
  cta: Link.optional()
})

export default defineContentConfig({
  collections: {
    index: defineCollection({
      type: 'data',
      source: 'index.yml',
      schema: z.object({
        hero: z.object({
          title: z.string(),
          description: z.string(),
          cta: Link.extend({
            icon: z.string()
          }).optional(),
          links: z.array(Button).optional()
        }),
        features: PageSection,
        testimonials: BaseSection.extend({
          headline: z.object({
            label: z.string(),
            to: z.string(),
            icon: z.string().optional().editor({ input: 'icon' })
          }).optional(),
          items: z.array(
            z.object({
              quote_title: z.string().optional(),
              quote: z.string(),
              author: z.object({
                name: z.string(),
                description: z.string().optional(),
                avatar: Image.optional(),
                to: z.string().optional(),
                target: z.enum(['_blank', '_self']).optional()
              })
            })
          )
        }).optional(),
        visual_builders: z.array(z.object({
          name: z.string(),
          url: z.string().url().optional(),
          icon: z.string().optional(),
          pro: z.boolean().optional()
        })),
        as_seen_on: z.array(z.object({
          name: z.string(),
          url: z.string().url(),
          logo: z.string()
        })).optional()
      })
    }),
    docs: defineCollection(
      asSitemapCollection({
        type: 'page',
        source: 'docs/**',
        schema: z.object({
          titleTemplate: z.string().optional(),
          links: z.array(Button)
        })
      })
    ),
    blog: defineCollection(
      asSitemapCollection({
        type: 'page',
        source: 'blog/*',
        schema: z.object({
          image: z.string().editor({ input: 'media' }),
          authors: z.array(Author),
          date: z.string().date(),
          draft: z.boolean().optional(),
          category: z.enum(['Release', 'Tutorial', 'Announcement', 'Article']),
          tags: z.array(z.string()).optional()
        })
      })
    ),
    landing: defineCollection({
      type: 'page',
      source: [
        { include: 'blog.yml' }
      ],
      schema: PageHero
    })
  }
})
