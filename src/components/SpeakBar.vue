<script setup lang="ts">
import { ref, watch, inject, onMounted, onUnmounted } from 'vue'
import { onSpeechEvent, speak, stop, getVoices, Voice } from 'tauri-plugin-tts-api'
import { SpeechEvent } from 'tauri-plugin-tts-api'
import { type UnlistenFn } from '@tauri-apps/api/event'
import type { AlertContext } from '../types'
import SpeakRate from './SpeakRate.vue'
import { loadTtsSetting } from '../composables/useTTS'
const { updateAlertContext } = inject<AlertContext>('alert') || {}

const props = defineProps<{
  divRef: HTMLElement
}>()

type ViewMode = 'view' | 'reader'

const checkpoint = ref(0)
const mode = ref<ViewMode>('view')
const rate = ref(1.0)
const ttsEnabled = ref(true)
const languages = ref<Voice[]>([])
const selectedIndex = ref<number | null>(null)
const voiceId = ref<string | null>(null)
const speechSuccessHandler = ref<UnlistenFn | null>()
const speechErrorHandler = ref<UnlistenFn | null>()
const speechInterruptedHandler = ref<UnlistenFn | null>()

async function loadVoices() {
  languages.value = await getVoices()
}

function loadCurrentPara(newId: number) {
  const paraId = `.tts_para_${newId}`
  const para = props.divRef.querySelector(paraId)
  para?.classList.add('current_para')
  props.divRef.querySelectorAll('.current_para').forEach(
    (el) => {
      if (!el.classList.contains(paraId.slice(1))) {
        el.classList.remove('current_para')
      }
    },
  )
}

function loadModeClass(newMode: ViewMode) {
  if (newMode === 'reader') {
    runReader()
    props.divRef?.classList.remove('view')
    props.divRef?.classList.add('reader')
  }
  else {
    props.divRef?.classList.remove('reader')
    props.divRef?.classList.add('view')
  }
}

async function onLanguageChange(event: Event) {
  const target = event.target as HTMLSelectElement
  const index = parseInt(target.value)
  const voice = index !== null ? languages.value[index] : null
  voiceId.value = voice?.id ?? null
}

function switchMode() {
  if (mode.value === 'reader') {
    stop()
    scrollToTop()
    mode.value = 'view'
  }
  else {
    checkpoint.value = findVisibleParaId()
    mode.value = 'reader'
  }
}

function findVisibleParaId(): number {
  const paras = props.divRef.querySelectorAll('[class^="tts_para_"]')
  for (let i = 0; i < paras.length; i++) {
    const paraId = `.tts_para_${i}`
    const para = props.divRef.querySelector(paraId)
    if (para) {
      const rect = para.getBoundingClientRect()
      if (rect.top >= 0 && rect.top < window.innerHeight / 2) {
        props.divRef.querySelectorAll('.current_para').forEach(el => el.classList.remove('current_para'))
        para.classList.add('current_para')
        return i
      }
    }
  }
  return 0
}

function scrollToTop() {
  const para = props.divRef.querySelector('.current_para')
  para?.scrollIntoView({ behavior: 'smooth', block: 'start' })
}

function scrollToCenter() {
  const para = props.divRef.querySelector('.current_para')
  para?.scrollIntoView({ behavior: 'smooth', block: 'center' })
}

function extractParaText(): string | null {
  const para = props.divRef.querySelector('.current_para')
  return para?.textContent || null
}

function handleSpeechSuccess() {
  if (mode.value === 'reader') {
    // read the next tts para
    checkpoint.value++
    setTimeout(runReader, 0)
  }
}

function handleSpeechError(speechEvent: SpeechEvent) {
  const err = speechEvent.reason || speechEvent.error || 'unknown error'
  updateAlertContext?.('error', `Failed to speak: ${err}`)
  mode.value = 'view'
}

async function runReader() {
  if (mode.value !== 'reader') return
  const paraText = extractParaText()
  if (paraText !== null) {
    const text = paraText.split(' ').filter(Boolean).join(' ')
    scrollToCenter()
    await speak({
      text,
      rate: rate.value,
      language: '',
      voiceId: voiceId.value,
      pitch: 1,
      volume: 1,
      queueMode: 'flush',
    })
  }
  else {
    mode.value = 'view'
    stop()
    return
  }
}

watch(mode, loadModeClass)

watch(checkpoint, loadCurrentPara)

onMounted(async () => {
  ttsEnabled.value = await loadTtsSetting()
  loadVoices()
  await new Promise(resolve => setTimeout(resolve, 1000))
  speechSuccessHandler.value = await onSpeechEvent('speech:finish', handleSpeechSuccess)
  speechErrorHandler.value = await onSpeechEvent('speech:error', handleSpeechError)
  speechInterruptedHandler.value = await onSpeechEvent('speech:interrupted', handleSpeechError)
  loadModeClass(mode.value)
  loadCurrentPara(0)
})

onUnmounted(() => {
  speechSuccessHandler.value?.()
  speechErrorHandler.value?.()
  speechInterruptedHandler.value?.()
  stop()
})

defineExpose({
  mode,
  checkpoint,
  switchMode,
})
</script>

<template>
  <template v-if="ttsEnabled">
    <fieldset
      v-if="mode === 'view'"
      role="group"
    >
      <button @click="switchMode">
        <i class="ti ti-volume">&#xeb51;</i>
      </button>
      <button @click="scrollToTop">
        <i class="ti ti-arrow-back">&#xea0c;</i>
      </button>
      <template v-if="languages.length > 0">
        <select
          role="button"
          class="ti"
          style="text-align-last: center;"
          @change="onLanguageChange"
        >
          <option
            :selected="selectedIndex === null"
            disabled
          >
            &#127757;
          </option>
          <option
            v-for="(lang, idx) in languages"
            :key="lang.id"
            :value="idx"
          >
            {{ lang.name }}
          </option>
        </select>
      </template>
    </fieldset>
    <fieldset
      v-else
      role="group"
    >
      <button @click="switchMode">
        <i class="ti ti-player-pause">&#xf690;</i>
      </button>
      <SpeakRate
        :model-value="rate"
        @update:model-value="rate = $event"
      />
    </fieldset>
  </template>
</template>
<style>
 .reader .current_para {
     background-color: var(--pico-mark-background-color) !important;
     color: var(--pico-mark-color) !important;
 }
 .view .current_para {
     border: var(--pico-border-width) solid var(--pico-primary-hover);
     border-radius: var(--pico-border-radius);
 }
</style>
