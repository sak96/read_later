<script setup lang="ts">
import { ref, onMounted, inject } from 'vue'
import { useRouter } from 'vue-router'
import { invokeParse, invokeNoParseLogError } from '@/composables/useTauri'
import { Channel } from '@tauri-apps/api/core';
import type { Article, FetchProgress } from '@/types'
import ReadViewer from '@/components/ReadViewer.vue'

const props = defineProps<{
  id: number
}>()

const router = useRouter()

type PageMode = 
  | { type: 'fetching'; progress: FetchProgress | null }
  | { type: 'returned'; article: Article }

const mode = ref<PageMode>({ type: 'fetching', progress: null })

const alert = inject<(message: string, status: 'success' | 'info' | 'error') => void>('alert')

const onProgress = (progress: FetchProgress | null) => {
  mode.value = { type: 'fetching', progress }
}

async function loadArticle() {
  try {
    const channel = new Channel<FetchProgress>(onProgress);
    const result = await invokeParse<Article>('get_article', { id: props.id, onProgress: channel })
    mode.value = { type: 'returned', article: result }
    channel.cleanupCallback()
  } catch (err) {
    if (alert) {
      alert(`Failed to fetch article: ${err}`, 'error')
    }
    await invokeNoParseLogError('delete_article', { id: props.id })
    router.push({ name: 'home' })
  }
}

async function deleteArticle() {
  await invokeNoParseLogError('delete_article', { id: props.id })
  if (alert) {
    alert('Deleted article.', 'success')
  }
  router.push({ name: 'home' })
}

onMounted(async () => {
  mode.value = { type: 'fetching', progress: null }
  await loadArticle()
})

function getProgressInfo(progress: FetchProgress | null): { icon: string; iconCode: string; title: string } {
  if (!progress) {
    return { icon: 'ti-loader', iconCode: '\ueca3', title: '...' }
  }
  
  if ('Downloading' in progress) {
    return { icon: 'ti-cloud-download', iconCode: '\uea71', title: progress.Downloading }
  }
  
  if ('Parsing' in progress) {
    return { icon: 'ti-database-search', iconCode: '\ufa18', title: progress.Parsing }
  }
  
  return { icon: 'ti-loader', iconCode: '\ueca3', title: '...' }
}
</script>

<template>
  <main v-if="mode.type === 'fetching'" class="container page" style="display: flex; justify-content: center; align-items: center;">
    <article style="width: 100%;">
      <h2 class="ti">
        {{ getProgressInfo(mode.progress).iconCode }}
        <p>{{ getProgressInfo(mode.progress).title }}</p>
      </h2>
      <progress></progress>
      <footer v-if="mode.progress && 'Downloading' in mode.progress" dir="rtl">
        <button class="secondary" @click="deleteArticle">
          <i class="ti ti-trash-x">&#xf784;</i>
        </button>
      </footer>
    </article>
  </main>

  <ReadViewer 
    v-else-if="mode.type === 'returned'" 
    :article="mode.article" 
  />
</template>
