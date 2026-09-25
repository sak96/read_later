<script setup lang="ts">
import { ref } from 'vue'
import { BookHeadphones, ArrowLeft, ArrowRight, Globe, Trash2, Home, Pause } from 'lucide-vue-next'
import ReaderSettingIcon from './ReaderSettingIcon.vue'
import ListenResetIcon from './ListenResetIcon.vue'

defineProps<{
  showTutorial: boolean
}>()

const emit = defineEmits<{
  dismiss: []
}>()

const tutorialStage = ref(0)

function goNext() {
  tutorialStage.value += 1
}

function goBack() {
  tutorialStage.value = Math.max(0, tutorialStage.value - 1)
}

function dismiss() {
  emit('dismiss')
}
</script>

<template>
  <dialog
    v-if="showTutorial"
    class="tutorial"
    open
  >
    <article>
      <template v-if="tutorialStage === 0">
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
          <ReaderSettingIcon />
          <small data-i18n="tutorial_reader_settings" />
        </div>
        <div>
          <Home />
          <small data-i18n="tutorial_home" />
        </div>
        <hr>
        <div>
          <small data-i18n="tutorial_scroll" />
        </div>
      </template>
      <template v-else>
        <div>
          <Globe />
          <small data-i18n="tutorial_browser" />
        </div>
        <div>
          <Trash2 />
          <small data-i18n="tutorial_delete" />
        </div>
        <hr>
        <div>
          <small data-i18n="tutorial_fetcher_settings" />
          <ul>
            <li data-i18n="fetcher_html" />
            <li data-i18n="fetcher_html_js" />
            <li data-i18n="fetcher_html_js_auth" />
          </ul>
        </div>
        <hr>
        <div>
          <small data-i18n="tutorial_enable_hint" />
        </div>
      </template>
      <footer>
        <button
          v-if="tutorialStage > 0"
          class="secondary"
          @click="goBack"
        >
          <ArrowLeft />
          <small data-i18n="tutorial_back" />
        </button>
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
