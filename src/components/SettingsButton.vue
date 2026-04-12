<script setup lang="ts">
import { useRouter } from 'vue-router'
import { Settings } from 'lucide-vue-next'

const router = useRouter()

function goToSettings() {
  router.push({ name: 'settings' })
}
</script>

<template>
  <button @click="goToSettings">
    <Settings />
  </button>
</template>
