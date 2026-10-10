<script setup lang="ts">
import { ref, onMounted, inject, type Ref } from 'vue'
import { getSetting, setSetting } from '../composables/useSettings'
import { getVersion } from '@tauri-apps/api/app'
import { openUrl } from '@tauri-apps/plugin-opener'
import HomeButton from '../components/HomeButton.vue'
import type { AlertContext } from '../types'
import DataTransferButton from '../components/DataTransferButton.vue'
import FetcherMode from '../components/FetcherMode.vue'
import ReaderSettingIcon from '../components/ReaderSettingIcon.vue'
import SpeakRate from '../components/SpeakRate.vue'
import LanguageSelect from '../components/LanguageSelect.vue'
import FontScale from '../components/FontScale.vue'
import LocaleBar from '../components/LocaleBar.vue'
import WebdavSettings from '../components/WebdavSettings.vue'
import PronunciationSettings from '../components/PronunciationSettings.vue'
import { Fab } from '../layouts'
import { loadTtsSetting } from '../composables/useTTS'
import { invokeParseLogError } from '../composables/useTauri'
import { TUTORIAL_SHOWN, TTS_ENABLED, THEME, OPEN_COUNT } from '../constants'
import { MonitorCog, Sun, Moon, CodeXml, Bug, CloudSync, Info, Settings } from 'lucide-vue-next'

type Theme = 'light' | 'dark' | 'system'

const themeContext = inject<{ mode: Ref<Theme>, setMode: (mode: Theme) => void }>('theme')
const alertContext = inject<AlertContext | null>('alert')

const appVersion = ref('N/A')
const ttsEnabled = ref(true)
const tutorialEnabled = ref(true)
const feedbackEnabled = ref(true)
const articleCount = ref(0)
const themes = [
  { value: 'light' as Theme, icon: Sun },
  { value: 'dark' as Theme, icon: Moon },
  { value: 'system' as Theme, icon: MonitorCog },
]

const infos = [
  { url: 'https://github.com/sak96/read_later', icon: CodeXml },
  { url: 'https://github.com/sak96/read_later/issues', icon: Bug },
]

async function onThemeChange(newTheme: Theme) {
  if (themeContext) {
    themeContext.setMode(newTheme)
  }
  await setSetting(THEME, newTheme)
}

async function onTtsToggle() {
  const newState = !ttsEnabled.value
  ttsEnabled.value = newState
  await setSetting(TTS_ENABLED, newState.toString())
}

async function onTutorialToggle() {
  const newState = !tutorialEnabled.value
  tutorialEnabled.value = newState
  await setSetting(TUTORIAL_SHOWN, newState ? 'false' : 'true')
}

async function onFeedbackToggle() {
  const newState = !feedbackEnabled.value
  feedbackEnabled.value = newState
  await setSetting(OPEN_COUNT, newState ? '1' : '0')
}

async function onFullErrorToggle() {
  await alertContext?.setShowErrorTrace(!alertContext.showErrorTrace.value)
}

onMounted(async () => {
  articleCount.value = await invokeParseLogError<number>('get_article_count') || 0
  appVersion.value = await getVersion()
  ttsEnabled.value = await loadTtsSetting()
  const tutorialSetting = await getSetting(TUTORIAL_SHOWN)
  tutorialEnabled.value = tutorialSetting !== 'true'
  const feedbackSetting = await getSetting(OPEN_COUNT)
  feedbackEnabled.value = feedbackSetting == null ? true : feedbackSetting !== '0'
})
</script>

<template>
  <main class="container page">
    <article>
      <form>
        <fieldset>
          <h4>
            <Settings style="margin-right: 1em" />
            <span data-i18n="app_settings" />
          </h4>
          <label data-i18n="theme" />
          <div role="group">
            <button
              v-for="themeOption in themes"
              :key="themeOption.value"
              :class="themeContext?.mode?.value === themeOption.value ? 'primary' : 'outline'"
              @click="onThemeChange(themeOption.value)"
            >
              <component :is="themeOption.icon" />
            </button>
          </div>
          <LocaleBar />
          <table>
            <tbody>
              <tr>
                <th data-i18n="tutorial_enabled" />
                <td>
                  <input
                    name="tutorial"
                    type="checkbox"
                    role="switch"
                    :checked="tutorialEnabled"
                    @change="onTutorialToggle"
                  >
                </td>
              </tr>
              <tr>
                <th data-i18n="feedback_enabled" />
                <td>
                  <input
                    name="feedback"
                    type="checkbox"
                    role="switch"
                    :checked="feedbackEnabled"
                    @change="onFeedbackToggle"
                  >
                </td>
              </tr>
              <tr>
                <th data-i18n="full_error" />
                <td>
                  <input
                    name="full_error"
                    type="checkbox"
                    role="switch"
                    :checked="alertContext?.showErrorTrace.value ?? false"
                    @change="onFullErrorToggle"
                  >
                </td>
              </tr>
            </tbody>
          </table>
        </fieldset>
        <hr>
        <fieldset>
          <h4>
            <ReaderSettingIcon style="margin-right: 1em" />
            <span data-i18n="tutorial_reader_settings" />
          </h4>
          <table>
            <tbody>
              <tr>
                <th data-i18n="speech_enabled" />
                <td>
                  <input
                    name="terms"
                    type="checkbox"
                    role="switch"
                    :checked="ttsEnabled"
                    @change="onTtsToggle"
                  >
                </td>
              </tr>
            </tbody>
          </table>
          <SpeakRate
            :model-value="1"
            @update:model-value="() => {}"
          />
          <FontScale :target="null" />
          <LanguageSelect />
          <FetcherMode />
          <PronunciationSettings />
        </fieldset>
        <hr>

        <fieldset>
          <h4>
            <CloudSync style="margin-right: 1em" />
            <span data-i18n="data_management" />
          </h4>
          <WebdavSettings />
          <hr>
          <label>
            <span
              data-i18n="restore"
              style="margin-right: 1em"
            />
            <mark>({{ articleCount.toString() }})</mark>
          </label>
          <div role="group">
            <DataTransferButton type="import" />
            <DataTransferButton type="export" />
          </div>
        </fieldset>
        <hr>

        <fieldset>
          <label>
            <h4>
              <Info style="margin-right: 1em" />
              <span data-i18n="about" />
            </h4>
            <div role="group">
              <button
                v-for="info in infos"
                :key="info.url"
                type="button"
                class="outline"
                @click="openUrl(info.url)"
              >
                <component :is="info.icon" />
              </button>
            </div>
          </label>
        </fieldset>

        <table>
          <tbody>
            <tr>
              <th data-i18n="version" />
              <td>{{ appVersion }}</td>
            </tr>
            <tr>
              <th data-i18n="privacy" />
              <td>
                <a
                  class="outline"
                  data-i18n="link"
                  @click="openUrl('https://github.com/sak96/read_later/blob/master/PRIVACY_POLICY.md')"
                />
              </td>
            </tr>
          </tbody>
        </table>
      </form>

      <Fab>
        <HomeButton />
      </Fab>
    </article>
  </main>
</template>
