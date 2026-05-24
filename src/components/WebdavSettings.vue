<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { getSetting, setSetting } from '../composables/useSettings'
import { BadgeAlert, RectangleEllipsis, Server, User, KeyRound, FolderRoot, X, Check, CloudSync, Pencil } from 'lucide-vue-next'
import {
  WEBDAV_ENABLED,
  WEBDAV_URL,
  WEBDAV_USERNAME,
  WEBDAV_PASSWORD,
  WEBDAV_PATH,
  LAST_SYNCED_AT,
  WEBDAV_AUTH_TYPE,
} from '../constants'

const router = useRouter()

const webdavEnabled = ref(false)

const webdavForm = ref({
  url: '',
  username: '',
  password: '',
  path: '',
  authType: 'basic',
})

const showWebdavDialog = ref(false)

async function onWebdavToggle() {
  webdavEnabled.value = !webdavEnabled.value
  await setSetting(WEBDAV_ENABLED, String(webdavEnabled.value))
}

function goSync() {
  router.push({ name: 'splash' })
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
  await setSetting(WEBDAV_AUTH_TYPE, webdavForm.value.authType)

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
  webdavForm.value.authType = await getSetting(WEBDAV_AUTH_TYPE) || 'basic'
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
        </tr>
        <tr>
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
          <td>
            <button
              type="button"
              :disabled="!webdavEnabled"
              class="outline"
              @click="goSync"
            >
              <CloudSync />
            </button>
          </td>
        </tr>
        <tr>
          <td colspan="2">
            <BadgeAlert />
            <small data-i18n="webdav_sync_instruction" />
          </td>
        </tr>
      </tbody>
    </table>
  </fieldset>
  <dialog :open="showWebdavDialog">
    <article>
      <header>
        <button
          type="button"
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
          <KeyRound />
          <span data-i18n="webdav_auth_type" />
          <select v-model="webdavForm.authType">
            <option
              value="anonymous"
              data-i18n="webdav_auth_anonymous"
            />
            <option
              value="basic"
              data-i18n="webdav_auth_basic"
            />
            <option
              value="digest"
              data-i18n="webdav_auth_digest"
            />
          </select>
        </label>

        <template v-if="webdavForm.authType !== 'anonymous'">
          <label>
            <User />
            <span data-i18n="webdav_username" />
            <input
              v-model="webdavForm.username"
              type="text"
            >
          </label>

          <label>
            <RectangleEllipsis />
            <span data-i18n="webdav_password" />
            <input
              v-model="webdavForm.password"
              type="password"
            >
          </label>
        </template>

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
          type="button"
          class="secondary"
          @click="closeWebdavDialog"
        >
          <X />
        </button>
        <button
          type="button"
          @click="submitWebdavSettings"
        >
          <Check />
        </button>
      </footer>
    </article>
  </dialog>
</template>
