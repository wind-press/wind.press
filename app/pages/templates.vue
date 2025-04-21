<script lang="ts" setup>
definePageMeta({
  heroBackground: 'opacity-80 -z-10'
})
const [{ data: page }, { data: templates }] = await Promise.all([
  useAsyncData('templates-landing', () => queryCollection('landing').path('/templates').first()),
  useAsyncData('templates', () => queryCollection('templates').all())
])

if (!page.value) {
  throw createError({ statusCode: 404, statusMessage: 'Page not found', fatal: true })
}

const title = page.value.title
const description = page.value.description

const featuredTemplates = computed(() => templates.value?.filter(template => template.featured) || [])
const baseTemplates = computed(() => templates.value?.filter(template => !template.featured) || [])

useSeoMeta({
  titleTemplate: '%s',
  title,
  description,
  ogDescription: description,
  ogTitle: title
})
defineOgImageComponent('Docs', {
  title,
  description
})

// sort templates. If template is featured, it should be first, else it should be sorted by name
const sortedTemplates = computed(() => {
  if (!templates.value) return []
  const sorted = [...templates.value]
  sorted.sort((a, b) => {
    if (a.featured && !b.featured) return -1
    if (!a.featured && b.featured) return 1
    return a.name.localeCompare(b.name)
  })
  return sorted
})

</script>

<template>
  <UContainer v-if="page">
    <UPageHero
      :title="title"
      :description="description"
      :links="page.links"
    />
    <UPage>
      <UPageBody>
        <div class="mb-24">
          <UPageGrid class="lg:grid-cols-3 xl:grid-cols-4">
            <TemplateCard
              v-for="(template, index) in sortedTemplates"
              :key="template.slug"
              :template="template"
              :index="index"
            />
          </UPageGrid>
        </div>
      </UPageBody>
    </UPage>
  </UContainer>
</template>
