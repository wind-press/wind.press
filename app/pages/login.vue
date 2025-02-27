<script setup lang="ts">
import type { FormError } from '#ui/types'
import Logo from '../components/Logo.vue';

definePageMeta({
    layout: 'auth'
})

useSeoMeta({
    title: 'Login'
})

const { loggedIn, user, fetch: refreshSession } = useUserSession()

const form = ref()

const fields = [{
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
    const errors: FormError[] = []
    if (!state.email) errors.push({ path: 'email', message: 'Email is required' })
    if (!state.password) errors.push({ path: 'password', message: 'Password is required' })
    return errors
}

async function onSubmit(data: any) {
    form.value.formRef!.clear()

    try {
        const response = await $fetch('/api/login', {
            method: 'POST',
            body: data
        })

        await refreshSession()
        await navigateTo('/')

    } catch (err) {
        const errors: FormError[] = []

        if (err.statusCode === 400) {
            for (const issue of err.data.data.issues) {
                errors.push({ path: issue.path[0], message: issue.message })
            }
            form.value.formRef!.setErrors(errors)
        } else if (err.statusCode === 401) {
            errors.push({ path: 'password', message: 'Invalid credentials' })
            form.value.formRef!.setErrors(errors)
        } else if (err.statusCode === 422) {
            form.value.formRef!.setErrors(err.data.errors.map((err) => ({
                message: err.message,
                path: err.path,
            })))
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
        <UAuthForm ref="form" @submit="onSubmit" :fields="fields" :validate="validate" title="Welcome back" align="top" :ui="{ base: 'text-center', footer: 'text-center' }" :submit-button="{ trailingIcon: 'i-heroicons-arrow-right-20-solid' }">

            <template #icon>
                <span class="w-full flex justify-center" title="WindPress">
                    <Logo class="w-8 h-8 fill-current self-center" />
                </span>
            </template>

            <template #description>
                Don't have an account? <NuxtLink to="/signup" class="text-primary font-medium">Sign up</NuxtLink>.
            </template>

            <template #password-hint>
                <NuxtLink to="/" class="text-primary font-medium">Forgot password?</NuxtLink>
            </template>

            <template #footer>
                By signing in, you agree to our <NuxtLink to="/" class="text-primary font-medium">Terms of Service</NuxtLink>.
            </template>
        </UAuthForm>
    </UCard>
</template>