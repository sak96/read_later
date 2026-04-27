<script setup lang="ts">
import { ref, provide, watch, onMounted } from 'vue'
import { getSetting, setSetting } from '../composables/useSettings'
import { THEME } from '../constants'

const themes = ['dark', 'light', 'system'] as const
type Theme = typeof themes[number]

const mode = ref<Theme>('system')

async function loadTheme() {
  const value = await getSetting(THEME)
  if (value && themes.includes(value as Theme)) {
    mode.value = value as Theme
  }
  else {
    await setSetting(THEME, 'system')
    mode.value = 'system'
  }
}

function applyTheme(newMode: Theme) {
  const html = document.documentElement
  if (newMode === 'dark' || newMode === 'light') {
    html.setAttribute('data-theme', newMode)
  }
  else {
    html.removeAttribute('data-theme')
  }
}

function setMode(newMode: Theme) {
  mode.value = newMode
  setSetting(THEME, newMode)
}

provide('theme', {
  mode,
  setMode,
})

watch(mode, (newMode) => {
  applyTheme(newMode)
})

onMounted(() => {
  loadTheme()
})
</script>

<template>
  <slot />
</template>
