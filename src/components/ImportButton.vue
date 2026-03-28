<script setup lang="ts">
import { ref } from 'vue'
import { invokeNoParseLogError } from '@/composables/useTauri'

const isImporting = ref(false)

async function importData() {
	isImporting.value = true
	await invokeNoParseLogError('pick_import_file', {})
	isImporting.value = false
}
</script>

<template>
  <button @click="importData" type="button" :aria-busy="isImporting">
    <i class="ti ti-upload">&#xeb47;</i>
  </button>
</template>
