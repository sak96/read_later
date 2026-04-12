<script setup lang="ts">
import { Home } from 'lucide-vue-next'
import { useRouter } from 'vue-router'

const router = useRouter()

function goHome() {
  router.push({ name: 'home' })
}
</script>

<template>
  <button @click="goHome">
    <Home />
  </button>
</template>
