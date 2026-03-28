import { invokeParseLogError } from './useTauri'

export async function setInset(): Promise<void> {
  try {
    const topResponse = await invokeParseLogError<{ position: { y: number } }>('plugin:window|outer_position', {})
    const bottomResponse = await invokeParseLogError<{ size: { height: number } }>('plugin:window|outer_size', {})
    
    if (topResponse) {
      document.documentElement.style.setProperty(
        'safe-area-inset-top',
        `${topResponse.position.y}px`
      )
    }
    if (bottomResponse) {
      const height = bottomResponse.size.height
      const top = topResponse?.position.y ?? 0
      document.documentElement.style.setProperty(
        'safe-area-inset-bottom',
        `${Math.max(window.innerHeight - height - top, 0)}px`
      )
    }
  } catch {
    if (CSS.supports('env(safe-area-inset-top)')) {
      const computedStyle = getComputedStyle(document.documentElement)
      const top = computedStyle.getPropertyValue('padding-top')
      if (top) {
        document.documentElement.style.setProperty('safe-area-inset-top', top)
      }
    }
  }
}
