<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { Channel } from '@tauri-apps/api/core'
import { invokeNoParse } from '../composables/useTauri'
import { CloudSync } from 'lucide-vue-next'

const router = useRouter()
const progress = ref(0)
const total = ref(0)
const errorMessage = ref('')

interface SyncProgress {
  count_processed: number
  total_count: number
}

const onProgress = (payload: SyncProgress) => {
  progress.value = payload.count_processed
  total.value = payload.total_count
}

onMounted(async () => {
  try {
    const channel = new Channel<SyncProgress>(onProgress)
    await invokeNoParse('sync_articles', { progressChannel: channel });
    (channel as any).cleanupCallback()
    await router.push('/home')
  }
  catch (e: any) {
    errorMessage.value = e.toString() || 'An unknown error occurred during sync.'
    console.error('Sync error:', e)
    await router.push('/home')
  }
})
</script>

<template>
  <main
    class="container"
    style="display: flex; justify-content: center; align-items: center; min-height: 100vh;"
  >
    <article style="width: 100%; max-width: 400px; text-align: center;">
      <CloudSync
        :size="48"
        style="color: var(--pico-primary);"
      />
      <h2 data-i18n="sync_title">
        Synchronizing Articles
      </h2>
      <progress
        v-if="total > 0"
        :value="progress"
        :max="total"
      />
      <p v-if="total > 0">
        {{ progress }} / {{ total }}
      </p>
      <p
        v-if="errorMessage"
        style="color: var(--pico-del-color);"
      >
        {{ errorMessage }}
      </p>
    </article>
  </main>
</template>
