<script setup lang="ts">
import { IconHome } from '@tabler/icons-vue'
import { useRouter } from 'vue-router'

const router = useRouter()

function goHome() {
  router.push({ name: 'home' })
}
</script>

<template>
  <button @click="goHome">
    <IconHome />
  </button>
</template>
