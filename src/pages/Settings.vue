<script setup lang="ts">
import { ref, onMounted, inject } from 'vue'
import { getSetting, setSetting } from '../composables/useSettings'
import { getVersion } from '@tauri-apps/api/app'
import { openUrl } from '@tauri-apps/plugin-opener'
import HomeButton from '../components/HomeButton.vue'
import DataTransferButton from '../components/DataTransferButton.vue'
import SpeakRate from '../components/SpeakRate.vue'
import LanguageSelect from '../components/LanguageSelect.vue'
import LocaleBar from '../components/LocaleBar.vue'
import { Fab } from '../layouts'
import { loadTtsSetting } from '../composables/useTTS'
import { invokeParseLogError } from '../composables/useTauri'
import { MonitorCog, Sun, Moon, CodeXml, Bug, Palette, Speech, Archive, Info } from 'lucide-vue-next'

type Theme = 'light' | 'dark' | 'system'

const themeContext = inject<{ mode: Theme, setMode: (mode: Theme) => void }>('theme')

const appVersion = ref('N/A')
const ttsEnabled = ref(true)
const tutorialEnabled = ref(true)
const iframeFetcherEnabled = ref(false)
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
  await setSetting('theme', newTheme)
}

async function onTtsToggle() {
  const newState = !ttsEnabled.value
  ttsEnabled.value = newState
  await setSetting('tts', newState.toString())
}

async function onTutorialToggle() {
  const newState = !tutorialEnabled.value
  tutorialEnabled.value = newState
  await setSetting('tutorial_speak_bar_shown', newState ? 'false' : 'true')
}

async function onIframeFetcherToggle() {
  const newState = !!!iframeFetcherEnabled.value
  iframeFetcherEnabled.value = newState
  await setSetting('iframe_fetcher', newState.toString())
}

onMounted(async () => {
  articleCount.value = await invokeParseLogError<number>('get_article_count') || 0
  appVersion.value = await getVersion()
  ttsEnabled.value = await loadTtsSetting()
  const tutorialSetting = await getSetting('tutorial_speak_bar_shown')
  tutorialEnabled.value = tutorialSetting !== 'true'
  const iframeFetcherSetting = await getSetting('iframe_fetcher')
  iframeFetcherEnabled.value = iframeFetcherSetting === 'true'
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
              :class="themeContext?.mode === themeOption.value ? 'primary' : 'outline'"
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
          <div role="group">
            <h4>
              <Speech style="margin-right: 1em" />
              <span data-i18n="speech" />
            </h4>
            <div>
              <input
                name="terms"
                type="checkbox"
                role="switch"
                :checked="ttsEnabled"
                @change="onTtsToggle"
              >
            </div>
          </div>
          <div role="group">
            <h4>
              <span data-i18n="tutorial_enabled" />
            </h4>
            <div>
              <input
                name="tutorial"
                type="checkbox"
                role="switch"
                :checked="tutorialEnabled"
                @change="onTutorialToggle"
              >
            </div>
          </div>
          <div role="group">
            <h4>
              <span data-i18n="experimental_fetcher" />
            </h4>
            <div>
              <input
                name="iframeFetcher"
                type="checkbox"
                role="switch"
                :checked="iframeFetcherEnabled"
                @change="onIframeFetcherToggle"
              >
            </div>
          </div>
          <div
            role="group"
            style="flex: 1;"
          >
            <SpeakRate
              :model-value="1"
              @update:model-value="() => {}"
            />
          </div>
          <LanguageSelect />
        </fieldset>
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
