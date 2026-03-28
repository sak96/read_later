<script setup lang="ts">
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import type { ArticleEntry } from '../types'

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
	} catch {
		// Invalid URL
	}
  
	return { text: 'Untitled', loaded: false }
})

function goToArticle() {
	router.push({ name: 'article', params: { id: props.article.id } })
}
</script>

<template>
  <article style="cursor: pointer;" @click="goToArticle">
    <h3>
      <i v-if="!title.loaded" class="ti ti-loader">&#xeca3;</i>
      {{ title.text }}
    </h3>
    <p><small>{{ article.created_at }}</small></p>
  </article>
</template>
