<script setup lang="ts">
import { ref, onMounted, inject } from 'vue'
import { setSetting } from '../composables/useSettings'
import { getVersion } from '@tauri-apps/api/app'
import { openUrl } from '@tauri-apps/plugin-opener'
import HomeButton from '../components/HomeButton.vue'
import ImportButton from '../components/ImportButton.vue'
import ExportButton from '../components/ExportButton.vue'
import SpeakRate from '../components/SpeakRate.vue'
import LocaleBar from '../components/LocaleBar.vue'
import { Fab } from '../layouts'
import { loadTtsSetting } from '../composables/useTTS'
import { invokeParseLogError } from '../composables/useTauri'
import { IconDeviceDesktopCog, IconInfoCircle, IconRestore, IconVolume, Icon, IconMoon, IconPalette, IconSun, IconBug, IconBrandGithub } from '@tabler/icons-vue'

type Theme = 'light' | 'dark' | 'system'

const themeContext = inject<{ mode: Theme, setMode: (mode: Theme) => void }>('theme')

const appVersion = ref('N/A')
const ttsEnabled = ref(true)
const articleCount = ref(0)

const themes: Array<{ value: Theme, icon: Icon }> = [
  { value: 'light', icon: IconSun },
  { value: 'dark', icon: IconMoon },
  { value: 'system', icon: IconDeviceDesktopCog },
]

const infos = [
  { url: 'https://github.com/sak96/read_later', icon: IconBrandGithub },
  { url: 'https://github.com/sak96/read_later/issues', icon: IconBug },
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

onMounted(async () => {
  articleCount.value = await invokeParseLogError<number>('get_article_count') || 0
  appVersion.value = await getVersion()
  ttsEnabled.value = await loadTtsSetting()
})
</script>

<template>
  <main class="container page">
    <article>
      <form>
        <fieldset>
          <h4>
            <IconPalette style="margin-right: 1em" />
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
              <IconVolume style="margin-right: 1em" />
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
          <div
            role="group"
            style="flex: 1;"
          >
            <SpeakRate
              :model-value="1"
              @update:model-value="() => {}"
            />
          </div>
        </fieldset>
        <hr>

        <fieldset>
          <label>
            <h4>
              <IconRestore style="margin-right: 1em" />
              <span
                data-i18n="restore"
                style="margin-right: 1em"
              />
              <mark>({{ articleCount.toString() }})</mark>
              <div role="group">
                <ImportButton />
                <ExportButton />
              </div>
            </h4></label>
        </fieldset>
        <hr>

        <fieldset>
          <label>
            <h4>
              <IconInfoCircle style="margin-right: 1em" />
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
