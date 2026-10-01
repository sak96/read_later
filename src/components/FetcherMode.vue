<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { getSetting, setSetting } from '../composables/useSettings'
import { FETCHER_MODE, FETCHER_MODES } from '../constants'

const props = withDefaults(defineProps<{
  persist?: boolean
}>(), { persist: true })

const emit = defineEmits<{
  change: [mode: string]
}>()

const fetcherMode = ref('html')

async function onChange(event: Event) {
  const target = event.target as HTMLSelectElement
  const newMode = target.value
  fetcherMode.value = newMode
  emit('change', newMode)
  if (props.persist) {
    await setSetting(FETCHER_MODE, newMode)
  }
}

onMounted(async () => {
  const value = await getSetting(FETCHER_MODE)
  fetcherMode.value = value || 'html'
  emit('change', fetcherMode.value)
})
</script>

<template>
  <label data-i18n="fetcher_mode" />
  <div>
    <select
      @change="onChange"
    >
      <option
        v-for="mode in FETCHER_MODES"
        :key="mode.value"
        :selected="fetcherMode === mode.value"
        :value="mode.value"
        :data-i18n="mode.label"
      />
    </select>
  </div>
</template>
