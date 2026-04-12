<script setup lang="ts">
import { ref, onMounted, inject } from 'vue'
import { useRouter } from 'vue-router'
import { invokeParse, invokeNoParseLogError } from '../composables/useTauri'
import { Channel } from '@tauri-apps/api/core'
import type { Article, FetchProgress, AlertContext } from '../types'
import ReadViewer from '../components/ReadViewer.vue'
import { Trash2, Loader, CloudDownload, DatabaseSearch, LucideIcon } from 'lucide-vue-next'

const props = defineProps<{
  id: number
}>()

const router = useRouter()

type PageMode
  = | { type: 'fetching', progress: FetchProgress | null }
    | { type: 'returned', article: Article }

const mode = ref<PageMode>({ type: 'fetching', progress: null })

const alertContext = inject<AlertContext | null>('alert')

const onProgress = (progress: FetchProgress | null) => {
  mode.value = { type: 'fetching', progress }
}

async function loadArticle() {
  try {
    const channel = new Channel<FetchProgress>(onProgress)
    const result = await invokeParse<Article>('get_article', { id: props.id, onProgress: channel })
    mode.value = { type: 'returned', article: result } as PageMode
    // eslint-disable-next-line @typescript-eslint/no-explicit-any -- call private functions
    (channel as any).cleanupCallback()
  }
  catch (err) {
    alertContext?.updateAlertContext?.('error', `Failed to fetch article: ${err}`)
    await invokeNoParseLogError('delete_article', { id: props.id })
    router.push({ name: 'home' })
  }
}

async function deleteArticle() {
  await invokeNoParseLogError('delete_article', { id: props.id })
  alertContext?.updateAlertContext?.('success', 'Deleted article.')
  router.push({ name: 'home' })
}

onMounted(async () => {
  mode.value = { type: 'fetching', progress: null }
  await loadArticle()
})

function getProgressInfo(progress: FetchProgress | null): { icon: LucideIcon, title: string } {
  if (progress) {
    if ('Downloading' in progress) {
      return { icon: CloudDownload, title: progress.Downloading }
    }
    if ('Parsing' in progress) {
      return { icon: DatabaseSearch, title: progress.Parsing }
    }
  }
  return { icon: Loader, title: '...' }
}
</script>

<template>
  <main
    v-if="mode.type === 'fetching'"
    class="container page"
    style="display: flex; justify-content: center; align-items: center;"
  >
    <article style="width: 100%;">
      <h2>
        <component :is="getProgressInfo(mode.progress).icon" />
        <p>{{ getProgressInfo(mode.progress).title }}</p>
      </h2>
      <progress />
      <footer
        v-if="mode.progress && 'Downloading' in mode.progress"
        dir="rtl"
      >
        <button
          class="secondary"
          @click="deleteArticle"
        >
          <Trash2 />
        </button>
      </footer>
    </article>
  </main>

  <ReadViewer
    v-else-if="mode.type === 'returned'"
    :article="mode.article"
  />
</template>
