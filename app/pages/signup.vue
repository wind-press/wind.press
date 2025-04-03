<script setup lang="ts">
import type { FormError } from '#ui/types'
import Logo from '../components/Logo.vue';

definePageMeta({
    layout: 'auth'
})

useSeoMeta({
    title: 'Sign up'
})

const form = ref()

const fields = [{
    name: 'name',
    type: 'text',
    label: 'Name',
    placeholder: 'Enter your name'
}, {
    name: 'email',
    type: 'email',
    label: 'Email',
    placeholder: 'Enter your email'
}, {
    name: 'password',
    label: 'Password',
    type: 'password',
    placeholder: 'Enter your password'
}]

const validate = (state: any) => {
    const errors = []
    if (!state.name) errors.push({ path: 'name', message: 'Name is required' })
    if (!state.email) errors.push({ path: 'email', message: 'Email is required' })
    if (!state.password) errors.push({ path: 'password', message: 'Password is required' })
    return errors
}

async function onSubmit(data: any) {
    console.log('Submitted', data)

    form.value.formRef!.clear()

    try {
        const response = await $fetch('/api/login', {
            method: 'POST',
            body: data
        })

        // await refreshSession()
        // await navigateTo('/')

    } catch (err) {
        const errors: FormError[] = []

        if (err.statusCode === 400) {
            for (const issue of err.data.data.issues) {
                errors.push({ path: issue.path[0], message: issue.message })
            }
            form.value.formRef!.setErrors(errors)
        } else {
            console.error("Unhandled error", err)
        }
    }
}
</script>

<!-- eslint-disable vue/multiline-html-element-content-newline -->
<!-- eslint-disable vue/singleline-html-element-content-newline -->
<template>
    <UCard class="max-w-sm w-full bg-white/75 dark:bg-white/5 backdrop-blur">
        <UAuthForm ref="form" :fields="fields" :validate="validate" align="top" title="Create an account" :ui="{ base: 'text-center', footer: 'text-center' }" :submit-button="{ label: 'Create account' }" @submit="onSubmit">

            <template #icon>
                <span class="w-full flex justify-center" title="WindPress">
                    <Logo class="w-8 h-8 fill-current self-center" />
                </span>
            </template>


            <template #description>
                Already have an account? <NuxtLink to="/login" class="text-primary font-medium">Login</NuxtLink>.
            </template>

            <template #footer>
                By signing up, you agree to our <NuxtLink to="/" class="text-primary font-medium">Terms of Service</NuxtLink>.
            </template>
        </UAuthForm>
    </UCard>
</template>