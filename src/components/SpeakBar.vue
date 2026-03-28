<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted } from 'vue'
import { getSetting, setSetting } from '@/composables/useSettings'
import { speak, stop, isSpeaking, getVoices, Voice } from "tauri-plugin-tts-api";
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
const languages = ref<Voice[]>([])
const selectedIndex = ref<number | null>(null)

async function loadTtsSetting() {
	const value = await getSetting('tts')
	ttsEnabled.value = value === 'true'
}


async function loadVoices() {
	languages.value = await getVoices()
}


async function onLanguageChange(event: Event) {
	const target = event.target as HTMLSelectElement
	const index = parseInt(target.value)
	selectedIndex.value = index
}

function switchMode() {
	if (mode.value === 'reader') {
		stop()
		scrollToTop()
		mode.value = 'view'
	} else {
		checkpoint.value = findVisibleParaId()
		mode.value = 'reader'
	}
}

function findVisibleParaId(): number {
	const paras = props.divRef.querySelectorAll('[class^="tts_para_"]')
	for (let i = 0; i < paras.length; i++) {
		const rect = paras[i].getBoundingClientRect()
		if (rect.top >= 0 && rect.top < window.innerHeight / 2) {
			return i
		}
	}
	return 0
}

function scrollToTop() {
	const para = props.divRef.querySelector(".current_para")
	para?.scrollIntoView({ behavior: 'smooth', block: 'start' })
}

function scrollToCenter() {
	const para = props.divRef.querySelector(".current_para")
	para?.scrollIntoView({ behavior: 'smooth', block: 'center' })
}

function extractParaText(): string | null {
	const para = props.divRef.querySelector(".current_para")
	return para?.textContent || null
}

async function runReader() {
	if (mode.value !== 'reader') return
	const paraText = extractParaText()
	if (paraText !== null) {
		const text = paraText.replace(/\s+/g, ' ').trim()
 	   scrollToCenter()
		try {
			await speak({ text, rate: rate.value});
		} catch (err) { console.error(err) }
		while (await isSpeaking()) {
			await new Promise(resolve => setTimeout(resolve, 1000));
		}
		checkpoint.value++
		if (mode.value === 'reader') {
		   setTimeout(runReader, 1000)
		}
	} else {
		mode.value = 'view'
		stop()
		return
	}
}

watch(mode, (newMode) => {
	if (newMode === 'reader') {
		runReader()
		props.divRef?.classList.remove('view')
		props.divRef?.classList.add('reader')
	} else {
		props.divRef?.classList.remove('reader')
		props.divRef?.classList.add('view')
	}
})

watch(checkpoint, (newId) => {
	const paraId = `.tts_para_${newId}`
	const para = props.divRef.querySelector(paraId)
	para?.classList.add('current_para');
	props.divRef.querySelectorAll('.current_para').forEach(
		el => {
			if (!el.classList.contains(paraId.slice(1))) {
				el.classList.remove("current_para")
			}
		}
	)
	const test = props.divRef.querySelector('.current_para');
})

onMounted(() => {
	loadTtsSetting()
	loadVoices()
})

onUnmounted(() => {
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
    <fieldset v-if="mode === 'view'" role="group">
      <button @click="switchMode">
        <i class="ti ti-volume">&#xeb51;</i>
      </button>
      <button @click="scrollToTop"><i class="ti ti-arrow-back">&#xea0c;</i></button>
      <template v-if="languages.length > 0">
        <select role="button" @change="onLanguageChange" class="ti">
          <option :selected="selectedIndex === null" disabled>
            <i class="ti ti-language">&#xebbe;</i>
          </option>
          <option v-for="(lang, idx) in languages" :key="lang.id" :value="idx">
            {{ lang.name }}
          </option>
        </select>
      </template>
    </fieldset>
    <fieldset v-else role="group">
      <button @click="switchMode"><i class="ti ti-player-pause">&#xf690;</i></button>
      <SpeakRate :modelValue="rate" @update:modelValue="rate = $event" />
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
