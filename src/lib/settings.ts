import { writable } from 'svelte/store';
import { Store } from '@tauri-apps/plugin-store';

export interface AppSettings {
	firecrawl_api_key: string;
	deepgram_api_key: string;
	llm_provider: 'anthropic' | 'openai' | 'google' | 'openrouter';
	anthropic_api_key: string;
	openai_api_key: string;
	google_api_key: string;
	openrouter_api_key: string;
	system_prompt: string;
}

const DEFAULT_PROMPT = `You are a content formatter. Take the following scraped web content and:
1. Clean up formatting artifacts (navigation menus, ads, boilerplate)
2. Structure the content with proper headings, paragraphs, and lists
3. For video transcripts: add paragraph breaks, remove timestamps, fix punctuation
4. Preserve all meaningful content — do not summarize or omit
5. Output clean, readable markdown`;

const defaults: AppSettings = {
	firecrawl_api_key: '',
	deepgram_api_key: '',
	llm_provider: 'anthropic',
	anthropic_api_key: '',
	openai_api_key: '',
	google_api_key: '',
	openrouter_api_key: '',
	system_prompt: DEFAULT_PROMPT
};

export const settings = writable<AppSettings>(defaults);

let store: Store | null = null;

export async function loadSettings() {
	store = await Store.load('settings.json');
	const saved = await store.get<AppSettings>('settings');
	if (saved) {
		settings.set({ ...defaults, ...saved });
	}
}

export async function saveSettings(s: AppSettings) {
	if (!store) store = await Store.load('settings.json');
	await store.set('settings', s);
	await store.save();
	settings.set(s);
}
