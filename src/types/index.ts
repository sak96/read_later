import { Ref } from 'vue'
export interface ArticleId {
  id: number
}

export interface Article {
  id: number
  title: string
  body: string
  url: string
}

export interface Snippet {
  prefix: string
  match_text: string | null
  suffix: string | null
}

export interface ArticleEntry {
  id: number
  url: string
  title: string
  snippet: Snippet
  created_at: string
}

export interface Setting {
  name: string
  value: string
  default_value: string
}

export type IntentEvent
  = | { TextIntent: string }
    | 'Empty'

export type FetchProgress
  = | { Downloading: string }
    | { Parsing: string }

export interface GetInsetResponse {
  top: number
  bottom: number
}

export type AlertStatus = 'success' | 'info' | 'error'

export interface AlertContext {
  updateAlertContext: (status: AlertStatus, message: string) => void
}

export interface LocaleContext {
  currentLocale: Ref<string>
  locales: Ref<string[]>
  updateLocale: (locale: string) => Promise<void>
}
