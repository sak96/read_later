import { invokeParseLogError, invokeNoParseLogError } from './useTauri'
import type { IntentEvent } from '@/types'

export async function getSharedText(): Promise<IntentEvent | null> {
  return invokeParseLogError<IntentEvent>('get_shared_text', {})
}

export async function clearSharedText(): Promise<void> {
  await invokeNoParseLogError('clear_shared_text', {})
}

export async function shareUrl(url: string): Promise<void> {
  await invokeNoParseLogError('share_url', { url })
}
