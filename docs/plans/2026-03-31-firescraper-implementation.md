# Firescraper Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a Tauri + SvelteKit desktop app that scrapes URLs via Firecrawl and optionally formats content with an LLM.

**Architecture:** Single-screen desktop app. Rust backend handles HTTP calls to Firecrawl and LLM providers via a unified trait. SvelteKit frontend with static adapter renders the UI. API keys stored locally via tauri-plugin-store.

**Tech Stack:** Tauri v2, SvelteKit, Rust (reqwest, serde), tauri-plugin-store, tauri-plugin-dialog, tauri-plugin-fs, TypeScript

---

### Task 1: Project Scaffolding

**Files:**
- Create: `package.json`, `svelte.config.js`, `src/routes/+layout.ts`, `src/routes/+page.svelte`, `src/app.html`
- Create: `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `src-tauri/src/main.rs`, `src-tauri/src/lib.rs`

**Step 1: Create SvelteKit project**

```bash
cd /Users/kexxt/code-opensource/firescraper
npx sv create . --template minimal --types ts --no-add-ons --no-install
```

**Step 2: Configure static adapter**

```bash
npm install -D @sveltejs/adapter-static
```

Update `svelte.config.js`:
```javascript
import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),
  kit: {
    adapter: adapter({
      fallback: 'index.html'
    })
  }
};

export default config;
```

Create `src/routes/+layout.ts`:
```typescript
export const prerender = true;
export const ssr = false;
```

**Step 3: Initialize Tauri**

```bash
npm install -D @tauri-apps/cli@latest
npx tauri init
```

Answer prompts:
- App name: `firescraper`
- Window title: `Firescraper`
- Web assets: `../build`
- Dev server URL: `http://localhost:5173`
- Dev command: `npm run dev`
- Build command: `npm run build`

**Step 4: Add Tauri plugins (Cargo)**

In `src-tauri/Cargo.toml`, add:
```toml
[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-store = "2"
tauri-plugin-dialog = "2"
tauri-plugin-fs = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
reqwest = { version = "0.12", features = ["json"] }
tokio = { version = "1", features = ["full"] }
```

**Step 5: Add Tauri JS dependencies**

```bash
npm install @tauri-apps/api @tauri-apps/plugin-store @tauri-apps/plugin-dialog @tauri-apps/plugin-fs
```

**Step 6: Register plugins in main.rs**

```rust
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**Step 7: Add Tailwind CSS**

```bash
npx sv add tailwindcss --no-install
npm install
```

**Step 8: Verify dev server starts**

```bash
npm run tauri dev
```

Expected: Tauri window opens with SvelteKit default page.

**Step 9: Commit**

```bash
git init
git add -A
git commit -m "feat: scaffold Tauri + SvelteKit project with plugins"
```

---

### Task 2: Rust Types & Firecrawl Client

**Files:**
- Create: `src-tauri/src/types.rs`
- Create: `src-tauri/src/firecrawl.rs`
- Modify: `src-tauri/src/lib.rs`

**Step 1: Define shared types**

`src-tauri/src/types.rs`:
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScrapeRequest {
    pub url: String,
    pub format_with_llm: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScrapeResult {
    pub url: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub markdown: String,
    pub formatted_markdown: Option<String>,
    pub scraped_at: String,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FirecrawlResponse {
    pub success: bool,
    pub data: Option<FirecrawlData>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FirecrawlData {
    pub markdown: Option<String>,
    pub metadata: Option<FirecrawlMetadata>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FirecrawlMetadata {
    pub title: Option<String>,
    pub description: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LlmProviderType {
    Anthropic,
    OpenAI,
    Google,
    OpenRouter,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSettings {
    pub firecrawl_api_key: String,
    pub llm_provider: LlmProviderType,
    pub anthropic_api_key: String,
    pub openai_api_key: String,
    pub google_api_key: String,
    pub openrouter_api_key: String,
    pub system_prompt: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            firecrawl_api_key: String::new(),
            llm_provider: LlmProviderType::Anthropic,
            anthropic_api_key: String::new(),
            openai_api_key: String::new(),
            google_api_key: String::new(),
            openrouter_api_key: String::new(),
            system_prompt: DEFAULT_SYSTEM_PROMPT.to_string(),
        }
    }
}

pub const DEFAULT_SYSTEM_PROMPT: &str = r#"You are a content formatter. Take the following scraped web content and:
1. Clean up formatting artifacts (navigation menus, ads, boilerplate)
2. Structure the content with proper headings, paragraphs, and lists
3. For video transcripts: add paragraph breaks, remove timestamps, fix punctuation
4. Preserve all meaningful content — do not summarize or omit
5. Output clean, readable markdown"#;
```

