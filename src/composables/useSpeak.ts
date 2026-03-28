import { invokeNoParseLogError } from './useTauri'

interface SpeakOptions {
  text: string
  rate?: number
  voice?: string
}

export async function speakText(options: SpeakOptions): Promise<void> {
	const { text, rate = 1.0 } = options
  
	if ('speechSynthesis' in window) {
		window.speechSynthesis.cancel()
    
		const utterance = new SpeechSynthesisUtterance(text)
		utterance.rate = rate
    
		await invokeNoParseLogError('plugin:tts|speak', {
			text,
			rate
		})
    
		window.speechSynthesis.speak(utterance)
	}
}

export async function stopSpeaking(): Promise<void> {
	if ('speechSynthesis' in window) {
		window.speechSynthesis.cancel()
	}
	await invokeNoParseLogError('plugin:tts|stop', {})
}

export async function getVoices(): Promise<SpeechSynthesisVoice[]> {
	if ('speechSynthesis' in window) {
		return window.speechSynthesis.getVoices()
	}
	return []
}
