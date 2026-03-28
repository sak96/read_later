<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { getVoices } from '@/composables/useSpeak'

interface Voice {
  voice: SpeechSynthesisVoice
  label: string
  id: string
}

const languages = ref<Voice[]>([])
const selectedIndex = ref<number | null>(null)

async function loadVoices() {
  const voices = await getVoices()
  languages.value = voices.map(v => ({
    voice: v,
    label: v.name,
    id: v.voiceURI
  }))
}

async function onLanguageChange(event: Event) {
  const target = event.target as HTMLSelectElement
  const index = parseInt(target.value)
  selectedIndex.value = index
  // Set voice in settings - would need implementation
}

onMounted(() => {
  loadVoices()
})
</script>

<template>
  <template v-if="languages.length > 0">
    <select role="button" @change="onLanguageChange" class="ti">
      <option :selected="selectedIndex === null" disabled>
        <i class="ti ti-language">&#xebbe;</i>
      </option>
      <option v-for="(lang, idx) in languages" :key="lang.id" :value="idx">
        {{ lang.label }}
      </option>
    </select>
  </template>
</template>
