<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, watch, computed } from 'vue'
import { useRouter } from 'vue-router'
import { invokeParseLogError } from '../composables/useTauri'
import type { ArticleEntry } from '../types'
import ArticleCard from '../components/ArticleCard.vue'
import SettingsButton from '../components/SettingsButton.vue'
import I18n from '@razein97/tauri-plugin-i18n'
import { Fab } from '../layouts'
import { BookmarkPlus } from 'lucide-vue-next'

const router = useRouter()
const articles = ref<ArticleEntry[]>([])
const loading = ref(false)
const search = ref('')
const searchPlaceholder = ref('')
let timeout: ReturnType<typeof setTimeout> | null = null

async function fetchArticles() {
  if (loading.value) return

  loading.value = true
  const data = await invokeParseLogError<ArticleEntry[]>('get_articles', { offset: articles.value.length, query: query.value })

  if (data) {
    if (data.length === 0) {
      if (articles.value.length === 0 && query.value === null) {
        router.push({ name: 'addArticle' })
      }
    }
    else {
      articles.value.push(...data)
    }
  }
  loading.value = false
}

function onScroll(e: Event) {
  const target = e.target as HTMLElement
  const scrollTop = target.scrollTop
  const scrollHeight = target.scrollHeight
  const clientHeight = target.clientHeight

  if (scrollTop + clientHeight > scrollHeight - 100) {
    fetchArticles()
  }
}

function goToAddArticle() {
  router.push({ name: 'addArticle' })
}

const query = computed(() => {
  return search.value.length >= 3 ? search.value : null
})

watch(query, async () => {
  if (timeout) {
    clearTimeout(timeout)
  }
  timeout = setTimeout(async () => {
    articles.value = []
    await fetchArticles()
  }, 500)
})

onMounted(async () => {
  searchPlaceholder.value = I18n.getInstance().translate('search')
  await fetchArticles()
})

onBeforeUnmount(() => {
  if (timeout) {
    clearTimeout(timeout)
  }
})
</script>

<template>
  <main
    class="container page"
    @scroll="onScroll"
  >
    <input
      v-model="search"
      type="search"
      :placeholder="searchPlaceholder"
    >
    <div class="container">
      <ArticleCard
        v-for="article in articles"
        :key="article.id"
        :article="article"
      />
    </div>
  </main>

  <article
    v-if="loading"
    aria-busy="true"
  />

  <Fab>
    <button @click="goToAddArticle">
      <BookmarkPlus />
    </button>
    <div>
      <SettingsButton />
    </div>
  </Fab>
</template>
