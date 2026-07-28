<script setup lang="ts">
import { ref, inject } from 'vue'
import { useRouter } from 'vue-router'
import { invokeParse } from '../composables/useTauri'
import { FilePen, FileInput, FileOutput } from 'lucide-vue-next'
import type { AlertContext } from '../types'

const router = useRouter()
const isLoading = ref(false)
const alertContext = inject<AlertContext | null>('alert')

async function handleImport() {
  isLoading.value = true
  try {
    await invokeParse('pick_pronunciation_import_file', {})
  } catch (err) {
    alertContext?.updateAlertContext?.('error', `${err}`)
  }
  isLoading.value = false
}

async function handleExport() {
  isLoading.value = true
  try {
    await invokeParse('pick_pronunciation_export_file', {})
  } catch (err) {
    alertContext?.updateAlertContext?.('error', `${err}`)
  }
  isLoading.value = false
}
</script>

<template>
  <label data-i18n="pronunciation_rules" />
  <div role="group">
    <button
      type="button"
      class="outline"
      @click="router.push({ name: 'pronunciationRules' })"
    >
      <FilePen />
    </button>
    <button
      type="button"
      class="outline"
      :aria-busy="isLoading"
      :disabled="isLoading"
      @click="handleImport"
    >
      <FileInput />
    </button>
    <button
      type="button"
      class="outline"
      :aria-busy="isLoading"
      :disabled="isLoading"
      @click="handleExport"
    >
      <FileOutput />
    </button>
  </div>
</template>
