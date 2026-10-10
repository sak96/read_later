<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { openUrl } from '@tauri-apps/plugin-opener'
import { getSetting, setSetting } from '../composables/useSettings'
import { OPEN_COUNT } from '../constants'
import { Star, Play, MessageSquareHeart } from 'lucide-vue-next'

const PLAYSTORE_URL = 'https://play.google.com/store/apps/details?id=io.github.sak.read.it.later'
const GITHUB_URL = 'https://github.com/sak96/read_later'
const MAX_OPEN_COUNT = 500

const show = ref(false)

function closeDialog() {
  show.value = false
}

onMounted(async () => {
  const raw = await getSetting(OPEN_COUNT)

  // disabled
  if (raw === '0') return

  const previous = parseInt(raw || '0', 10) || 0
  let count = previous + 1

  // overflow guard: wrap back to 1 (never 0 — 0 means disabled)
  if (count > MAX_OPEN_COUNT) {
    count = 1
  }

  await setSetting(OPEN_COUNT, String(count))

  if (count % MAX_OPEN_COUNT === MAX_OPEN_COUNT / 2) {
    show.value = true
  }
})
</script>

<template>
  <dialog :open="show">
    <article>
      <header>
        <MessageSquareHeart style="margin-right: 1em" />
        <span data-i18n="feedback_title" />
        <button
          type="button"
          aria-label="Close"
          rel="prev"
          @click="closeDialog"
        />
      </header>
      <span data-i18n="feedback_message" />
      <footer>
        <div role="group">
          <button
            type="button"
            class="outline"
            @click="openUrl(PLAYSTORE_URL)"
          >
            <Play />
            <span data-i18n="feedback_playstore" />
          </button>
          <button
            type="button"
            class="outline"
            @click="openUrl(GITHUB_URL)"
          >
            <Star />
            <span data-i18n="feedback_github" />
          </button>
        </div>
      </footer>
    </article>
  </dialog>
</template>
