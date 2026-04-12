<script setup lang="ts">
import { ref, onMounted, inject, watch } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { invokeParse } from '../composables/useTauri'
import { readClipboard } from '../composables/useClipboard'
import type { Article, AlertContext } from '../types'
import HomeButton from '../components/HomeButton.vue'
import SettingsButton from '../components/SettingsButton.vue'
import I18n from '@razein97/tauri-plugin-i18n'
import { Fab } from '../layouts'
import { IconClipboard, IconDeviceFloppy } from '@tabler/icons-vue'

const router = useRouter()
const route = useRoute()
const urlInput = ref('')
const progressBar = ref(false)
const urlInputPlaceholder = ref('')

const alertContext = inject<AlertContext | null>('alert')

async function pasteFromClipboard() {
  const text = await readClipboard()
  if (text) {
    urlInput.value = text
  }
}

async function onSubmit(e: Event) {
  e.preventDefault()
  progressBar.value = true
  try {
    const article = await invokeParse<Article>('add_article', { url: urlInput.value })
    router.replace({ name: 'article', params: { id: article.id } })
  }
  catch (err) {
    alertContext?.updateAlertContext?.('error', `Failed to add article: ${err}`)
    progressBar.value = false
  }
}

function setSharedUrl(url: string | null) {
  urlInput.value = decodeURIComponent(url || '')
}

watch(
  () => route.query,
  (newQuery) => {
    if (newQuery?.shared) {
      setSharedUrl(newQuery?.shared as string)
      router.replace({ query: { } })
    }
  },
  { deep: true, immediate: true },
)

onMounted(() => {
  const addArticle = I18n.getInstance().translate('add_article')
  urlInputPlaceholder.value = addArticle + ': https://example.com/article'
  setSharedUrl(route.query?.shared as string)
})
</script>

<template>
  <article>
    <div v-if="progressBar">
      <blockquote>{{ urlInput }}</blockquote>
      <article aria-busy="true" />
    </div>

    <form
      v-else
      class="container page"
      @submit="onSubmit"
    >
      <input
        v-model="urlInput"
        type="url"
        :placeholder="urlInputPlaceholder"
        required
      >
      <div role="group">
        <button
          class="outline"
          type="button"
          @click="pasteFromClipboard"
        >
          <IconClipboard />
        </button>
        <div role="group">
          <button type="submit">
            <IconDeviceFloppy />
          </button>
        </div>
      </div>
    </form>

    <Fab>
      <HomeButton />
      <SettingsButton />
    </Fab>
  </article>
</template>
