<script setup lang="ts">
import { ref, onMounted, inject } from 'vue'
import { useRouter } from 'vue-router'
import { invokeParse, invokeNoParseLogError } from '../composables/useTauri'
import type { Article, AlertContext } from '../types'
import ReadViewer from '../components/ReadViewer.vue'
import { Trash2, Loader } from 'lucide-vue-next'

const props = defineProps<{
  id: number
}>()

const router = useRouter()

type PageMode
  = | { type: 'fetching' }
    | { type: 'returned', article: Article }

const mode = ref<PageMode>({ type: 'fetching' })

const alertContext = inject<AlertContext | null>('alert')

async function waitForTauriReady(): Promise<void> {
  while (!('__TAURI_INTERNALS__' in window)) {
    await new Promise(resolve => setTimeout(resolve, 10))
  }
}

async function loadArticle() {
  mode.value = { type: 'fetching' }
  await waitForTauriReady()
  try {
    let result: Article | null = null

    while (result === null) {
      result = await invokeParse<Article | null>('get_article', {
        id: props.id,
      })
      if (result === null) {
        await new Promise(resolve => setTimeout(resolve, 500))
      }
    }
    mode.value = { type: 'returned', article: result } as PageMode
  }
  catch (err) {
    alertContext?.updateAlertContext?.('error', `Failed to fetch article: ${err}`)
    await invokeNoParseLogError('delete_article', { id: props.id })
    router.replace({ name: 'home' })
  }
}

async function deleteArticle() {
  await invokeNoParseLogError('delete_article', { id: props.id })
  alertContext?.updateAlertContext?.('success', 'Deleted article.')
  router.replace({ name: 'home' })
}

onMounted(async () => {
  await loadArticle()
})
</script>

<template>
  <main
    v-if="mode.type === 'fetching'"
    class="page"
    style="display: flex; justify-content: center; align-items: center;"
  >
    <article style="width: 100%;">
      <h1>
        <Loader :size="128"/>
        <progress />
      </h1>
      <footer dir="rtl">
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
    v-else
    :article="mode.article"
    @refreshed="loadArticle"
  />
</template>
