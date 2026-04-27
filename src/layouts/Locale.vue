<script setup lang="ts">
import { ref, provide, onMounted } from 'vue'
import I18n from '@razein97/tauri-plugin-i18n'
import { getSetting, setSetting } from '../composables/useSettings'
import { locale } from '@tauri-apps/plugin-os'
import type { LocaleContext } from '../types'
import { LOCALE } from '../constants'

const currentLocale = ref('en')
const locales = ref <string[]> ([])

async function updateLocale(newLocale: string) {
  currentLocale.value = newLocale
  await setSetting(LOCALE, newLocale)
  await I18n.setLocale(currentLocale.value)
}

provide<LocaleContext>('locale', {
  currentLocale,
  locales,
  updateLocale,
})

onMounted(async () => {
  await I18n.getInstance().load()
  locales.value = await I18n.getAvailableLocales()
  let saved = await getSetting(LOCALE)
  if (!saved || !locales.value.includes(saved)) {
    saved = await locale()
    if (saved !== null) {
      if (!saved || !locales.value.includes(saved)) {
        saved = 'en'
      }
    }
    await setSetting(LOCALE, currentLocale.value)
  }
  currentLocale.value = saved || 'en'
  await I18n.setLocale(currentLocale.value)
},
)
</script>

<template>
  <slot />
</template>
