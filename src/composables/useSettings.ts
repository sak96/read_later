import { invokeParseLogError, invokeNoParseLogError } from './useTauri'

export async function getSetting(name: string): Promise<string | null> {
  return invokeParseLogError<string>('get_setting', { name })
}

export async function setSetting(name: string, value: string): Promise<void> {
  await invokeNoParseLogError('set_setting', { name, value })
}
