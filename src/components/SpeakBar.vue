<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted } from 'vue'
import { getSetting, setSetting } from '@/composables/useSettings'
import { speakText, stopSpeaking } from '@/composables/useSpeak'
import SpeakRate from './SpeakRate.vue'
import LanguageSelection from './Language.vue'

const props = defineProps<{
  divRef: HTMLElement
}>()

type ViewMode = 'view' | 'reader'

const checkpoint = ref(0)
const mode = ref<ViewMode>('view')
const rate = ref(1.0)
const ttsEnabled = ref(true)

async function loadTtsSetting() {
  const value = await getSetting('tts')
  if (value !== null) {
    ttsEnabled.value = value === 'true'
  } else {
    const isAndroid = await checkAndroid()
    await setSetting('tts', isAndroid.toString())
    ttsEnabled.value = isAndroid
  }
}

async function checkAndroid(): Promise<boolean> {
  try {
    const osType = await import('@tauri-apps/api/core').then(m => 
      m.invoke<string>('plugin:os|type')
    )
    return osType === 'android'
  } catch {
    return false
  }
}

function switchMode() {
  if (mode.value === 'reader') {
    stopSpeaking()
    scrollToTop()
    mode.value = 'view'
  } else {
    checkpoint.value = findVisibleParaId()
    mode.value = 'reader'
  }
}

function findVisibleParaId(): number {
  const paras = props.divRef.querySelectorAll('[id^="para_"]')
  for (let i = 0; i < paras.length; i++) {
    const rect = paras[i].getBoundingClientRect()
    if (rect.top >= 0 && rect.top < window.innerHeight / 2) {
      return i
    }
  }
  return 0
}

function scrollToTop() {
  const para = props.divRef.querySelector(`[id="para_${checkpoint.value}"]`)
  if (para) {
    para.scrollIntoView({ behavior: 'smooth', block: 'start' })
  }
}

function scrollToCenter() {
  const para = props.divRef.querySelector(`[id="para_${checkpoint.value}"]`)
  if (para) {
    para.scrollIntoView({ behavior: 'smooth', block: 'center' })
  }
}

function extractParaText(id: number): string | null {
  const para = props.divRef.querySelector(`[id="para_${id}"]`)
  return para?.textContent || null
}

async function runReader() {
  if (mode.value !== 'reader') return
  
  const text = extractParaText(checkpoint.value)
  if (text) {
    scrollToCenter()
    await speakText({ text, rate: rate.value })
    checkpoint.value++
    if (mode.value === 'reader') {
      setTimeout(runReader, 100)
    }
  } else {
    mode.value = 'view'
    stopSpeaking()
  }
}

watch(mode, (newMode) => {
  if (newMode === 'reader') {
    runReader()
  }
})

onMounted(() => {
  loadTtsSetting()
})

onUnmounted(() => {
  stopSpeaking()
})

defineExpose({
  mode,
  checkpoint,
  switchMode,
})
</script>

<template>
  <template v-if="ttsEnabled">
    <fieldset v-if="mode === 'view'" role="group">
      <button @click="switchMode">
        <i class="ti ti-volume">&#xeb51;</i>
      </button>
      <button @click="scrollToTop"><i class="ti ti-arrow-back">&#xea0c;</i></button>
      <LanguageSelection />
    </fieldset>
    <fieldset v-else role="group">
      <button @click="switchMode"><i class="ti ti-player-pause">&#xf690;</i></button>
      <SpeakRate :modelValue="rate" @update:modelValue="rate = $event" />
    </fieldset>
  </template>
</template>
