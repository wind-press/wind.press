<template>
    <UPageCard to="https://daisyui.com/store/?aff=GZ5Zd" target="_blank" :ui="{
        header: { padding: 'p-0 sm:p-0' },
        rounded: 'rounded',
        body: { padding: 'p-2 sm:p-2', base: 'text-center' }
    }">
        <template #header>
            <UColorModeImage light="https://img.daisyui.com/images/store/large/saas-landing/saas-landing-10.webp" dark="https://img.daisyui.com/images/store/large/agency-landing/agency-landing-1.webp" alt="DaisyUI templates" />
        </template>
        <div class="inline-flex gap-1 my-2">
            DaisyUI

            <UBadge variant="subtle" size="xs" class="text-[10px]">
                Ads
            </UBadge>
        </div>
        <p class="text-sm dark:text-gray-400 text-gray-500 pb-1">
            Production-ready templates for Tailwind CSS made with DaisyUI.
        </p>
    </UPageCard>
</template>