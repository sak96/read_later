<script setup lang="ts">
import { inject } from 'vue'
import type { LocaleContext } from '../types'
import { Languages } from 'lucide-vue-next'

const localeContext = inject<LocaleContext>('locale')

async function onLocaleChange(event: Event) {
  const target = event.target as HTMLSelectElement
  localeContext?.updateLocale(target.value)
}
</script>

<template>
  <template v-if="(localeContext?.locales.value.length) || 0 > 0">
    <fieldset>
      <h4>
        <Languages style="margin-right: 1em" />
        <span data-i18n="locale" />
      </h4>
      <select
        style="text-align-last: center;"
        @change="onLocaleChange"
      >
        <option
          v-for="locale in localeContext?.locales.value || []"
          :key="locale"
          :selected="localeContext?.currentLocale.value == locale"
          :value="locale"
        >
          {{ locale }}
        </option>
      </select>
    </fieldset>
  </template>
</template>
