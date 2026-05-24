<script setup lang="ts">
import { BookText, Settings } from 'lucide-vue-next'
</script>

<template>
  <BookText>
    <Settings
      :size="12"
      x="12"
      y="12"
      absolute-stroke-width
    />
  </BookText>
</template>
