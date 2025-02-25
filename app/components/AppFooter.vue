<script setup lang="ts">
const columnLinks = [{
  label: 'Resources',
  children: [{
    label: 'Help center'
  }, {
    label: 'Docs'
  }, {
    label: 'Roadmap'
  }, {
    label: 'Changelog'
  }]
}, {
  label: 'Features',
  children: [{
    label: 'Affiliates'
  }, {
    label: 'Portal'
  }, {
    label: 'Jobs'
  }, {
    label: 'Sponsors'
  }]
}, {
  label: 'Company',
  children: [{
    label: 'About'
  }, {
    label: 'Pricing'
  }, {
    label: 'Careers'
  }, {
    label: 'Blog'
  }]
}]

const links = [
  {
    label: 'Contact',
    to: 'https://rosua.org/support-portal',
    target: '_blank'
  },
  {
    label: 'My Orders',
    to: 'https://rosua.org/checkout/order-history',
    target: '_blank'
  },
];

const toast = useToast()

const email = ref('')
const loading = ref(false)

function onSubmit() {
  loading.value = true

  setTimeout(() => {
    toast.add({
      title: 'Subscribed!',
      description: 'You\'ve been subscribed to our newsletter.'
    })

    loading.value = false
  }, 1000)
}
</script>

<template>
  <UFooter :links="links">
    <!-- <template #top>
      <UFooterColumns :links="columnLinks">
        <template #right>
          <form @submit.prevent="onSubmit">
            <UFormGroup label="Subscribe to our newsletter" :ui="{ container: 'mt-3' }">
              <UInput v-model="email" type="email" placeholder="Enter your email" :ui="{ icon: { trailing: { pointer: '' } } }" required size="xl" autocomplete="off" class="max-w-sm" disabled>
                <template #trailing>
                  <UButton type="submit" size="xs" color="primary" :label="loading ? 'Subscribing' : 'Subscribe'" :loading="loading" disabled />
                </template>
</UInput>
</UFormGroup>
</form>
</template>
</UFooterColumns>
</template> -->

    <template #left>
      <p class="text-gray-500 dark:text-gray-400 text-base items-center text-center" title="Made with hundred cups of coffee in Indonesia">
        Made with hundred
        <UIcon name="fa6-solid:mug-hot" class="mx-0.5" /> of
        <UIcon name="ph:coffee-bean-fill" /> in <span class="font-bold">Indonesia</span>
      </p>
    </template>

    <template #right>
      <!-- <UColorModeButton size="sm" /> -->
      <!-- <UButton aria-label="WindPress on X" to="/go/x" target="_blank" icon="i-simple-icons-x" v-bind="($ui.button.secondary as any)" /> -->
      <UButton aria-label="WindPress on Facebook" to="/go/facebook" target="_blank" icon="i-simple-icons-facebook" v-bind="($ui.button.secondary as any)" />
      <UButton aria-label="WindPress on Discord" to="/go/discord" target="_blank" icon="i-simple-icons-discord" v-bind="($ui.button.secondary as any)" />
      <UButton aria-label="WindPress on GitHub" to="/go/github" target="_blank" icon="i-simple-icons-github" v-bind="($ui.button.secondary as any)" />
    </template>
  </UFooter>
</template>
