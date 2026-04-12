<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import type { Article } from '../types'
import { invokeNoParseLogError } from '../composables/useTauri'
import { openUrl } from '@tauri-apps/plugin-opener'
import DOMPurify from 'dompurify'
import HomeButton from './HomeButton.vue'
import ConfirmModal from './ConfirmModal.vue'
import SpeakBar from './SpeakBar.vue'
import { IconTrashX, IconWorldWww } from '@tabler/icons-vue'

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
  <div class="container">
    <article
      ref="divRef"
      class="page reader_view overflow-auto"
    >
      <h1>{{ article.title }}</h1>
      <!-- eslint-disable-next-line vue/no-v-html -->
      <div
        class="article"
        v-html="safeHtml"
      />
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

    <aside style="position: sticky; bottom: var(--safe-area-inset-bottom, 0);">
      <nav>
        <SpeakBar
          :div-ref="divRef!"
          :title="article.title"
        />
        <div role="group">
          <HomeButton />
          <button @click="openUrl(article.url)">
            <IconWorldWww />
          </button>
          <button
            class="secondary"
            @click="toggleDeleteModal"
          >
            <IconTrashX />
          </button>
        </div>
      </nav>
    </aside>
  </div>
</template>
