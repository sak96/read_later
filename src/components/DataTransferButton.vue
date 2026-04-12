<script setup lang="ts">
import { ref } from 'vue'
import { invokeNoParseLogError } from '../composables/useTauri'
import { IconUpload, IconDownload } from '@tabler/icons-vue'

const props = defineProps<{
  type: 'import' | 'export'
}>()

const isLoading = ref(false)

async function handleClick() {
  isLoading.value = true
  const command = props.type === 'import' ? 'pick_import_file' : 'pick_export_file'
  await invokeNoParseLogError(command, {})
  isLoading.value = false
}
</script>

<template>
  <button
    type="button"
    :aria-busy="isLoading"
    :disabled="isLoading"
    @click.stop="handleClick"
  >
    <IconUpload v-if="type === 'import'" />
    <IconDownload v-else />
  </button>
</template>
