<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { RouterView } from 'vue-router'
import { useRouter, useRoute } from 'vue-router'
import { Alert, Theme } from './layouts'
import { popIntentQueueAndExtractText } from 'tauri-plugin-mobile-sharetarget-api'
import '@saurl/tauri-plugin-safe-area-insets-css-api'
import { platform } from '@tauri-apps/plugin-os'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

const focusUnlistener = ref<UnlistenFn | null>(null)
const router = useRouter()
const route = useRoute()
const currentPlatform: string = platform()

async function hanldeShare() {
  const url = await popIntentQueueAndExtractText()
  if (url) {
    const shareArticle = { name: 'addArticle', query: { shared: url } }
    if (route.name === 'addArticle') {
      router.replace({ query: { shared: url } })
    }
    else {
      router.push(shareArticle)
    }
  }
}

onMounted(async () => {
  focusUnlistener.value = await listen(
    currentPlatform === 'android' ? 'tauri://focus' : 'new-intent',
    hanldeShare,
  )
  await hanldeShare()
})

onUnmounted(() => {
  focusUnlistener.value?.()
})
</script>

<template>
  <Alert>
    <Theme>
      <RouterView key="$router.fullPath" />
    </Theme>
  </Alert>
</template>
