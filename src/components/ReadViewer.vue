<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import type { Article } from '../types'
import { invokeNoParseLogError } from '../composables/useTauri'
import { openUrl } from '@tauri-apps/plugin-opener'
import DOMPurify from 'dompurify'
import HomeButton from './HomeButton.vue'
import LinkPopup from './LinkPopup.vue'
import SpeakBar from './SpeakBar.vue'
import { IconTrashX, IconX, IconCheck, IconWorldWww } from '@tabler/icons-vue'

const props = defineProps<{
  article: Article
}>()

const router = useRouter()
const divRef = ref<HTMLElement | null>(null)
const deleteModal = ref(false)
const externalUrl = ref<string | null>(null)
const safeHtml = computed(() => DOMPurify.sanitize(props.article.body))

function handleLinkClick(href: string) {
  externalUrl.value = href
}

function closeLinkPopup() {
  externalUrl.value = null
}

function toggleDeleteModal() {
  deleteModal.value = !deleteModal.value
}

async function deleteArticle() {
  await invokeNoParseLogError('delete_article', { id: props.article.id })
  router.replace({ name: 'home' })
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
      <div v-html="safeHtml" />
    </article>

    <LinkPopup
      :url="externalUrl"
      @close="closeLinkPopup"
    />

    <dialog :open="deleteModal">
      <article>
        <header>
          <button
            aria-label="Close"
            rel="prev"
            @click="toggleDeleteModal"
          />
          <IconTrashX
            size="2em"
            style="margin-right: 1em"
          />
          <span data-i18n="delete_article" />
        </header>
        <strong>{{ article.title }}</strong>
        <footer>
          <button
            class="secondary"
            @click="toggleDeleteModal"
          >
            <IconX />
          </button>
          <button @click="deleteArticle">
            <IconCheck />
          </button>
        </footer>
      </article>
    </dialog>

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
