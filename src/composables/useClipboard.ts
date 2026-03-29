import { readText } from '@tauri-apps/plugin-clipboard-manager'

export async function readClipboard(): Promise<string | null> {
  try {
    return await readText()
  }
  catch {
    return null
  }
}
