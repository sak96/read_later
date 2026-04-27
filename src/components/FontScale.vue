<script lang="ts" setup>
import { ref, onMounted, watch } from 'vue'
import { getSetting, setSetting } from '../composables/useSettings'
import { FONT_SCALE } from '../constants'

const props = defineProps<{
  target: HTMLElement | null
}>()

const scale = ref(1)

async function loadScale() {
  const value = await getSetting(FONT_SCALE)
  console.log(value)
  if (value !== null) {
    const parsed = parseFloat(value)
    scale.value = parsed
  }
  else {
    await setSetting(FONT_SCALE, '1.0')
    scale.value = 1.0
  }
}

onMounted(() => {
  loadScale()
})

watch(scale, async (val) => {
  const size = val === 0 ? 1 : val
  await setSetting(FONT_SCALE, size.toFixed(1))

  /* eslint-disable vue/no-mutating-props */
  if (props.target) {
    props.target.style.fontSize = `${size}rem`
    props.target.style.padding = `${size}rem`
  }
  /* eslint-enable vue/no-mutating-props */
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
