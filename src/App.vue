<script setup lang="ts">
import { RouterView } from 'vue-router'
import { Alert, Theme, Locale, Focus } from './layouts'
import '@picocss/pico/css/pico.min.css'
</script>

<template>
  <Focus>
    <Alert>
      <Theme>
        <Locale>
          <RouterView key="$router.fullPath" />
        </Locale>
      </Theme>
    </Alert>
  </Focus>
</template>
