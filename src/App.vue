<script setup lang="ts">
import { onMounted } from 'vue'
import { RouterView } from 'vue-router'
import { Alert, Theme } from '@/layouts'
import { setInset } from '@/composables/useInset'

onMounted(() => {
	setInset()
})
</script>

<template>
  <Alert>
    <Theme>
      <RouterView />
    </Theme>
  </Alert>
</template>
