<script setup lang="ts">
import { ref } from 'vue'
import { invokeNoParseLogError } from '../composables/useTauri'

const isExporting = ref(false)

async function exportData() {
  isExporting.value = true
  await invokeNoParseLogError('pick_export_file', {})
  isExporting.value = false
}
</script>

<template>
  <button
    type="button"
    :aria-busy="isExporting"
    @click="exportData"
  >
    <i class="ti ti-download">&#xea96;</i>
  </button>
</template>
