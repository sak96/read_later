<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { getSetting, setSetting } from '../composables/useSettings'
import { Server, User, KeyRound, FolderRoot, X, Check, CloudSync, Pencil } from 'lucide-vue-next'
import {
  WEBDAV_ENABLED,
  WEBDAV_URL,
  WEBDAV_USERNAME,
  WEBDAV_PASSWORD,
  WEBDAV_PATH,
  LAST_SYNCED_AT,
} from '../constants'

const webdavEnabled = ref(false)

const webdavForm = ref({
  url: '',
  username: '',
  password: '',
  path: '',
})

const showWebdavDialog = ref(false)

async function onWebdavToggle() {
  webdavEnabled.value = !webdavEnabled.value
  await setSetting(WEBDAV_ENABLED, String(webdavEnabled.value))
}

function openWebdavDialog() {
  showWebdavDialog.value = true
}

function closeWebdavDialog() {
  showWebdavDialog.value = false
}

async function submitWebdavSettings() {
  await setSetting(WEBDAV_URL, webdavForm.value.url)
  await setSetting(WEBDAV_USERNAME, webdavForm.value.username)
  await setSetting(WEBDAV_PASSWORD, webdavForm.value.password)
  await setSetting(WEBDAV_PATH, webdavForm.value.path)

  // force resync
  await setSetting(LAST_SYNCED_AT, String(0))

  showWebdavDialog.value = false
}

onMounted(async () => {
  webdavEnabled.value = await getSetting(WEBDAV_ENABLED) === 'true'
  webdavForm.value.url = await getSetting(WEBDAV_URL) || ''
  webdavForm.value.username = await getSetting(WEBDAV_USERNAME) || ''
  webdavForm.value.password = await getSetting(WEBDAV_PASSWORD) || ''
  webdavForm.value.path = await getSetting(WEBDAV_PATH) || ''
})

</script>

<template>
  <fieldset>
    <h4>
      <CloudSync style="margin-right: 1em" />
      <span data-i18n="webdav" />
    </h4>

    <table>
      <tbody>
        <tr>
          <th data-i18n="enabled" />
          <td>
            <input
              type="checkbox"
              role="switch"
              :checked="webdavEnabled"
              @change="onWebdavToggle"
            >
          </td>
          <td>
            <button
              type="button"
              :disabled="!webdavEnabled"
              class="outline"
              @click="openWebdavDialog"
            >
              <Pencil />
            </button>
          </td>
        </tr>
      </tbody>
    </table>
  </fieldset>
  <dialog :open="showWebdavDialog">
    <article>
      <header>
        <button
          aria-label="Close"
          rel="prev"
          @click="closeWebdavDialog"
        />
        <CloudSync style="margin-right: 1em" />
        <span data-i18n="webdav" />
      </header>
      <fieldset>
        <label>
          <Server />
          <span data-i18n="webdav_url" />
          <input
            v-model="webdavForm.url"
            type="text"
          >
        </label>

        <label>
          <User />
          <span data-i18n="webdav_username" />
          <input
            v-model="webdavForm.username"
            type="text"
          >
        </label>

        <label>
          <KeyRound />
          <span data-i18n="webdav_password" />
          <input
            v-model="webdavForm.password"
            type="password"
          >
        </label>

        <label>
          <FolderRoot />
          <span data-i18n="webdav_path" />
          <input
            v-model="webdavForm.path"
            type="text"
          >
        </label>
      </fieldset>
      <footer>
        <button
          class="secondary"
          @click="closeWebdavDialog"
        >
          <X />
        </button>
        <button @click="submitWebdavSettings">
          <Check />
        </button>
      </footer>
    </article>
  </dialog>
</template>
