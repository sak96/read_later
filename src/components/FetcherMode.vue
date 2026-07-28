<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { getSetting, setSetting } from '../composables/useSettings'
import { FETCHER_MODE } from '../constants'

const fetcherMode = ref('html')

const fetcherModes = [
  { value: 'html', label: 'fetcher_html' },
  { value: 'html_js', label: 'fetcher_html_js' },
  { value: 'html_js_auth', label: 'fetcher_html_js_auth' },
]

async function onChange(event: Event) {
  const target = event.target as HTMLSelectElement
  const newMode = target.value
  fetcherMode.value = newMode
  await setSetting(FETCHER_MODE, newMode)
}

onMounted(async () => {
  const value = await getSetting(FETCHER_MODE)
  fetcherMode.value = value || 'html'
})
</script>

<template>
  <label data-i18n="fetcher_mode" />
  <div>
    <select
      @change="onChange"
    >
      <option
        v-for="mode in fetcherModes"
        :key="mode.value"
        :selected="fetcherMode === mode.value"
        :value="mode.value"
        :data-i18n="mode.label"
      />
    </select>
  </div>
</template>