**Step 2: Implement Firecrawl client**

`src-tauri/src/firecrawl.rs`:
```rust
use crate::types::{FirecrawlResponse, ScrapeResult};
use reqwest::Client;

pub async fn scrape_url(api_key: &str, url: &str) -> Result<ScrapeResult, String> {
    let client = Client::new();

    let response = client
        .post("https://api.firecrawl.dev/v1/scrape")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&serde_json::json!({
            "url": url,
            "formats": ["markdown"]
        }))
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("Firecrawl API error ({}): {}", status, body));
    }

    let fcr: FirecrawlResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    let data = fcr.data.ok_or("No data in Firecrawl response")?;
    let metadata = data.metadata.clone();

    Ok(ScrapeResult {
        url: url.to_string(),
        title: metadata.as_ref().and_then(|m| m.title.clone()),
        description: metadata.as_ref().and_then(|m| m.description.clone()),
        markdown: data.markdown.unwrap_or_default(),
        formatted_markdown: None,
        scraped_at: chrono::Utc::now().to_rfc3339(),
        metadata: metadata
            .map(|m| serde_json::to_value(m).unwrap_or_default())
            .unwrap_or(serde_json::Value::Null),
    })
}
```

Add `chrono` to `Cargo.toml`:
```toml
chrono = { version = "0.4", features = ["serde"] }
```

**Step 3: Register modules in lib.rs**

```rust
mod firecrawl;
mod types;
```

**Step 4: Verify it compiles**

```bash
cd src-tauri && cargo check
```

**Step 5: Commit**

```bash
git add -A
git commit -m "feat: add types and Firecrawl client"
```

---

### Task 3: LLM Provider Abstraction

**Files:**
- Create: `src-tauri/src/llm/mod.rs`
- Create: `src-tauri/src/llm/anthropic.rs`
- Create: `src-tauri/src/llm/openai.rs`
- Create: `src-tauri/src/llm/google.rs`
- Create: `src-tauri/src/llm/openrouter.rs`
- Modify: `src-tauri/src/lib.rs`

**Step 1: Define the LLM trait**

`src-tauri/src/llm/mod.rs`:
```rust
pub mod anthropic;
pub mod google;
pub mod openai;
pub mod openrouter;

use async_trait::async_trait;

#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn format_content(&self, content: &str, system_prompt: &str) -> Result<String, String>;
    fn name(&self) -> &str;
}
```

Add `async-trait` to `Cargo.toml`:
```toml
async-trait = "0.1"
```

**Step 2: Implement Anthropic provider**

`src-tauri/src/llm/anthropic.rs`:
```rust
use super::LlmProvider;
use async_trait::async_trait;
use reqwest::Client;

pub struct AnthropicProvider {
    api_key: String,
    client: Client,
}

impl AnthropicProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: Client::new(),
        }
    }
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    async fn format_content(&self, content: &str, system_prompt: &str) -> Result<String, String> {
        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&serde_json::json!({
                "model": "claude-sonnet-4-20250514",
                "max_tokens": 8192,
                "system": system_prompt,
                "messages": [{
                    "role": "user",
                    "content": content
                }]
            }))
            .send()
            .await
            .map_err(|e| format!("Anthropic request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("Anthropic API error ({}): {}", status, body));
        }

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse Anthropic response: {}", e))?;

        body["content"][0]["text"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "No text in Anthropic response".to_string())
    }

    fn name(&self) -> &str {
        "Anthropic"
    }
}
```

**Step 3: Implement OpenAI provider**

`src-tauri/src/llm/openai.rs`:
```rust
use super::LlmProvider;
use async_trait::async_trait;
use reqwest::Client;

pub struct OpenAIProvider {
    api_key: String,
    client: Client,
}

impl OpenAIProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: Client::new(),
        }
    }
}

#[async_trait]
impl LlmProvider for OpenAIProvider {
    async fn format_content(&self, content: &str, system_prompt: &str) -> Result<String, String> {
        let response = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&serde_json::json!({
                "model": "gpt-4o-mini",
                "messages": [
                    { "role": "system", "content": system_prompt },
                    { "role": "user", "content": content }
                ]
            }))
            .send()
            .await
            .map_err(|e| format!("OpenAI request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("OpenAI API error ({}): {}", status, body));
        }

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse OpenAI response: {}", e))?;

        body["choices"][0]["message"]["content"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "No content in OpenAI response".to_string())
    }

    fn name(&self) -> &str {
        "OpenAI"
    }
}
```

