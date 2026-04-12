<script setup lang="ts">
import type { Component } from 'vue'
import { X, Check } from 'lucide-vue-next'

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
          <X />
        </button>
        <button @click="emit('confirm')">
          <Check />
        </button>
      </footer>
    </article>
  </dialog>
</template>
