<script setup lang="ts">
import { invokeNoParseLogError } from '@/composables/useTauri'

defineProps<{
  url: string | null
}>()

const emit = defineEmits<{
  close: []
}>()

async function openExternal(url: string) {
  await invokeNoParseLogError('open_url', { url })
  emit('close')
}

function close() {
  emit('close')
}
</script>

<template>
  <dialog :open="!!url">
    <article v-if="url">
      <header>
        <button aria-label="Close" @click="close" rel="prev"></button>
        <h2 class="ti ti-world-www">&#xf38f;</h2>
      </header>
      <strong>{{ url }}</strong>
      <footer>
        <button type="button" @click="openExternal(url)">
          <i class="ti ti-check">&#xea5e;</i>
        </button>
      </footer>
    </article>
  </dialog>
</template>
