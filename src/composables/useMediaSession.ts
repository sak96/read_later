import { addPluginListener, PluginListener } from '@tauri-apps/api/core'
import { platform } from '@tauri-apps/plugin-os'

/*
 * Source: https://docs.rs/crate/tauri-plugin-media-session/0.2.4/source/guest-js/
 */

export type MediaAction = 'play' | 'pause' | 'stop' | 'next' | 'previous' | 'seek'
const currentPlatform: string = platform()

export interface MediaActionEvent {
  action: MediaAction
  seekPosition?: number
}

export async function onAction(
  handler: (event: MediaActionEvent) => void,
): Promise<PluginListener | null> {
  if (currentPlatform === 'android' || currentPlatform == 'ios') {
    return addPluginListener<MediaActionEvent>(
      'media-session',
      'media_action',
      handler,
    )
  }
  return null
}
