<script setup lang="ts">
import { ref, onMounted, inject } from 'vue'
import { getSetting, setSetting } from '../composables/useSettings'
import { version } from '@tauri-apps/plugin-os'
import { openUrl } from '@tauri-apps/plugin-opener'
import HomeButton from '../components/HomeButton.vue'
import ImportButton from '../components/ImportButton.vue'
import ExportButton from '../components/ExportButton.vue'
import SpeakRate from '../components/SpeakRate.vue'
import { Fab } from '../layouts'

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

async function loadTtsSetting() {
  const value = await getSetting('tts')
  if (value == null) {
    setSetting('tts', 'true')
    loadTtsSetting()
  }
  else {
    ttsEnabled.value = value === 'true'
  }
}

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
  appVersion.value = version()
  await loadTtsSetting()
})
</script>

<template>
  <article class="container page">
    <form class="container">
      <fieldset>
        <label>
          <h2 class="ti ti-palette">&#xeb01;</h2>
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
        </label>
      </fieldset>

      <fieldset>
        <div role="group">
          <div style="display: flex; align-items: center; gap: 0.5rem;">
            <h2 class="ti ti-volume">
              &#xeb51;
            </h2>
            <input
              name="terms"
              type="checkbox"
              role="switch"
              :checked="ttsEnabled"
              @change="onTtsToggle"
            >
            <div
              role="group"
              style="flex: 1;"
            >
              <SpeakRate
                :model-value="1"
                @update:model-value="() => {}"
              />
            </div>
          </div>
        </div>
      </fieldset>

      <fieldset>
        <label>
          <h2 class="ti ti-database-exclamation">&#xfa13;</h2>
          <small class="ti ti-alert-triangle">(beta) &#xea06;</small>
          <div role="group">
            <ImportButton />
            <ExportButton />
          </div>
        </label>
      </fieldset>

      <fieldset>
        <label>
          <h2 class="ti ti-info-circle">&#xeac5;</h2>
          <div role="group">
            <button
              v-for="info in infos"
              :key="info.url"
              type="button"
              class="outline"
              @click="openExternalUrl(info.url)"
            >
              <i class="ti">{{ info.code }}</i>
            </button>
          </div>
        </label>
      </fieldset>

      <table>
        <tbody>
          <tr>
            <th><i class="ti ti-tag">Version #</i></th>
            <td>{{ appVersion }}</td>
          </tr>
          <tr>
            <th><i class="ti ti-file-text-shield">&#x100f2;</i></th>
            <td>
              <a
                class="outline"
                @click="openUrl('https://github.com/sak96/read_later/blob/master/PRIVACY_POLICY.md')"
              >
                Last Updated: December 7, 2025
              </a>
            </td>
          </tr>
        </tbody>
      </table>
    </form>

    <Fab>
      <HomeButton />
    </Fab>
  </article>
</template>
