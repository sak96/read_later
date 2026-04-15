<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { getVoices, Voice } from 'tauri-plugin-tts-api'
import { invokeNoParseLogError } from '../composables/useTauri'

const languages = ref<Voice[]>([])
const voiceId = ref<string | null>(null)

async function loadVoices() {
  try {
    languages.value = await getVoices()
  }
  catch (e) {
    console.error(`Failed to load voices: ${e}`)
  }
}

async function onLanguageChange(event: Event) {
  const target = event.target as HTMLSelectElement
  const index = parseInt(target.value)
  const voice = index !== null ? languages.value[index] : null
  voiceId.value = voice?.id ?? null
  await invokeNoParseLogError('set_voice_id', { voiceId: voiceId.value })
}

onMounted(async () => {
  await loadVoices()
})
</script>

<template>
  <template v-if="languages.length > 0">
    <select
      role="button"
      class="ti"
      style="text-align-last: center;"
      @change="onLanguageChange"
    >
      <option
        v-for="(lang, idx) in languages"
        :key="lang.id"
        :value="idx"
      >
        {{ lang.name }}
      </option>
    </select>
  </template>
</template>
