<script setup lang="ts">
import { Volume, Settings } from 'lucide-vue-next'
</script>

<template>
  <Volume>
    <Settings
      :size="12"
      x="12"
      y="12"
      absolute-stroke-width
    />
  </Volume>
</template>
