import { isInitialized } from 'tauri-plugin-tts-api'
import { getSetting, setSetting } from './useSettings'
import { TTS_ENABLED } from '../constants'

export async function loadTtsSetting() {
  // check initialization
  let ttsStatus = await isInitialized()
  if (!ttsStatus.initialized) {
    await new Promise(resolve => setTimeout(resolve, 1000))
    ttsStatus = await isInitialized()
    if (!ttsStatus.initialized) {
      console.error('TTS is not initialized. Disabling speech.')
      return false
    }
  }

  // check settings
  const value = await getSetting(TTS_ENABLED)
  if (value == null) {
    await setSetting(TTS_ENABLED, 'true')
    return true
  }
  return value === 'true'
}
