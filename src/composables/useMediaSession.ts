import { invoke, addPluginListener, PluginListener } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { type UnlistenFn } from '@tauri-apps/api/event'

/**
 * Media playback state. All fields are optional —
 * omitted fields preserve their previous values (merge semantics).
 *
 * Source: https://docs.rs/crate/tauri-plugin-media-session/0.2.4/source/guest-js/
 */
export interface MediaState {
  [key: string]: unknown
  /** Track title */
  title?: string
  /** Artist name */
  artist?: string
  /** Album name */
  album?: string
  /** URL to an image (JPEG/PNG). Downloaded natively by the plugin (no CORS restrictions). */
  artworkUrl?: string
  /** Track duration in seconds */
  duration?: number
  /** Current playback position in seconds */
  position?: number
  /** Playback speed multiplier (default: 1.0) */
  playbackSpeed?: number
  /** Whether media is currently playing */
  isPlaying?: boolean
  /** Whether "previous track" action is available */
  canPrev?: boolean
  /** Whether "next track" action is available */
  canNext?: boolean
  /** Whether seeking is available (default: true) */
  canSeek?: boolean
}

/** Actions that can be triggered from notification/lockscreen controls. */
export type MediaAction = 'play' | 'pause' | 'stop' | 'next' | 'previous' | 'seek'

/** Event payload received when the user interacts with media controls. */
export interface MediaActionEvent {
  /** The action triggered by the user */
  action: MediaAction
  /** Target position in seconds (only present for `'seek'` actions) */
  seekPosition?: number
}

/**
 * Update the media session state and notification.
 *
 * Auto-initializes the session and requests notification permissions on first call.
 * Only pass the fields that changed — previous values are preserved.
 *
 * @example
 * ```ts
 * // First call: set everything
 * await updateState({
 *   title: 'Bohemian Rhapsody',
 *   artist: 'Queen',
 *   album: 'A Night at the Opera',
 *   artworkUrl: 'https://cdn.example.com/cover.jpg',
 *   duration: 354,
 *   position: 0,
 *   isPlaying: true,
 *   canPrev: true,
 *   canNext: true,
 * });
 *
 * // Later: only update what changed
 * await updateState({ position: 42, isPlaying: false });
 * ```
 */
export async function updateState(state: MediaState): Promise<void> {
  await invoke('plugin:media-session|update_state', state)
}

/** Lightweight timeline update — position, duration, and/or playback speed only. */
export interface TimelineUpdate {
  [key: string]: unknown
  /** Current playback position in seconds */
  position?: number
  /** Track duration in seconds */
  duration?: number
  /** Playback speed multiplier */
  playbackSpeed?: number
}

/**
 * Lightweight timeline sync — only updates the playback position/speed
 * without rebuilding the notification.
 *
 * Use this for frequent updates during playback (e.g. on seek).
 * The session must already be initialized via `updateState()`.
 *
 * **Note:** Android automatically extrapolates the playback position
 * while `isPlaying` is `true`, so you don't need to call this every second.
 * Only call it on seek, track change, or speed change.
 *
 * @example
 * ```ts
 * // After a user seek
 * await updateTimeline({ position: 120.5 });
 *
 * // Speed change
 * await updateTimeline({ playbackSpeed: 1.5 });
 * ```
 */
export async function updateTimeline(timeline: TimelineUpdate): Promise<void> {
  await invoke('plugin:media-session|update_timeline', timeline)
}

/**
 * Clear the media session, dismiss the notification, and release all resources.
 * The session will be re-created automatically on the next `updateState()` call.
 */
export async function clear(): Promise<void> {
  await invoke('plugin:media-session|clear')
}

/**
 * Pre-initialize the media session and request notification permissions.
 *
 * This is optional — `updateState()` auto-initializes when needed.
 * Call this early if you want to prompt for notification permissions
 * before showing the first media notification.
 */
export async function initialize(): Promise<void> {
  await invoke('plugin:media-session|initialize')
}

/**
 * Listen for media control actions from the notification, lockscreen, or hardware buttons.
 *
 * @returns A listener that can be unregistered when no longer needed.
 *
 * @example
 * ```ts
 * const listener = await onAction((event) => {
 *   switch (event.action) {
 *     case 'play':  player.play();  break;
 *     case 'pause': player.pause(); break;
 *     case 'next':  player.next();  break;
 *     case 'previous': player.previous(); break;
 *     case 'seek':  player.seekTo(event.seekPosition!); break;
 *     case 'stop':  player.stop();  break;
 *   }
 * });
 *
 * // Later, to stop listening:
 * await listener.unregister();
 * ```
 */
export async function onAction(
  handler: (event: MediaActionEvent) => void,
): Promise<PluginListener> {
  return addPluginListener<MediaActionEvent>(
    'media-session',
    'media_action',
    handler,
  )
}

export async function onMediaEvent(callback: (event: MediaActionEvent) => void): Promise<UnlistenFn> {
  return listen(`media-session://media_action`, (e: any) => callback(e))
}
