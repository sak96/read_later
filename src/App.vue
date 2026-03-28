<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue'
import { RouterView } from 'vue-router'
import { useRouter } from 'vue-router'
import { Alert, Theme } from '@/layouts'
import { setInset } from '@/composables/useInset'
import { popIntentQueueAndExtractText } from 'tauri-plugin-mobile-sharetarget-api';
import { platform } from '@tauri-apps/plugin-os';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

let focusUnlistener: UnlistenFn | null = null;
const router = useRouter()
const currentPlatform: string = platform();

onMounted(async () => {
	const focusUnlistener = await listen(
		currentPlatform === 'android' ? 'tauri://focus' : 'new-intent',
		async () => {
			let url = await popIntentQueueAndExtractText();
			if (url) {
				router.replace({ name: 'addArticle', query: {shared: url} })
			}
		}
	);
	await setInset();
})

onUnmounted(() => {
	focusUnlistener()
});
</script>

<template>
  <Alert>
    <Theme>
      <RouterView />
    </Theme>
  </Alert>
</template>
