<script lang="ts">
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { get } from 'svelte/store';
	import { settings, loadSettings } from '$lib/settings';
	import Settings from '$lib/components/Settings.svelte';
	import OutputTabs from '$lib/components/OutputTabs.svelte';

	let url = $state('');
	let isYoutube = $state(false);
	let formatWithLlm = $state(false);
	let loading = $state(false);
	let error = $state('');
	let result: any = $state(null);
	let settingsOpen = $state(false);

	onMount(() => {
		loadSettings();
	});

	async function handleUrlChange() {
		if (url) {
			try {
				isYoutube = await invoke('check_is_youtube', { url });
				formatWithLlm = isYoutube;
			} catch {
				isYoutube = false;
			}
		} else {
			isYoutube = false;
		}
	}

	async function handleScrape() {
		if (!url.trim()) return;
		loading = true;
		error = '';
		result = null;

		try {
			const command = isYoutube ? 'scrape_youtube_command' : 'scrape_url_command';
			result = await invoke(command, {
				url: url.trim(),
				formatWithLlm,
				settings: get(settings)
			});
		} catch (e: any) {
			error = typeof e === 'string' ? e : e.message || 'Unknown error';
		} finally {
			loading = false;
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter' && !loading) {
			handleScrape();
		}
	}
</script>

<main class="h-screen flex flex-col bg-zinc-900 text-white p-4">
	<div class="flex gap-2 mb-2">
		<input
			type="url"
			bind:value={url}
			oninput={handleUrlChange}
			onkeydown={handleKeydown}
			placeholder="Paste a URL..."
			class="flex-1 bg-zinc-800 border border-zinc-600 rounded-lg px-4 py-2.5 text-white placeholder-zinc-500 focus:outline-none focus:border-orange-500 transition"
			disabled={loading}
		/>
		<button
			onclick={handleScrape}
			disabled={loading || !url.trim()}
			class="bg-orange-600 hover:bg-orange-500 disabled:bg-zinc-700 disabled:text-zinc-500 text-white font-medium px-6 py-2.5 rounded-lg transition"
		>
			{loading ? 'Scraping...' : 'Scrape'}
		</button>
	</div>

	<div class="flex items-center justify-between mb-3">
		<label class="flex items-center gap-2 text-sm text-zinc-400">
			<input type="checkbox" bind:checked={formatWithLlm} class="accent-orange-500" />
			Format with LLM
		</label>
		<button
			onclick={() => (settingsOpen = true)}
			class="text-zinc-400 hover:text-white text-sm transition flex items-center gap-1"
		>
			<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					stroke-width="2"
					d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"
				></path>
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					stroke-width="2"
					d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
				></path>
			</svg>
			Settings
		</button>
	</div>

	{#if error}
		<div class="bg-red-900/50 border border-red-700 text-red-300 text-sm rounded-lg px-4 py-2 mb-3">
			{error}
		</div>
	{/if}

	{#if loading}
		<div class="flex-1 flex items-center justify-center">
			<div class="flex flex-col items-center gap-3">
				<div
					class="w-8 h-8 border-2 border-orange-500 border-t-transparent rounded-full animate-spin"
				></div>
				<span class="text-zinc-400 text-sm">
					{isYoutube ? 'Transcribing video...' : formatWithLlm ? 'Scraping & formatting...' : 'Scraping...'}
				</span>
			</div>
		</div>
	{:else}
		<div class="flex-1 min-h-0">
			<OutputTabs {result} />
		</div>
	{/if}

	<Settings bind:open={settingsOpen} />
</main>
