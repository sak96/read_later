import { listen, type UnlistenFn } from '@tauri-apps/api/event'
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
): Promise<UnlistenFn | null> {
  if (currentPlatform === 'android' || currentPlatform == 'ios') {
    return listen<MediaActionEvent>('media_action', event => handler(event.payload))
  }
  return null
}
