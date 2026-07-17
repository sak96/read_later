<script setup lang="ts">
import { ref, onMounted, inject, type Ref } from 'vue'
import { getSetting, setSetting } from '../composables/useSettings'
import { getVersion } from '@tauri-apps/api/app'
import { openUrl } from '@tauri-apps/plugin-opener'
import HomeButton from '../components/HomeButton.vue'
import DataTransferButton from '../components/DataTransferButton.vue'
import SpeakRate from '../components/SpeakRate.vue'
import LanguageSelect from '../components/LanguageSelect.vue'
import FontScale from '../components/FontScale.vue'
import LocaleBar from '../components/LocaleBar.vue'
import WebdavSettings from '../components/WebdavSettings.vue'
import { Fab } from '../layouts'
import { loadTtsSetting } from '../composables/useTTS'
import { invokeParseLogError } from '../composables/useTauri'
import { FETCHER_MODE, TUTORIAL_SHOWN, TTS_ENABLED, THEME } from '../constants'
import { MonitorCog, Sun, Moon, CodeXml, Bug, Palette, Speech, Archive, Info } from 'lucide-vue-next'

type Theme = 'light' | 'dark' | 'system'

const themeContext = inject<{ mode: Ref<Theme>, setMode: (mode: Theme) => void }>('theme')

const appVersion = ref('N/A')
const ttsEnabled = ref(true)
const tutorialEnabled = ref(true)
const fetcherMode = ref('html')
const articleCount = ref(0)

const fetcherModes = [
  { value: 'html', label: 'fetcher_html' },
  { value: 'html_js', label: 'fetcher_html_js' },
  { value: 'html_js_auth', label: 'fetcher_html_js_auth' },
]

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

async function onFetcherModeChange(event: Event) {
  const target = event.target as HTMLSelectElement
  const newMode = target.value
  fetcherMode.value = newMode
  await setSetting(FETCHER_MODE, newMode)
}

onMounted(async () => {
  articleCount.value = await invokeParseLogError<number>('get_article_count') || 0
  appVersion.value = await getVersion()
  ttsEnabled.value = await loadTtsSetting()
  const tutorialSetting = await getSetting(TUTORIAL_SHOWN)
  tutorialEnabled.value = tutorialSetting !== 'true'
  const fetcherModeSetting = await getSetting(FETCHER_MODE)
  fetcherMode.value = fetcherModeSetting || 'html'
})
</script>

<template>
  <main class="container page">
    <article>
      <form>
        <fieldset>
          <h4>
            <Palette style="margin-right: 1em" />
            <span data-i18n="theme" />
          </h4>
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
        </fieldset>
        <hr>
        <LocaleBar />
        <hr>
        <fieldset>
          <h4>
            <Speech style="margin-right: 1em" />
            <span data-i18n="speech" />
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
                <th data-i18n="fetcher_mode" />
                <td>
                  <select
                    style="text-align-last: center;"
                    @change="onFetcherModeChange"
                  >
                    <option
                      v-for="mode in fetcherModes"
                      :key="mode.value"
                      :selected="fetcherMode === mode.value"
                      :value="mode.value"
                      :data-i18n="mode.label"
                    />
                  </select>
                </td>
              </tr>
            </tbody>
          </table>
          <SpeakRate
            :model-value="1"
            @update:model-value="() => {}"
          />
          <LanguageSelect />
          <FontScale :target="null" />
        </fieldset>
        <hr>

        <WebdavSettings />
        <hr>

        <fieldset>
          <label>
            <h4>
              <Archive style="margin-right: 1em" />
              <span
                data-i18n="restore"
                style="margin-right: 1em"
              />
              <mark>({{ articleCount.toString() }})</mark>
            </h4>
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
