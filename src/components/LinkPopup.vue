<script setup lang="ts">
import { openUrl } from '@tauri-apps/plugin-opener'
import { IconCheck, IconWorldWww } from '@tabler/icons-vue'

defineProps<{
  url: string | null
}>()

const emit = defineEmits<{
  close: []
}>()

function close() {
  emit('close')
}
</script>

<template>
  <dialog :open="!!url">
    <article v-if="url">
      <header>
        <button
          aria-label="Close"
          rel="prev"
          @click="close"
        />
        <IconWorldWww size="2em" />
      </header>
      <strong>{{ url }}</strong>
      <footer>
        <button
          type="button"
          @click="openUrl(url)"
        >
          <IconCheck />
        </button>
      </footer>
    </article>
  </dialog>
</template>
