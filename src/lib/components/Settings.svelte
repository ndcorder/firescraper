<script lang="ts">
	import { get } from 'svelte/store';
	import { settings, saveSettings, type AppSettings } from '$lib/settings';

	let { open = $bindable(false) } = $props();

	let local: AppSettings = $state(get(settings));

	$effect(() => {
		if (open) {
			local = get(settings);
		}
	});

	const providers = [
		{ value: 'anthropic' as const, label: 'Anthropic (Claude)' },
		{ value: 'openai' as const, label: 'OpenAI (GPT)' },
		{ value: 'google' as const, label: 'Google (Gemini)' },
		{ value: 'openrouter' as const, label: 'OpenRouter' }
	];

	async function handleSave() {
		await saveSettings(local);
		open = false;
	}
</script>

{#if open}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<div class="fixed inset-0 bg-black/50 z-40" onclick={() => (open = false)}></div>
	<div
		class="fixed right-0 top-0 h-full w-96 bg-zinc-900 border-l border-zinc-700 z-50 overflow-y-auto p-6"
	>
		<div class="flex justify-between items-center mb-6">
			<h2 class="text-lg font-semibold text-white">Settings</h2>
			<button class="text-zinc-400 hover:text-white" onclick={() => (open = false)}>&#x2715;</button
			>
		</div>

		<div class="space-y-5">
			<div>
				<label class="block text-sm text-zinc-400 mb-1" for="firecrawl-key">Firecrawl API Key</label>
				<input
					id="firecrawl-key"
					type="password"
					bind:value={local.firecrawl_api_key}
					class="w-full bg-zinc-800 border border-zinc-600 rounded px-3 py-2 text-white text-sm"
					placeholder="fc-..."
				/>
			</div>

			<div>
				<label class="block text-sm text-zinc-400 mb-1" for="deepgram-key">Deepgram API Key</label>
				<input
					id="deepgram-key"
					type="password"
					bind:value={local.deepgram_api_key}
					class="w-full bg-zinc-800 border border-zinc-600 rounded px-3 py-2 text-white text-sm"
					placeholder="dg-..."
				/>
			</div>

			<hr class="border-zinc-700" />

			<div>
				<label class="block text-sm text-zinc-400 mb-1" for="llm-provider">LLM Provider</label>
				<select
					id="llm-provider"
					bind:value={local.llm_provider}
					class="w-full bg-zinc-800 border border-zinc-600 rounded px-3 py-2 text-white text-sm"
				>
					{#each providers as p}
						<option value={p.value}>{p.label}</option>
					{/each}
				</select>
			</div>

			{#if local.llm_provider === 'anthropic'}
				<div>
					<label class="block text-sm text-zinc-400 mb-1" for="anthropic-key">Anthropic API Key</label>
					<input
						id="anthropic-key"
						type="password"
						bind:value={local.anthropic_api_key}
						class="w-full bg-zinc-800 border border-zinc-600 rounded px-3 py-2 text-white text-sm"
						placeholder="sk-ant-..."
					/>
				</div>
			{:else if local.llm_provider === 'openai'}
				<div>
					<label class="block text-sm text-zinc-400 mb-1" for="openai-key">OpenAI API Key</label>
					<input
						id="openai-key"
						type="password"
						bind:value={local.openai_api_key}
						class="w-full bg-zinc-800 border border-zinc-600 rounded px-3 py-2 text-white text-sm"
						placeholder="sk-..."
					/>
				</div>
			{:else if local.llm_provider === 'google'}
				<div>
					<label class="block text-sm text-zinc-400 mb-1" for="google-key">Google API Key</label>
					<input
						id="google-key"
						type="password"
						bind:value={local.google_api_key}
						class="w-full bg-zinc-800 border border-zinc-600 rounded px-3 py-2 text-white text-sm"
						placeholder="AIza..."
					/>
				</div>
			{:else if local.llm_provider === 'openrouter'}
				<div>
					<label class="block text-sm text-zinc-400 mb-1" for="openrouter-key">OpenRouter API Key</label>
					<input
						id="openrouter-key"
						type="password"
						bind:value={local.openrouter_api_key}
						class="w-full bg-zinc-800 border border-zinc-600 rounded px-3 py-2 text-white text-sm"
						placeholder="sk-or-..."
					/>
				</div>
			{/if}

			<hr class="border-zinc-700" />

			<div>
				<label class="block text-sm text-zinc-400 mb-1" for="system-prompt">System Prompt</label>
				<textarea
					id="system-prompt"
					bind:value={local.system_prompt}
					rows="6"
					class="w-full bg-zinc-800 border border-zinc-600 rounded px-3 py-2 text-white text-sm font-mono"
				></textarea>
			</div>

			<button
				onclick={handleSave}
				class="w-full bg-orange-600 hover:bg-orange-500 text-white font-medium py-2 rounded transition"
			>
				Save Settings
			</button>
		</div>
	</div>
{/if}
