<script setup lang="ts">
defineSlots<{
  default?: () => void
}>()
</script>

<template>
  <div style="
    position: fixed;
    bottom: var(--safe-area-inset-bottom, 0);
    right: 0em;
    z-index: 100;
    padding: 1em;
    display: flex;
    flex-direction: column;
    gap: 5px;
  ">
    <slot></slot>
  </div>
</template>
