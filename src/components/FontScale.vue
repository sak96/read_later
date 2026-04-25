<script lang="ts" setup>
import { ref, onMounted, watch } from 'vue'
import { getSetting, setSetting } from '../composables/useSettings'

const props = defineProps<{
  target: HTMLElement | null
}>()

const scale = ref(1)

async function loadScale() {
  const value = await getSetting('fontScale')
  console.log(value)
  if (value !== null) {
    const parsed = parseFloat(value)
    scale.value = parsed
  }
  else {
    await setSetting('fontScale', '1.0')
    scale.value = 1.0
  }
}

onMounted(() => {
  loadScale()
})

watch(scale, async (val) => {
  const size = val === 0 ? 1 : val
  await setSetting('fontScale', size.toFixed(1))

  if (props.target) {
    props.target.style.fontSize = `${size}rem`
    props.target.style.padding = `${size}rem`
  }
})
</script>

<template>
  <label data-i18n="font_scale" />
  <div>
    <b>{{ scale.toFixed(1) }}x</b>
    <input
      v-model.number="scale"
      type="range"
      min="0.5"
      max="2"
      step="0.5"
    >
    <div />
  </div>
</template>
