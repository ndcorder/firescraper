<script lang="ts">
	import { save } from '@tauri-apps/plugin-dialog';
	import { writeTextFile } from '@tauri-apps/plugin-fs';

	interface ScrapeResult {
		url: string;
		title: string | null;
		description: string | null;
		markdown: string;
		formatted_markdown: string | null;
		scraped_at: string;
		metadata: unknown;
	}

	let { result = null }: { result: ScrapeResult | null } = $props();

	let activeTab: 'markdown' | 'json' = $state('markdown');
	let copied = $state(false);

	function getMarkdown(): string {
		if (!result) return '';
		return result.formatted_markdown || result.markdown;
	}

	function getJson(): string {
		if (!result) return '';
		return JSON.stringify(result, null, 2);
	}

	async function copyToClipboard() {
		const text = activeTab === 'markdown' ? getMarkdown() : getJson();
		await navigator.clipboard.writeText(text);
		copied = true;
		setTimeout(() => (copied = false), 2000);
	}

	async function saveToFile() {
		const ext = activeTab === 'markdown' ? 'md' : 'json';
		const slug = (result?.title || 'scrape')
			.toLowerCase()
			.replace(/[^a-z0-9]+/g, '-')
			.slice(0, 50);

		const path = await save({
			defaultPath: `${slug}.${ext}`,
			filters: [
				{
					name: activeTab === 'markdown' ? 'Markdown' : 'JSON',
					extensions: [ext]
				}
			]
		});

		if (path) {
			const content = activeTab === 'markdown' ? getMarkdown() : getJson();
			await writeTextFile(path, content);
		}
	}
</script>

{#if result}
	<div class="flex flex-col h-full">
		<div class="flex border-b border-zinc-700">
			<button
				class="px-4 py-2 text-sm font-medium transition {activeTab === 'markdown'
					? 'text-orange-400 border-b-2 border-orange-400'
					: 'text-zinc-400 hover:text-white'}"
				onclick={() => (activeTab = 'markdown')}
			>
				Markdown
			</button>
			<button
				class="px-4 py-2 text-sm font-medium transition {activeTab === 'json'
					? 'text-orange-400 border-b-2 border-orange-400'
					: 'text-zinc-400 hover:text-white'}"
				onclick={() => (activeTab = 'json')}
			>
				JSON
			</button>
		</div>

		<div class="flex-1 overflow-y-auto p-4 bg-zinc-950 rounded-b-lg">
			{#if activeTab === 'markdown'}
				<div
					class="prose prose-invert prose-sm max-w-none whitespace-pre-wrap text-sm text-zinc-200"
				>
					{getMarkdown()}
				</div>
			{:else}
				<pre class="text-sm text-green-400 font-mono whitespace-pre-wrap">{getJson()}</pre>
			{/if}
		</div>

		<div class="flex gap-2 mt-3">
			<button
				onclick={copyToClipboard}
				class="flex-1 bg-zinc-700 hover:bg-zinc-600 text-white text-sm py-2 rounded transition"
			>
				{copied ? 'Copied!' : 'Copy'}
			</button>
			<button
				onclick={saveToFile}
				class="flex-1 bg-zinc-700 hover:bg-zinc-600 text-white text-sm py-2 rounded transition"
			>
				Save
			</button>
		</div>
	</div>
{:else}
	<div class="flex items-center justify-center h-full text-zinc-500 text-sm">
		Paste a URL and hit Scrape to get started
	</div>
{/if}
