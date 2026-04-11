import { addPluginListener, PluginListener } from '@tauri-apps/api/core'

/*
 * Source: https://docs.rs/crate/tauri-plugin-media-session/0.2.4/source/guest-js/
 */

export type MediaAction = 'play' | 'pause' | 'stop' | 'next' | 'previous' | 'seek'

export interface MediaActionEvent {
  action: MediaAction
  seekPosition?: number
}

export async function onAction(
  handler: (event: MediaActionEvent) => void,
): Promise<PluginListener> {
  return addPluginListener<MediaActionEvent>(
    'media-session',
    'media_action',
    handler,
  )
}
