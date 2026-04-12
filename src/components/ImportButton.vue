<script setup lang="ts">
import { ref } from 'vue'
import { invokeNoParseLogError } from '../composables/useTauri'
import { IconUpload } from '@tabler/icons-vue'

const isImporting = ref(false)

async function importData() {
  isImporting.value = true
  await invokeNoParseLogError('pick_import_file', {})
  isImporting.value = false
}
</script>

<template>
  <button
    type="button"
    :aria-busy="isImporting"
    @click="importData"
  >
    <IconUpload />
  </button>
</template>
