<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { getSetting, setSetting } from '../composables/useSettings'
import { RATE } from '../constants'

const props = defineProps<{
  modelValue: number
}>()

const emit = defineEmits<{
  'update:modelValue': [value: number]
}>()

const rate = ref(1.0)

function normalize(value: number): number {
  if (!Number.isFinite(value))
    return 1.0
  // Clamp to [0.1, 2.0] and round to the nearest 0.1
  return Math.min(2.0, Math.max(0.1, Math.round(value * 10) / 10))
}

async function loadRate() {
  const value = await getSetting(RATE)
  const raw = value !== null ? parseFloat(value) : Number.NaN
  const parsed = normalize(raw)
  if (!Number.isFinite(raw) || raw !== parsed)
    await setSetting(RATE, parsed.toFixed(1))
  rate.value = parsed
  emit('update:modelValue', parsed)
}

async function onInput(event: Event) {
  const target = event.target as HTMLInputElement
  const newRate = normalize(parseFloat(target.value))
  rate.value = newRate
  await setSetting(RATE, newRate.toFixed(1))
  emit('update:modelValue', newRate)
}

onMounted(() => {
  loadRate()
})

watch(() => props.modelValue, (val) => {
  rate.value = val
})
</script>

<template>
  <label data-i18n="speech_rate" />
  <div
    style="text-align-last: center;"
    group="role"
  >
    <label style="display: flex; gap: 1em; padding-left: 1em; padding-right: 1em; min-height: 2.5em; justify-content: space-evenly;">
      <b>{{ rate.toFixed(1) }}x</b>
      <input
        type="range"
        min="0.1"
        step="0.1"
        max="2"
        :value="rate"
        @input="onInput"
      >
    </label>
  </div>
</template>
