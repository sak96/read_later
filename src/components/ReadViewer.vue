<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import type { Article } from '../types'
import { invokeNoParseLogError } from '../composables/useTauri'
import { openUrl } from '@tauri-apps/plugin-opener'
import DOMPurify from 'dompurify'
import ConfirmModal from './ConfirmModal.vue'
import SpeakBar from './SpeakBar.vue'
import { Trash2, Globe } from 'lucide-vue-next'

const props = defineProps<{
  article: Article
}>()

const router = useRouter()
const divRef = ref<HTMLElement | null>(null)
const externalUrl = ref<string | null>(null)
const deleteModal = ref(false)
const safeHtml = computed(() => DOMPurify.sanitize(props.article.body))

function handleLinkClick(href: string) {
  externalUrl.value = href
}

function toggleDeleteModal() {
  deleteModal.value = !deleteModal.value
}

async function deleteArticle() {
  await invokeNoParseLogError('delete_article', { id: props.article.id })
  router.replace({ name: 'home' })
}

function openExternalUrl(url: string) {
  openUrl(url)
  externalUrl.value = null
}

function setLinkCallbacks() {
  if (!divRef.value) return

  const links = divRef.value.querySelectorAll('a')
  links.forEach((link) => {
    link.addEventListener('click', (e) => {
      e.preventDefault()
      const href = link.getAttribute('href')
      if (href) {
        handleLinkClick(href)
      }
    })
  })
}

onMounted(() => {
  setLinkCallbacks()
})
</script>

<template>
  <div>
    <article
      ref="divRef"
      class="page reader_view overflow-auto"
    >
      <h1>{{ article.title }}</h1>
      <!-- eslint-disable vue/no-v-html -->
      <div
        class="article"
        v-html="safeHtml"
      />
      <!-- eslint-enable vue/no-v-html -->
    </article>

    <ConfirmModal
      :icon="Globe"
      i18n-key="open_url"
      :message="externalUrl ?? ''"
      :show="!!externalUrl"
      @confirm="openExternalUrl(externalUrl!)"
      @close="externalUrl = null"
    />

    <ConfirmModal
      :icon="Trash2"
      i18n-key="delete_article"
      :message="article.title"
      :show="deleteModal"
      @confirm="deleteArticle"
      @close="deleteModal = false"
    />

    <SpeakBar
      :div-ref="divRef!"
      :title="article.title"
    >
      <button @click="openUrl(article.url)">
        <Globe />
      </button>
      <button
        class="secondary"
        @click="toggleDeleteModal"
      >
        <Trash2 />
      </button>
    </SpeakBar>
  </div>
</template>
