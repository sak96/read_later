<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { getSetting, setSetting } from '../composables/useSettings'
import { BookHeadphones, ChevronLeft, ArrowRight, ChevronRight, Globe, Trash2, Home, Pause } from 'lucide-vue-next'
import SpeechSettingIcon from './SpeechSettingIcon.vue'
import ListenResetIcon from './ListenResetIcon.vue'

defineProps<{
  foldBar: boolean
}>()

const emit = defineEmits<{
  dismiss: []
  updateFoldBar: []
}>()

const shownTutorial = ref(false)
const tutorialStage = ref(0)

onMounted(async () => {
  const tutorialShown = await getSetting('tutorial_speak_bar_shown')
  if (tutorialShown !== 'true') {
    shownTutorial.value = true
  }
})

async function dismiss() {
  await setSetting('tutorial_speak_bar_shown', 'true')
  shownTutorial.value = false
  emit('dismiss')
}

async function goNext() {
  tutorialStage.value += 1
  if (tutorialStage.value > 2) {
    await dismiss()
  }
  emit('updateFoldBar')
}
</script>

<template>
  <dialog
    v-if="shownTutorial"
    class="tutorial"
    open
  >
    <article>
      <template v-if="foldBar">
        <div>
          <ListenResetIcon />
          <small data-i18n="tutorial_undo" />
        </div>
        <div>
          <BookHeadphones />
          <small data-i18n="tutorial_speak" />
        </div>
        <div>
          <Pause />
          <small data-i18n="tutorial_pause" />
        </div>
        <div>
          <ChevronLeft />
          <small data-i18n="tutorial_expand" />
        </div>
        <hr>
        <div>
          <small data-i18n="tutorial_scroll" />
        </div>
      </template>
      <template v-else>
        <div>
          <ChevronRight />
          <small data-i18n="tutorial_collapse" />
        </div>
        <div>
          <Globe />
          <small data-i18n="tutorial_browser" />
        </div>
        <div>
          <Trash2 />
          <small data-i18n="tutorial_delete" />
        </div>
        <div>
          <Home />
          <small data-i18n="tutorial_home" />
        </div>
        <div>
          <SpeechSettingIcon />
          <small data-i18n="tutorial_speech_settings" />
        </div>
      </template>
      <footer>
        <button
          v-if="tutorialStage < 1"
          class="tutorial-next"
          @click="goNext"
        >
          <ArrowRight />
          <small data-i18n="tutorial_next" />
        </button>
        <button
          class="secondary"
          data-i18n="tutorial_dismiss"
          @click="dismiss"
        />
      </footer>
    </article>
  </dialog>
</template>

<style>
.tutorial::backdrop,
.tutorial::before,
.tutorial::after,
.tutorial {
  backdrop-filter: none !important;
  -webkit-backdrop-filter: none !important;
  background: rgba(0, 0, 0, 0.5);
}
</style>
