<script setup lang="ts">
import { ref, watch, inject, onMounted, onUnmounted, nextTick } from 'vue'
import type { PluginListener } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { invokeNoParseLogError, invokeParseLogError } from '../composables/useTauri'
import type { AlertContext } from '../types'
import SpeakRate from './SpeakRate.vue'
import LanguageSelect from './LanguageSelect.vue'
import SpeechSettingIcon from './SpeechSettingIcon.vue'
import ListenResetIcon from './ListenResetIcon.vue'
import { loadTtsSetting } from '../composables/useTTS'
import { onAction } from '../composables/useMediaSession'
import { platform } from '@tauri-apps/plugin-os'
import { BookHeadphones, Pause, ChevronLeft, ChevronRight } from 'lucide-vue-next'
import Fab from '../layouts/Fab.vue'
import HomeButton from './HomeButton.vue'
import TutorialSpeakBar from './TutorialSpeakBar.vue'
import FontScale from './FontScale.vue'

const alertContext = inject<AlertContext | null>('alert')

const props = defineProps<{
  divRef: HTMLElement
  title?: string
}>()

const foldBar = ref(true)
const showSettings = ref(false)

type ViewMode = 'view' | 'reader'

const mode = ref<ViewMode>('view')
const rate = ref(1.0)
const ttsEnabled = ref(true)
const stateHandler = ref<UnlistenFn | null>()
const notificationListener = ref<PluginListener | null>()
const currentPlatform: string = platform()
const focusUnlistener = ref<UnlistenFn | null>(null)

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

function extractParaText(): string[] {
  const paras = props.divRef.querySelectorAll('[class^="tts_para_"]')
  return Array.from(paras).map(para => para?.textContent?.trim() || '.')
}
async function initReading() {
  await invokeNoParseLogError('init_reading', { rate: rate.value, title: props.title || 'Untitled', paragraphs: extractParaText() })
}

async function loadNotificationHandlers() {
  try {
    const unlisten = await onAction((event) => {
      switch (event.action) {
        case 'play':
          mode.value = 'reader'
          break
        case 'pause':
        case 'stop':
          mode.value = 'view'
          stop()
          break
        default:
          console.error('Unhandled media session action:', event.action)
      }
    })
    notificationListener.value = unlisten
  }
  catch (e) {
    console.error('Failed to register media session action listener:', e)
  }
}

async function loadEventHandlers() {
  try {
    stateHandler.value = await listen<{ position?: number, mode: 'view' | 'reader' }>('speakbar:state-changed', (event) => {
      const { position, mode: newMode } = event.payload
      if (newMode === 'view') {
        mode.value = 'view'
        if (position === undefined) {
          alertContext?.updateAlertContext?.('error', 'Failed to speak')
        }
      }
      else {
        if (position !== undefined) {
          loadCurrentPara(position)
          scrollTo('center')
        }
        mode.value = 'reader'
      }
    })
  }
  catch (e) {
    console.error('Failed to register state-changed listener:', e)
  }
}

async function loadModeClass(newMode: ViewMode) {
  if (newMode === 'reader') {
    props.divRef?.classList.remove('view')
    props.divRef?.classList.add('reader')
    scrollTo('center')
    await invokeNoParseLogError('start_reading', { startPara: findVisibleParaId() })
  }
  else {
    await invokeNoParseLogError('stop_reading')
    props.divRef?.classList.remove('reader')
    props.divRef?.classList.add('view')
    scrollTo('start')
  }
}

async function handleRateUpdate(newRate: number) {
  await invokeNoParseLogError('change_rate', { rate: newRate })
  rate.value = newRate
}

function openSettings() {
  foldBar.value = true
  showSettings.value = true
}

function dismissTutorial() {
  foldBar.value = true
}

function expandForTutorial() {
  foldBar.value = false
}

function scrollTo(block: 'start' | 'center') {
  const para = props.divRef.querySelector('.current_para') as HTMLElement | null
  if (para) {
    const isFirst = para.classList.contains('tts_para_0')

    if (isFirst && block === 'start') {
      // NOTE: window scroll works for desktop props scroll works for android
      window.scrollTo({ top: 0, behavior: 'smooth' })
      props.divRef.scrollTo({ top: 0, behavior: 'smooth' })
    }
    else {
      para.scrollIntoView({ behavior: 'smooth', block })
    }
  }
}

async function switchMode() {
  if (mode.value === 'reader') {
    await nextTick()
    mode.value = 'view'
  }
  else {
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

async function scrollOnFocus() {
  const readState = await invokeParseLogError<{ mode: 'view' | 'reader', position: number }>('get_read_state')
  if (!readState) return
  if (readState.mode !== mode.value) {
    mode.value = readState.mode
  }
  else {
    loadCurrentPara(readState.position)
    scrollTo(readState.mode === 'reader' ? 'center' : 'start')
  }
}

watch(mode, loadModeClass)

onMounted(async () => {
  focusUnlistener.value = await listen(
    currentPlatform === 'android' ? 'tauri://focus' : 'new-intent',
    scrollOnFocus,
  )
  ttsEnabled.value = await loadTtsSetting()
  await nextTick()
  await initReading()
  await loadNotificationHandlers()
  await loadEventHandlers()
  props.divRef?.classList.add('view')
  loadCurrentPara(0)
  scrollTo('start')
})

onUnmounted(async () => {
  await notificationListener.value?.unregister()
  stateHandler.value?.()
  await invokeNoParseLogError('cleanup_reading')
})

</script>

<template>
  <TutorialSpeakBar
    :fold-bar="foldBar"
    @dismiss="dismissTutorial"
    @update-fold-bar="expandForTutorial"
  />
  <Fab :class="{'transparent': foldBar }">
    <template v-if="ttsEnabled">
      <template v-if="mode === 'view'">
        <button
          style="width: fit-content; align-self: flex-end;"
          @click="scrollTo('start')"
        >
          <ListenResetIcon />
        </button>
        <button
          style="width: fit-content; align-self: flex-end;"
          @click="switchMode"
        >
          <BookHeadphones />
        </button>
      </template>
      <button
        v-else
        style="width: fit-content; align-self: flex-end;"
        @click="switchMode"
      >
        <Pause />
      </button>
    </template>
    <HomeButton />
    <div>
      <button
        @click="openSettings"
      >
        <SpeechSettingIcon />
      </button>
    </div>
  </Fab>
  <dialog
    :open="showSettings"
    @close="showSettings = false"
  >
    <article>
      <header>
        <button
          aria-label="Close"
          rel="prev"
          @click="showSettings = false"
        />
        <span data-i18n="speech_settings" />
      </header>
      <template v-if="ttsEnabled">
        <SpeakRate
          :model-value="rate"
          @update:model-value="handleRateUpdate"
        />
        <LanguageSelect />
      </template>
      <FontScale :target="divRef" />
      <div role="group">
        <slot />
      </div>
    </article>
  </dialog>
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
 .transparent * {
   opacity: 0.75;
 }
</style>
