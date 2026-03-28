import { invoke } from '@tauri-apps/api/core'

export async function invokeParse<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  return await invoke<T>(cmd, args)
}

export async function invokeNoParse(cmd: string, args?: Record<string, unknown>): Promise<void> {
  await invoke(cmd, args)
}

export async function invokeParseLogError<T>(cmd: string, args?: Record<string, unknown>): Promise<T | null> {
  try {
    return await invoke<T>(cmd, args)
  } catch (err) {
    console.error(`\`${cmd}\` failed with error: ${err}`)
    return null
  }
}

export async function invokeNoParseLogError(cmd: string, args?: Record<string, unknown>): Promise<void> {
  try {
    await invoke(cmd, args)
  } catch (err) {
    console.error(`\`${cmd}\` failed with error: ${err}`)
  }
}
