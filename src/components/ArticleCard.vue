<script setup lang="ts">
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import type { ArticleEntry } from '../types'
import I18n from '@razein97/tauri-plugin-i18n'
import { IconLoader } from '@tabler/icons-vue'

const props = defineProps<{
  article: ArticleEntry
}>()

const router = useRouter()

const title = computed(() => {
  if (props.article.title) {
    return { text: props.article.title, loaded: true }
  }
  try {
    const url = new URL(props.article.url)
    const pathParts = url.pathname.split('/').filter(s => s.length > 0)
    const lastPart = pathParts[pathParts.length - 1]
    if (lastPart) {
      return { text: lastPart, loaded: false }
    }
  }
  catch {}
  const untitled = I18n.getInstance().translate('untitled')
  return { text: untitled, loaded: false }
})

function goToArticle() {
  router.push({ name: 'article', params: { id: props.article.id } })
}
</script>

<template>
  <article
    style="cursor: pointer;"
    @click="goToArticle"
  >
    <header>
      <IconLoader
        v-if="!title?.loaded"
        size="2em"
        style="margin-right: 1em"
      />
      <h5 style="display: inline-block">
        {{ title?.text }}
      </h5>
    </header>
    <p><small>{{ article.created_at }}</small></p>
    <!-- eslint-disable-next-line vue/no-v-html -->
    <p><small><div v-html="article.snippet" /></small></p>
  </article>
</template>
