<script setup lang="ts">
import { ref, onMounted, inject } from 'vue'
import { useRouter } from 'vue-router'
import { invokeParse, invokeNoParseLogError } from '../composables/useTauri'
import { getSetting } from '../composables/useSettings'
import { FETCHER_MODE } from '../constants'
import type { Article, ArticleResponse, AlertContext } from '../types'
import ReadViewer from '../components/ReadViewer.vue'
import FetcherMode from '../components/FetcherMode.vue'
import { Loader, Check, Trash2 } from 'lucide-vue-next'

const props = defineProps<{
  id: number
}>()

const router = useRouter()

type PageMode
  = | { type: 'fetching' }
    | { type: 'chooser' }
    | { type: 'returned', article: Article }

const mode = ref<PageMode>({ type: 'fetching' })

// One-time fetcher mode chosen in the chooser; null until FetcherMode reports
// the global setting, and reset every time the chooser is shown.
const chooserMode = ref<string | null>(null)

const alertContext = inject<AlertContext | null>('alert')

async function waitForTauriReady(): Promise<void> {
  while (!('__TAURI_INTERNALS__' in window)) {
    await new Promise(resolve => setTimeout(resolve, 10))
  }
}

async function loadArticle(trigger = false, selectedMode?: string) {
  mode.value = { type: 'fetching' }
  await waitForTauriReady()
  try {
    // One-time fetcher mode: explicit choice wins, else the global setting.
    const fetcherMode = selectedMode
      || await getSetting(FETCHER_MODE)
      || 'html'

    let result = await invokeParse<ArticleResponse>('get_article', {
      id: props.id,
      fetcherMode,
      trigger,
    })

    while (result.status === 'db_locked') {
      await new Promise(resolve => setTimeout(resolve, 500))
      result = await invokeParse<ArticleResponse>('get_article', {
        id: props.id,
        fetcherMode,
        trigger,
      })
    }

    if (result.status === 'ok') {
      mode.value = { type: 'returned', article: result.article } as PageMode
    }
    else {
      showChooser()
    }
  }
  catch (err) {
    alertContext?.updateAlertContext?.('error', `Failed to fetch article: ${err}`)
    showChooser()
  }
}

function showChooser() {
  chooserMode.value = null
  mode.value = { type: 'chooser' }
}

function onTick() {
  const selected = chooserMode.value
  if (!selected) return
  void loadArticle(true, selected)
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
        <Loader :size="128" />
        <progress />
      </h1>
    </article>
  </main>

  <main
    v-else-if="mode.type === 'chooser'"
    class="page"
    style="display: flex; justify-content: center; align-items: center;"
  >
    <article style="width: 100%;">
      <FetcherMode
        :persist="false"
        @change="chooserMode = $event"
      />
      <footer>
        <div role="group">
          <button
            class="secondary"
            @click="deleteArticle"
          >
            <Trash2 />
          </button>
          <button
            :disabled="!chooserMode"
            @click="onTick"
          >
            <Check />
          </button>
        </div>
      </footer>
    </article>
  </main>

  <ReadViewer
    v-else
    :article="mode.article"
    @refreshed="loadArticle"
  />
</template>
