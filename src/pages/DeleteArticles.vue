<script setup lang="ts">
import { ref, computed, onMounted, inject } from 'vue'
import { useRouter } from 'vue-router'
import { invokeParseLogError } from '../composables/useTauri'
import type { ArticleEntry, AlertContext } from '../types'
import I18n from '@razein97/tauri-plugin-i18n'
import { Fab } from '../layouts'
import HomeButton from '../components/HomeButton.vue'
import ConfirmModal from '../components/ConfirmModal.vue'
import { Anchor, ListChecks, Trash2 } from 'lucide-vue-next'
import { SCROLL_THRESHOLD } from '../constants'

const router = useRouter()
const alertContext = inject<AlertContext | null>('alert')

const articles = ref<ArticleEntry[]>([])
const selected = ref(new Set<number>())
const anchorIndex = ref<number | null>(null)

const loading = ref(false)
const hasMore = ref(true)
const error = ref<string | null>(null)
const showMenu = ref(false)
const showConfirm = ref(false)

const selectBelow = ref<number | null>(null)

const selectFuture = ref(false)

const selectedCount = computed(() =>
  `${selected.value.size}${selectBelow.value !== null ? '+' : ''}`,
)

function titleOf(article: ArticleEntry): string {
  if (article.title) return article.title

  try {
    const parts = new URL(article.url).pathname.split('/').filter(Boolean)
    return parts.pop() || I18n.getInstance().translate('untitled')
  }
  catch {
    return I18n.getInstance().translate('untitled')
  }
}

async function fetchArticles(): Promise<boolean> {
  if (loading.value || !hasMore.value) return true

  loading.value = true
  error.value = null

  try {
    const data = await invokeParseLogError<ArticleEntry[]>('get_articles', {
      offset: articles.value.length,
      query: null,
    })

    if (data === null) {
      error.value = 'Failed to load articles'
      return false
    }

    articles.value.push(...data)

    if (selectFuture.value) {
      for (const article of data) {
        selected.value.add(article.id)
      }
    }

    hasMore.value = data.length === 100

    return true
  }
  finally {
    loading.value = false
  }
}

function onScroll(event: Event) {
  const el = event.target as HTMLElement

  if (el.scrollTop + el.clientHeight >= el.scrollHeight - SCROLL_THRESHOLD) {
    fetchArticles()
  }
}

function toggle(index: number) {
  const id = articles.value[index].id

  if (selected.value.has(id)) selected.value.delete(id)
  else selected.value.add(id)

  anchorIndex.value = index
}

function selectAll() {
  selected.value = new Set(articles.value.map(article => article.id))
  selectBelow.value = articles.value.at(-1)?.id ?? null
  selectFuture.value = true
}

function selectAbove() {
  if (anchorIndex.value === null) return

  for (let i = 0; i <= anchorIndex.value; i++) {
    selected.value.add(articles.value[i].id)
  }
}

function selectBelowRange() {
  if (anchorIndex.value === null) return

  for (let i = anchorIndex.value; i < articles.value.length; i++) {
    selected.value.add(articles.value[i].id)
  }

  selectBelow.value = articles.value.at(-1)?.id ?? null
  selectFuture.value = true
}

function unselectAll() {
  selected.value.clear()
  selectBelow.value = null
  selectFuture.value = false
  anchorIndex.value = null
}

function selectAction(action: () => void) {
  action()
  showMenu.value = false
}

async function confirmDelete() {
  showConfirm.value = false

  const result = await invokeParseLogError<number>('delete_articles', {
    ids: [...selected.value],
    selectBelow: selectBelow.value,
  })

  if (result === null) {
    alertContext?.updateAlertContext?.('error', 'Failed to delete articles.')
    return
  }

  alertContext?.updateAlertContext?.('success', `Deleted ${result} articles.`)
  router.replace({ name: 'home' })
}

onMounted(fetchArticles)
</script>

<template>
  <main class="container page" @scroll="onScroll">
    <h3>
      <span data-i18n="delete_articles" /> ({{ selectedCount }})
    </h3>
    <span data-i18n="delete_single_article" />

    <div class="container">
      <article v-for="(article, index) in articles" :key="article.id">
        <label>
          <input
            type="checkbox"
            :checked="selected.has(article.id)"
            @change="toggle(index)"
          >
          <Anchor v-if="anchorIndex === index" title="Anchor row for ranges" />
          {{ titleOf(article) }}
        </label>
      </article>

      <p
        v-if="!loading && !articles.length && !error"
        class="center"
        data-i18n="no_articles"
      />
    </div>

    <article v-if="error" class="error">
      {{ error }}
    </article>
  </main>

  <article v-if="loading" aria-busy="true" />

  <dialog :open="showMenu">
    <article>
      <header>
        <button
          aria-label="Close"
          rel="prev"
          @click="showMenu = false"
        />
        <strong data-i18n="delete_articles" />
      </header>

      <div class="grid" style="grid-template-columns: 1fr">
        <div role="button" data-i18n="select_all" @click="selectAction(selectAll)" />

        <div
          role="button"
          :aria-disabled="anchorIndex === null"
          @click="selectAction(selectAbove)"
        >
          <span data-i18n="select_above" />
          <Anchor />
        </div>

        <div
          role="button"
          :aria-disabled="anchorIndex === null"
          @click="selectAction(selectBelowRange)"
        >
          <span data-i18n="select_below" />
          <Anchor />
        </div>

        <div
          role="button"
          class="secondary"
          data-i18n="unselect_all"
          @click="selectAction(unselectAll)"
        />
      </div>
    </article>
  </dialog>

  <ConfirmModal
    :icon="Trash2"
    i18n-key="delete_articles"
    :message="`Delete ${selectedCount} articles?`"
    :show="showConfirm"
    @confirm="confirmDelete"
    @close="showConfirm = false"
  />

  <Fab>
    <button
      :disabled="!selected.size && selectBelow === null"
      @click="showConfirm = true"
    >
      <Trash2 />
    </button>

    <button @click="showMenu = true">
      <ListChecks />
    </button>

    <div>
      <HomeButton />
    </div>
  </Fab>
</template>