**Step 4: Implement Google provider**

`src-tauri/src/llm/google.rs`:
```rust
use super::LlmProvider;
use async_trait::async_trait;
use reqwest::Client;

pub struct GoogleProvider {
    api_key: String,
    client: Client,
}

impl GoogleProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: Client::new(),
        }
    }
}

#[async_trait]
impl LlmProvider for GoogleProvider {
    async fn format_content(&self, content: &str, system_prompt: &str) -> Result<String, String> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent?key={}",
            self.api_key
        );

        let response = self
            .client
            .post(&url)
            .json(&serde_json::json!({
                "system_instruction": {
                    "parts": [{ "text": system_prompt }]
                },
                "contents": [{
                    "parts": [{ "text": content }]
                }]
            }))
            .send()
            .await
            .map_err(|e| format!("Google request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("Google API error ({}): {}", status, body));
        }

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse Google response: {}", e))?;

        body["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "No text in Google response".to_string())
    }

    fn name(&self) -> &str {
        "Google"
    }
}
```

**Step 5: Implement OpenRouter provider**

`src-tauri/src/llm/openrouter.rs`:
```rust
use super::LlmProvider;
use async_trait::async_trait;
use reqwest::Client;

pub struct OpenRouterProvider {
    api_key: String,
    client: Client,
}

impl OpenRouterProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: Client::new(),
        }
    }
}

#[async_trait]
impl LlmProvider for OpenRouterProvider {
    async fn format_content(&self, content: &str, system_prompt: &str) -> Result<String, String> {
        let response = self
            .client
            .post("https://openrouter.ai/api/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("HTTP-Referer", "https://github.com/firescraper")
            .json(&serde_json::json!({
                "model": "anthropic/claude-sonnet-4",
                "messages": [
                    { "role": "system", "content": system_prompt },
                    { "role": "user", "content": content }
                ]
            }))
            .send()
            .await
            .map_err(|e| format!("OpenRouter request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("OpenRouter API error ({}): {}", status, body));
        }

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse OpenRouter response: {}", e))?;

        body["choices"][0]["message"]["content"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "No content in OpenRouter response".to_string())
    }

    fn name(&self) -> &str {
        "OpenRouter"
    }
}
```

**Step 6: Register module in lib.rs**

Add to `src-tauri/src/lib.rs`:
```rust
mod llm;
```

**Step 7: Verify it compiles**

```bash
cd src-tauri && cargo check
```

**Step 8: Commit**

```bash
git add -A
git commit -m "feat: add LLM provider abstraction with 4 providers"
```

---

### Task 4: Tauri Commands

**Files:**
- Create: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs`

**Step 1: Implement Tauri commands**

`src-tauri/src/commands.rs`:
```rust
use crate::firecrawl;
use crate::llm::{
    anthropic::AnthropicProvider, google::GoogleProvider, openai::OpenAIProvider,
    openrouter::OpenRouterProvider, LlmProvider,
};
use crate::types::{AppSettings, LlmProviderType, ScrapeResult};

fn is_youtube_url(url: &str) -> bool {
    url.contains("youtube.com/watch")
        || url.contains("youtu.be/")
        || url.contains("youtube.com/shorts")
}

fn get_provider(settings: &AppSettings) -> Result<Box<dyn LlmProvider>, String> {
    match settings.llm_provider {
        LlmProviderType::Anthropic => {
            if settings.anthropic_api_key.is_empty() {
                return Err("Anthropic API key not set".to_string());
            }
            Ok(Box::new(AnthropicProvider::new(settings.anthropic_api_key.clone())))
        }
        LlmProviderType::OpenAI => {
            if settings.openai_api_key.is_empty() {
                return Err("OpenAI API key not set".to_string());
            }
            Ok(Box::new(OpenAIProvider::new(settings.openai_api_key.clone())))
        }
        LlmProviderType::Google => {
            if settings.google_api_key.is_empty() {
                return Err("Google API key not set".to_string());
            }
            Ok(Box::new(GoogleProvider::new(settings.google_api_key.clone())))
        }
        LlmProviderType::OpenRouter => {
            if settings.openrouter_api_key.is_empty() {
                return Err("OpenRouter API key not set".to_string());
            }
            Ok(Box::new(OpenRouterProvider::new(settings.openrouter_api_key.clone())))
        }
    }
}

