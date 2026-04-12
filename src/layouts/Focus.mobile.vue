<script setup lang="ts">
import { popIntentQueueAndExtractText } from 'tauri-plugin-mobile-sharetarget-api'
import { useRouter, useRoute } from 'vue-router'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { platform } from '@tauri-apps/plugin-os'
import { ref, onMounted, onUnmounted } from 'vue'
import '@saurl/tauri-plugin-safe-area-insets-css-api'

const focusUnlistener = ref<UnlistenFn | null>(null)
const currentPlatform: string = platform()

const router = useRouter()
const route = useRoute()
async function hanldeShare() {
  // set inset on all foucs
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
  <slot />
</template>
