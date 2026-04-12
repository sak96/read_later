<script setup lang="ts">
import type { Component } from 'vue'
import { IconX, IconCheck } from '@tabler/icons-vue'

defineProps<{
  icon: Component
  i18nKey: string
  message: string
  show: boolean
}>()

const emit = defineEmits<{
  confirm: []
  close: []
}>()
</script>

<template>
  <dialog :open="show">
    <article>
      <header>
        <button
          aria-label="Close"
          rel="prev"
          @click="emit('close')"
        />
        <component
          :is="icon"
          size="2em"
          style="margin-right: 1em"
        />
        <span :data-i18n="i18nKey" />
      </header>
      <strong>{{ message }}</strong>
      <footer>
        <button
          class="secondary"
          @click="emit('close')"
        >
          <IconX />
        </button>
        <button @click="emit('confirm')">
          <IconCheck />
        </button>
      </footer>
    </article>
  </dialog>
</template>
