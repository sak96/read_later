<script setup lang="ts">
import { ref } from 'vue'
import { invokeNoParseLogError } from '../composables/useTauri'
import { HardDriveDownload, HardDriveUpload } from 'lucide-vue-next'

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
    class="outline"
    type="button"
    :aria-busy="isLoading"
    :disabled="isLoading"
    @click.stop="handleClick"
  >
    <HardDriveUpload v-if="type === 'import'" />
    <HardDriveDownload v-else />
  </button>
</template>