#[tauri::command]
pub async fn scrape_url_command(
    url: String,
    format_with_llm: bool,
    settings: AppSettings,
) -> Result<ScrapeResult, String> {
    if settings.firecrawl_api_key.is_empty() {
        return Err("Firecrawl API key not set".to_string());
    }

    let mut result = firecrawl::scrape_url(&settings.firecrawl_api_key, &url).await?;

    if format_with_llm {
        let provider = get_provider(&settings)?;
        let formatted = provider
            .format_content(&result.markdown, &settings.system_prompt)
            .await?;
        result.formatted_markdown = Some(formatted);
    }

    Ok(result)
}

#[tauri::command]
pub fn check_is_youtube(url: String) -> bool {
    is_youtube_url(&url)
}
```

**Step 2: Register commands in lib.rs**

Update `src-tauri/src/lib.rs`:
```rust
mod commands;
mod firecrawl;
mod llm;
mod types;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            commands::scrape_url_command,
            commands::check_is_youtube,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**Step 3: Verify it compiles**

```bash
cd src-tauri && cargo check
```

**Step 4: Commit**

```bash
git add -A
git commit -m "feat: add Tauri commands for scraping and YouTube detection"
```

---

### Task 5: Frontend - Settings Store & Panel

**Files:**
- Create: `src/lib/settings.ts`
- Create: `src/lib/components/Settings.svelte`

**Step 1: Create settings store**

`src/lib/settings.ts`:
```typescript
import { writable } from 'svelte/store';
import { Store } from '@tauri-apps/plugin-store';

export interface AppSettings {
  firecrawl_api_key: string;
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
  llm_provider: 'anthropic',
  anthropic_api_key: '',
  openai_api_key: '',
  google_api_key: '',
  openrouter_api_key: '',
  system_prompt: DEFAULT_PROMPT,
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
```

**Step 2: Create Settings panel component**

`src/lib/components/Settings.svelte`:
```svelte
<script lang="ts">
  import { settings, saveSettings, type AppSettings } from '$lib/settings';

  let { open = $bindable(false) } = $props();

  let local: AppSettings = $state({ ...$settings });

  $effect(() => {
    if (open) {
      local = { ...$settings };
    }
  });

  const providers = [
    { value: 'anthropic', label: 'Anthropic (Claude)' },
    { value: 'openai', label: 'OpenAI (GPT)' },
    { value: 'google', label: 'Google (Gemini)' },
    { value: 'openrouter', label: 'OpenRouter' },
  ] as const;

  async function handleSave() {
    await saveSettings(local);
    open = false;
  }
</script>

{#if open}
  <div class="fixed inset-0 bg-black/50 z-40" onclick={() => (open = false)}></div>
  <div class="fixed right-0 top-0 h-full w-96 bg-zinc-900 border-l border-zinc-700 z-50 overflow-y-auto p-6">
    <div class="flex justify-between items-center mb-6">
      <h2 class="text-lg font-semibold text-white">Settings</h2>
      <button class="text-zinc-400 hover:text-white" onclick={() => (open = false)}>✕</button>
    </div>

    <div class="space-y-5">
      <div>
        <label class="block text-sm text-zinc-400 mb-1">Firecrawl API Key</label>
        <input
          type="password"
          bind:value={local.firecrawl_api_key}
          class="w-full bg-zinc-800 border border-zinc-600 rounded px-3 py-2 text-white text-sm"
          placeholder="fc-..."
        />
      </div>

      <hr class="border-zinc-700" />

      <div>
        <label class="block text-sm text-zinc-400 mb-1">LLM Provider</label>
        <select
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
          <label class="block text-sm text-zinc-400 mb-1">Anthropic API Key</label>
          <input type="password" bind:value={local.anthropic_api_key} class="w-full bg-zinc-800 border border-zinc-600 rounded px-3 py-2 text-white text-sm" placeholder="sk-ant-..." />
        </div>
      {:else if local.llm_provider === 'openai'}
        <div>
          <label class="block text-sm text-zinc-400 mb-1">OpenAI API Key</label>
          <input type="password" bind:value={local.openai_api_key} class="w-full bg-zinc-800 border border-zinc-600 rounded px-3 py-2 text-white text-sm" placeholder="sk-..." />
        </div>
      {:else if local.llm_provider === 'google'}
        <div>
          <label class="block text-sm text-zinc-400 mb-1">Google API Key</label>
          <input type="password" bind:value={local.google_api_key} class="w-full bg-zinc-800 border border-zinc-600 rounded px-3 py-2 text-white text-sm" placeholder="AIza..." />
        </div>
      {:else if local.llm_provider === 'openrouter'}
        <div>
          <label class="block text-sm text-zinc-400 mb-1">OpenRouter API Key</label>
          <input type="password" bind:value={local.openrouter_api_key} class="w-full bg-zinc-800 border border-zinc-600 rounded px-3 py-2 text-white text-sm" placeholder="sk-or-..." />
        </div>
      {/if}

      <hr class="border-zinc-700" />

      <div>
        <label class="block text-sm text-zinc-400 mb-1">System Prompt</label>
        <textarea
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
```

