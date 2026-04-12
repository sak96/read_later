<script setup lang="ts">
import { useRouter } from 'vue-router'
import { IconSettingsFilled } from '@tabler/icons-vue'

const router = useRouter()

function goToSettings() {
  router.push({ name: 'settings' })
}
</script>

<template>
  <button @click="goToSettings">
    <IconSettingsFilled />
  </button>
</template>
