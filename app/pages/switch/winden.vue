<script setup lang="ts">
import { z } from 'zod'
import type { FormSubmitEvent } from '@nuxt/ui'
import { joinURL } from 'ufo'

const title = 'Bye, Winden! Hello, WindPress!'
const description = 'Switch to WindPress today and we’ll help cover the cost of your current Winden invoice. Switch to WindPress today and we’ll help cover the cost of your current Winden invoice. Experience improved performance, enhanced features, and reliable support with WindPress.'

const route = useRoute()
const { url } = useSiteConfig()

useSeoMeta({
    title,
    description,
    ogTitle: title,
    ogDescription: description,
    ogImage: joinURL(url, '/pro/og-image.png')
})

const schema = z.object({
    license: z.string(),
})

type Schema = z.output<typeof schema>

const state = reactive<Partial<Schema>>({
    license: '',
})

const activating = ref(false)
const errorMessage = ref('')
const discountCode = ref('')

async function submit(event: FormSubmitEvent<any>) {
    activating.value = true
    errorMessage.value = ''

    try {
        const { data } = await useFetch('/api/switch/winden', {
            query: {
                license: event.data.license,
            }
        })

        if ('error' in data.value && data.value.error) {
            throw new Error(data.value.error)
        }
        if (data.value && 'data' in data.value && data.value.data) {
            discountCode.value = data.value.data.discount_code
        } else {
            throw new Error('Unexpected response structure')
        }
    } catch (err) {
        errorMessage.value = err.message
    }

    activating.value = false
}

onMounted(() => {
    if (route.query.license_key && !state.license) {
        state.license = route.query.license_key as string
    }
})
</script>

<template>
    <UMain>
        <UPageHero headline="Move your project from Winden" title="Switch to WindPress" description="We're the creators of the Winden plugin and are excited to launch its next generation, WindPress. Claim your discount code and switch today!" :ui="{ container: 'relative overflow-hidden', wrapper: 'lg:px-12', description: 'text-pretty' }">
            <SkyBg />

            <div aria-hidden="true" class="hidden lg:block absolute z-[-1] border-x border-(--ui-border) inset-0 mx-4 sm:mx-6 lg:mx-8" />

            <div class="px-4 py-10 lg:border border-(--ui-border) bg-(--ui-bg)">
                <div class="max-w-xl mx-auto">
                    <UForm :schema="schema" :validate-on="['blur']" :state="state" class="space-y-4" @submit="submit">
                        <UFormField label="Winden's License Key" name="license">
                            <UInput v-model="state.license" autocomplete="off" :ui="{ root: 'flex' }" />
                        </UFormField>
                        <UButton type="submit" :loading="activating" :disabled="state.license?.length < 1">
                            Generate discount code
                        </UButton>
                        <UAlert v-if="discountCode" color="success" variant="subtle" icon="lucide:badge-check">
                            <template #description>
                                We've created a special discount code just for you to use on your WindPress purchase. Enjoy some savings on your next order! Visit the <NuxtLink to="/#pricing" target="_blank" class="font-bold text-primary hover:underline">pricing page</NuxtLink> and use the discount code at checkout.
                                <div class="flex flex-col gap-1 my-1">
                                    <div>
                                        <b>Discount Code:</b> <ProseCode> {{ discountCode }}</ProseCode>
                                    </div>
                                    <div>
                                        <b>Valid until:</b> <ProseCode> {{ new Date(Date.now() + 1000 * 60 * 60 * 24 * 1).toLocaleDateString() }}</ProseCode>
                                    </div>
                                    <div>
                                        <b>Amount:</b> <ProseCode> 60% off</ProseCode>
                                    </div>
                                </div>
                            </template>
                        </UAlert>
                        <UAlert v-else-if="errorMessage" color="error" variant="subtle" :title="errorMessage" />
                    </UForm>
                    <ProseHr />
                    <ProseNote>
                        A discount code is generated only once for each Winden license key. If you have already generated a discount code, clicking "Generate discount code" will produce the same code.
                    </ProseNote>
                </div>
            </div>
        </UPageHero>
    </UMain>
</template>