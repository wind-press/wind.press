<script setup lang="ts">
const { footerLinks } = useFooterLinks()
</script>

<template>
  <!-- <USeparator icon="i-simple-icons-nuxtdotjs" class="h-px" /> -->
  <USeparator icon="i-custom-windpress" class="h-px" />

  <UFooter :ui="{ top: 'border-b border-(--ui-border)' }">
    <template #top>
      <UContainer>
        <UFooterColumns :columns="footerLinks">
          <template #right>
            <NewsletterForm />
          </template>
        </UFooterColumns>
      </UContainer>
    </template>

    <template #left>
      <UTooltip text="Made with hundred cups of coffee in Indonesia">
        <p class="text-(--ui-text-muted) text-sm">
          Made with hundred
          <UIcon name="fa6-solid:mug-hot" class="mx-0.5" /> of
          <UIcon name="ph:coffee-bean-fill" /> in <span class="font-bold">Indonesia</span>
        </p>
      </UTooltip>
    </template>

    <template #right>
      <!-- <UButton color="neutral" variant="ghost" to="https://go.nuxt.com/x" target="_blank" icon="i-simple-icons-x">
        <span class="sr-only">WindPress on X</span>
      </UButton> -->
      <UButton color="neutral" variant="ghost" to="/go/facebook" target="_blank" icon="i-simple-icons-facebook">
        <span class="sr-only">WindPress on Facebook</span>
      </UButton>
      <UButton color="neutral" variant="ghost" to="/go/discord" target="_blank" icon="i-simple-icons-discord">
        <span class="sr-only">WindPress on Discord</span>
      </UButton>
      <UButton color="neutral" variant="ghost" to="/go/github" target="_blank" icon="i-simple-icons-github">
        <span class="sr-only">WindPress on GitHub</span>
      </UButton>
    </template>
  </UFooter>
</template>
