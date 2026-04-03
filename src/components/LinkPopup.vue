<script setup lang="ts">
import { openUrl } from '@tauri-apps/plugin-opener'

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
        <h2 class="ti ti-world-www">
          &#xf38f;
        </h2>
      </header>
      <strong>{{ url }}</strong>
      <footer>
        <button
          type="button"
          @click="openUrl(url)"
        >
          <i class="ti ti-check">&#xea5e;</i>
        </button>
      </footer>
    </article>
  </dialog>
</template>
