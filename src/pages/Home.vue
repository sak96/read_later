<script setup lang="ts">
import { ref, onMounted, watch, computed } from 'vue'
import { useRouter } from 'vue-router'
import { invokeParseLogError } from '../composables/useTauri'
import type { ArticleEntry } from '../types'
import ArticleCard from '../components/ArticleCard.vue'
import SettingsButton from '../components/SettingsButton.vue'
import { Fab } from '../layouts'

const router = useRouter()
const articles = ref<ArticleEntry[]>([])
const loading = ref(false)
const search = ref('')

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
  articles.value = []
  await fetchArticles()
})

onMounted(async () => {
  await fetchArticles()
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
      placeholder="Search"
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
      <i class="ti ti-bookmark-plus">&#xfa60;</i>
    </button>
    <div>
      <SettingsButton />
    </div>
  </Fab>
</template>
