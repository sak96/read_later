<script setup lang="ts">
import { ref, provide, computed } from 'vue'
import type { AlertContext, AlertStatus } from '../types'
import { ALERT_DURATION_MS } from '../constants'

const message = ref<string | null>(null)
const status = ref<AlertStatus>('info')

function updateAlertContext(newStatus: AlertStatus, newMessage: string) {
  status.value = newStatus
  message.value = newMessage
  setTimeout(() => {
    message.value = null
  }, ALERT_DURATION_MS)
}

provide<AlertContext>('alert', {
  updateAlertContext,
})

const alertStyle = computed(() => {
  const { bgcolor, color } = {
    info: { bgcolor: '#B7D9FC', color: '#017FC0' },
    error: { bgcolor: '#F6CABF', color: '#D93526' },
    success: { bgcolor: '#39F1A6', color: '#00895A' },
  }[status.value]

  return `position: fixed; bottom: var(--safe-area-inset-bottom, 0); z-index: 1000; background-color: ${bgcolor}; color: ${color}`
})
</script>

<template>
  <slot />

  <article
    v-if="message"
    class="pico container-fluid"
    :style="alertStyle"
  >
    {{ message }}
  </article>
</template>
