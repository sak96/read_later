<script setup lang="ts">
import { useRouter } from 'vue-router'

const router = useRouter()

function goToSettings() {
  router.push({ name: 'settings' })
}
</script>

<template>
  <button @click="goToSettings">
    <i class="ti ti-settings-filled">&#xf69e;</i>
  </button>
</template>
