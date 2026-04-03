<script setup lang="ts">
import { inject } from 'vue'
import type { LocaleContext } from '../types'

const localeContext = inject<LocaleContext>('locale')

async function onLocaleChange(event: Event) {
  const target = event.target as HTMLSelectElement
  localeContext?.updateLocale(target.value)
}
</script>

<template>
  <template v-if="(localeContext?.locales.value.length) || 0 > 0">
    <fieldset>
      <div style="display: flex; align-items: center; gap: 0.5rem;">
        <h2 class="ti ti-language">
          &#xebbe;
        </h2>
        <p data-i18n="hello" />
        <select
          style="text-align-last: center;"
          @change="onLocaleChange"
        >
          <option
            v-for="locale in localeContext?.locales.value || []"
            :key="locale"
            :value="locale"
          >
            {{ locale }}
          </option>
        </select>
      </div>
    </fieldset>
  </template>
</template>
