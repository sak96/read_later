<script setup lang="ts">
import { ref, provide, computed } from 'vue'
import { writeText } from '@tauri-apps/plugin-clipboard-manager'
import type { AlertContext, AlertStatus } from '../types'
import { ALERT_DURATION_MS } from '../constants'

const message = ref<string | null>(null)
const status = ref<AlertStatus>('info')
const alertTimeout = ref<ReturnType<typeof setTimeout> | undefined>()

function clearAlertTimeout() {
  if (alertTimeout.value) {
    clearTimeout(alertTimeout.value)
    alertTimeout.value = undefined
  }
}

function updateAlertContext(newStatus: AlertStatus, newMessage: string) {
  clearAlertTimeout()
  status.value = newStatus
  message.value = newMessage
  alertTimeout.value = setTimeout(() => {
    message.value = null
  }, ALERT_DURATION_MS)
}

async function click() {
  if (!message.value) {
    return
  }
  await writeText(message.value)
  message.value = null
  clearAlertTimeout()
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
    @click="click"
  >
    {{ message }}
  </article>
</template>
