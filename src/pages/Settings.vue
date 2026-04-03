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

type Theme = 'light' | 'dark' | 'system'

const themeContext = inject<{ mode: Theme, setMode: (mode: Theme) => void }>('theme')

const appVersion = ref('N/A')
const ttsEnabled = ref(true)

const themes: Array<{ value: Theme, icon: string, code: string }> = [
  { value: 'light', icon: 'ti-sun', code: '\uf6a9' },
  { value: 'dark', icon: 'ti-moon', code: '\ueaf8' },
  { value: 'system', icon: 'ti-device-desktop-cog', code: '\uf862' },
]

const infos = [
  { url: 'https://github.com/sak96/read_later', icon: 'ti-brand-github', code: '\uec1c' },
  { url: 'https://github.com/sak96/read_later/issues', icon: 'ti-bug', code: '\uea48' },
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
  appVersion.value = await getVersion()
  ttsEnabled.value = await loadTtsSetting()
})
</script>

<template>
  <main class="container page">
    <article>
      <form>
        <fieldset>
          <h2 class="ti ti-palette">
            &#xeb01;&nbsp;<span data-i18n="theme" />
          </h2>
          <div role="group">
            <button
              v-for="themeOption in themes"
              :key="themeOption.value"
              :class="themeContext?.mode === themeOption.value ? 'primary' : 'outline'"
              @click="onThemeChange(themeOption.value)"
            >
              <i class="ti">{{ themeOption.code }}</i>
            </button>
          </div>
        </fieldset>
        <hr>
        <LocaleBar />
        <hr>
        <fieldset>
          <div role="group">
            <h2 class="ti ti-volume">
              &#xeb51;&nbsp;<span data-i18n="speech" />
            </h2>
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
            <h2 class="ti ti-restore">
              &#xfafd;&nbsp;<span data-i18n="restore" />
            </h2>
            <div role="group">
              <ImportButton />
              <ExportButton />
            </div>
          </label>
        </fieldset>
        <hr>

        <fieldset>
          <label>
            <h2 class="ti ti-info-circle">
              &#xeac5;&nbsp;<span data-i18n="about" />
            </h2>
            <div role="group">
              <button
                v-for="info in infos"
                :key="info.url"
                type="button"
                class="outline"
                @click="openUrl(info.url)"
              >
                <i class="ti">{{ info.code }}</i>
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
