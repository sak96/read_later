<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { getSetting, setSetting } from '../composables/useSettings'

const props = defineProps<{
  modelValue: number
}>()

const emit = defineEmits<{
  'update:modelValue': [value: number]
}>()

const RATES = [0.5, 1.0, 1.5, 2.0]
const rate = ref(1.0)

async function loadRate() {
  const value = await getSetting('rate')
  if (value !== null) {
    const parsed = parseFloat(value)
    if (RATES.includes(parsed)) {
      rate.value = parsed
      emit('update:modelValue', parsed)
    }
  }
  else {
    await setSetting('rate', '1.0')
    rate.value = 1.0
    emit('update:modelValue', 1.0)
  }
}

async function onChange(event: Event) {
  const target = event.target as HTMLInputElement
  const newRate = parseFloat(target.value)
  rate.value = newRate
  await setSetting('rate', newRate.toString())
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
  <div role="group">
    <label style="display: flex; gap: 1em; padding-left: 1em; padding-right: 1em; min-height: 2.5em; justify-content: space-evenly; background-color: var(--pico-primary-background)">
      <b>{{ rate.toFixed(1) }}x</b>
      <input
        type="range"
        min="0.5"
        step="0.5"
        max="2"
        :value="rate"
        @change="onChange"
      >
    </label>
  </div>
</template>
