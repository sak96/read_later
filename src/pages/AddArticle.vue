<script setup lang="ts">
import { ref, onMounted, inject } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { invokeParse } from '@/composables/useTauri'
import { readClipboard } from '@/composables/useClipboard'
import type { Article, AlertContext } from '@/types'
import HomeButton from '@/components/HomeButton.vue'
import SettingsButton from '@/components/SettingsButton.vue'
import { Fab } from '@/layouts'

const router = useRouter()
const route = useRoute()

const urlInput = ref('')
const progressBar = ref(false)

const { updateAlertContext } = inject<AlertContext>('alert') || {}

async function pasteFromClipboard() {
	const text = await readClipboard()
	if (text) {
		urlInput.value = text
	}
}

async function onSubmit(e: Event) {
	e.preventDefault()
  
	progressBar.value = true
  
	try {
		const article = await invokeParse<Article>('add_article', { url: urlInput.value })
		router.replace({ name: 'article', params: { id: article.id } })
	} catch (err) {
		updateAlertContext?.('error', `Failed to add article: ${err}`)
		progressBar.value = false
	}
}

onMounted(() => {
	urlInput.value = decodeURIComponent(route.query?.shared || '');
})
</script>

<template>
  <article>
    <div v-if="progressBar">
      <blockquote>{{ urlInput }}</blockquote>
      <article aria-busy="true"></article>
    </div>

    <form v-else class="container page" @submit="onSubmit">
      <input
        type="url"
        v-model="urlInput"
        placeholder="https://example.com/article"
        required
      />
      <div role="group">
        <button class="outline" type="button" @click="pasteFromClipboard">
          <i class="ti ti-clipboard">&#x100cc;</i>
        </button>
        <div role="group">
          <button type="submit">
            <i class="ti ti-device-floppy">&#xeb62;</i>
          </button>
        </div>
      </div>
    </form>

    <Fab>
      <HomeButton />
      <SettingsButton />
    </Fab>
  </article>
</template>
