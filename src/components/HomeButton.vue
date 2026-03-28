<script setup lang="ts">
import { useRouter } from 'vue-router'

const router = useRouter()

function goHome() {
  router.push({ name: 'home' })
}
</script>

<template>
  <button @click="goHome">
    <i class="ti ti-home">&#xeaeb;</i>
  </button>
</template>
