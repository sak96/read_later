import { isInitialized } from 'tauri-plugin-tts-api'
import { getSetting, setSetting } from './useSettings'

export async function loadTtsSetting() {
  // check initialization
  let ttsStatus = await isInitialized()
  if (!ttsStatus.initialized) {
    await new Promise(resolve => setTimeout(resolve, 1000))
    ttsStatus = await isInitialized()

    if (!ttsStatus.initialized) {
      return false
    }
  }

  // check settings
  const value = await getSetting('tts')
  if (value == null) {
    await setSetting('tts', 'true')
    return true
  }
  return value === 'true'
}