**Step 3: Verify it compiles**

```bash
npm run check
```

**Step 4: Commit**

```bash
git add -A
git commit -m "feat: add settings store and settings panel"
```

---

### Task 6: Frontend - Main UI

**Files:**
- Modify: `src/routes/+page.svelte`
- Create: `src/lib/components/OutputTabs.svelte`

**Step 1: Create output tabs component**

`src/lib/components/OutputTabs.svelte`:
```svelte
<script lang="ts">
  import { save } from '@tauri-apps/plugin-dialog';
  import { writeTextFile } from '@tauri-apps/plugin-fs';

  let { result = null }: { result: any } = $props();

  let activeTab: 'markdown' | 'json' = $state('markdown');
  let copied = $state(false);

  function getMarkdown() {
    if (!result) return '';
    return result.formatted_markdown || result.markdown;
  }

  function getJson() {
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
          extensions: [ext],
        },
      ],
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

    <div class="flex-1 overflow-y-auto p-4 bg-zinc-950 rounded-b">
      {#if activeTab === 'markdown'}
        <div class="prose prose-invert prose-sm max-w-none whitespace-pre-wrap font-mono text-sm text-zinc-200">
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
```

**Step 2: Build the main page**

`src/routes/+page.svelte`:
```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { settings, loadSettings } from '$lib/settings';
  import Settings from '$lib/components/Settings.svelte';
  import OutputTabs from '$lib/components/OutputTabs.svelte';

  let url = $state('');
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
      const isYt: boolean = await invoke('check_is_youtube', { url });
      formatWithLlm = isYt;
    }
  }

  async function handleScrape() {
    if (!url.trim()) return;
    loading = true;
    error = '';
    result = null;

    try {
      result = await invoke('scrape_url_command', {
        url: url.trim(),
        formatWithLlm,
        settings: $settings,
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
  <!-- Top bar -->
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

  <!-- Controls row -->
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
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"></path>
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"></path>
      </svg>
      Settings
    </button>
  </div>

  <!-- Error -->
  {#if error}
    <div class="bg-red-900/50 border border-red-700 text-red-300 text-sm rounded-lg px-4 py-2 mb-3">
      {error}
    </div>
  {/if}

  <!-- Loading spinner -->
  {#if loading}
    <div class="flex-1 flex items-center justify-center">
      <div class="flex flex-col items-center gap-3">
        <div class="w-8 h-8 border-2 border-orange-500 border-t-transparent rounded-full animate-spin"></div>
        <span class="text-zinc-400 text-sm">
          {formatWithLlm ? 'Scraping & formatting...' : 'Scraping...'}
        </span>
      </div>
    </div>
  {:else}
    <!-- Output -->
    <div class="flex-1 min-h-0">
      <OutputTabs {result} />
    </div>
  {/if}

  <Settings bind:open={settingsOpen} />
</main>
```

**Step 3: Verify frontend compiles**

```bash
npm run check
```

**Step 4: Verify full app starts**

```bash
npm run tauri dev
```

**Step 5: Commit**

```bash
git add -A
git commit -m "feat: add main UI with URL input, output tabs, and settings panel"
```

---

### Task 7: Permissions & Polish

**Files:**
- Modify: `src-tauri/capabilities/default.json`

**Step 1: Configure Tauri permissions**

Tauri v2 requires explicit permissions. Update `src-tauri/capabilities/default.json`:
```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Capability for the main window",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "store:default",
    "dialog:default",
    "fs:default"
  ]
}
```

**Step 2: Verify full app builds**

```bash
npm run tauri build
```

**Step 3: Commit**

```bash
git add -A
git commit -m "feat: configure Tauri permissions and finalize app"
```
