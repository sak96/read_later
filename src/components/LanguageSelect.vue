<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { getVoices, Voice } from 'tauri-plugin-tts-api'
import { invokeNoParseLogError } from '../composables/useTauri'
import { getSetting, setSetting } from '../composables/useSettings'

const SETTING_KEY = 'voice_id'

const languages = ref<Voice[]>([])
const selectedIndex = ref<number>(-1)

async function loadVoices() {
  try {
    languages.value = await getVoices()
    const saved = await getSetting(SETTING_KEY)
    if (saved) {
      const idx = languages.value.findIndex(v => v.id === saved)
      if (idx !== -1) {
        selectedIndex.value = idx
        await invokeNoParseLogError('set_voice_id', { voiceId: saved })
      }
    }
  }
  catch (e) {
    console.error(`Failed to load voices: ${e}`)
  }
}

async function onLanguageChange(event: Event) {
  const target = event.target as HTMLSelectElement
  const index = parseInt(target.value)
  const voice = !isNaN(index) ? languages.value[index] : null
  const id = voice?.id ?? null
  selectedIndex.value = index
  await invokeNoParseLogError('set_voice_id', { voiceId: id })
  if (id) {
    await setSetting(SETTING_KEY, id)
  }
}

onMounted(async () => {
  await loadVoices()
})
</script>

<template>
  <template v-if="languages.length > 0">
    <label data-i18n="speech_voice" />
    <div>
      <select
        :value="selectedIndex >= 0 ? selectedIndex : ''"
        style="text-align-last: center;"
        @change="onLanguageChange"
      >
        <option
          value=""
          disabled
        >
          &#127760;
        </option>
        <option
          v-for="(lang, idx) in languages"
          :key="lang.id"
          :value="idx"
        >
          {{ lang.name }}
        </option>
      </select>
    </div>
  </template>
</template>
