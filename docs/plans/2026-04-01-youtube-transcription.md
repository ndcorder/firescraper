# YouTube Transcription Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Transcribe YouTube videos via yt-dlp (audio extraction) + Deepgram Nova-3 (STT), returning transcript + metadata as markdown.

**Architecture:** When a YouTube URL is detected, bypass Firecrawl. Call yt-dlp as a subprocess to download audio to a temp file and extract metadata as JSON. Send the audio to Deepgram's `/v1/listen` REST API. Assemble a ScrapeResult with metadata header + transcript body. Optional LLM formatting already exists.

**Tech Stack:** Rust (reqwest, tokio::process, tempfile), Deepgram REST API, yt-dlp CLI

---

### Task 1: Add `deepgram_api_key` to Rust types

**Files:**
- Modify: `src-tauri/src/types.rs:44-52` (AppSettings struct)
- Modify: `src-tauri/src/types.rs:54-66` (Default impl)

**Step 1: Add the field to AppSettings**

In `src-tauri/src/types.rs`, add `deepgram_api_key` to the struct:

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSettings {
    pub firecrawl_api_key: String,
    pub deepgram_api_key: String,
    pub llm_provider: LlmProviderType,
    pub anthropic_api_key: String,
    pub openai_api_key: String,
    pub google_api_key: String,
    pub openrouter_api_key: String,
    pub system_prompt: String,
}
```

**Step 2: Add default value**

In the `Default` impl, add:

```rust
deepgram_api_key: String::new(),
```

**Step 3: Verify it compiles**

Run: `cd src-tauri && cargo check`
Expected: compiles with no errors

**Step 4: Commit**

```bash
git add src-tauri/src/types.rs
git commit -m "feat: add deepgram_api_key to AppSettings"
```

---

### Task 2: Add `deepgram_api_key` to frontend settings

**Files:**
- Modify: `src/lib/settings.ts:4-11` (AppSettings interface)
- Modify: `src/lib/settings.ts:21-28` (defaults object)
- Modify: `src/lib/components/Settings.svelte` (add input field)

**Step 1: Add to TypeScript interface**

In `src/lib/settings.ts`, add `deepgram_api_key: string;` to the `AppSettings` interface (after `firecrawl_api_key`).

**Step 2: Add to defaults**

In the `defaults` object, add `deepgram_api_key: '',` after `firecrawl_api_key`.

**Step 3: Add input field to Settings.svelte**

After the Firecrawl API Key `<div>` block (line ~51), add:

```svelte
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
```

**Step 4: Verify the frontend compiles**

Run: `npm run check` (from project root)
Expected: no errors

**Step 5: Commit**

```bash
git add src/lib/settings.ts src/lib/components/Settings.svelte
git commit -m "feat: add Deepgram API key to settings UI"
```

---

### Task 3: Add `tempfile` crate dependency

**Files:**
- Modify: `src-tauri/Cargo.toml`

**Step 1: Add tempfile to dependencies**

Add to `[dependencies]` in `src-tauri/Cargo.toml`:

```toml
tempfile = "3"
```

**Step 2: Verify it compiles**

Run: `cd src-tauri && cargo check`
Expected: downloads tempfile, compiles successfully

**Step 3: Commit**

```bash
git add src-tauri/Cargo.toml
git commit -m "feat: add tempfile dependency for youtube audio handling"
```

---

### Task 4: Create `youtube.rs` module — yt-dlp integration

**Files:**
- Create: `src-tauri/src/youtube.rs`
- Modify: `src-tauri/src/lib.rs` (add `mod youtube;`)

**Step 1: Add module declaration**

In `src-tauri/src/lib.rs`, add `mod youtube;` after the existing mod declarations.

**Step 2: Create youtube.rs with yt-dlp functions**

Create `src-tauri/src/youtube.rs`:

```rust
use serde::Deserialize;
use std::path::PathBuf;
use tokio::process::Command;

#[derive(Debug, Deserialize)]
pub struct YtMetadata {
    pub title: Option<String>,
    pub channel: Option<String>,
    pub description: Option<String>,
    pub duration_string: Option<String>,
    pub upload_date: Option<String>,
    pub view_count: Option<u64>,
}

pub async fn check_ytdlp_available() -> Result<(), String> {
    Command::new("yt-dlp")
        .arg("--version")
        .output()
        .await
        .map_err(|_| "yt-dlp not found on PATH. Install it with: brew install yt-dlp".to_string())?;
    Ok(())
}

pub async fn extract_metadata(url: &str) -> Result<YtMetadata, String> {
    let output = Command::new("yt-dlp")
        .args([
            "--dump-json",
            "--no-download",
            url,
        ])
        .output()
        .await
        .map_err(|e| format!("Failed to run yt-dlp: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("yt-dlp metadata failed: {}", stderr));
    }

    let json_str = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&json_str)
        .map_err(|e| format!("Failed to parse yt-dlp metadata: {}", e))
}

pub async fn download_audio(url: &str, output_path: &PathBuf) -> Result<(), String> {
    let output = Command::new("yt-dlp")
        .args([
            "-x",
            "--audio-format", "wav",
            "--audio-quality", "0",
            "-o", output_path.to_str().unwrap(),
            url,
        ])
        .output()
        .await
        .map_err(|e| format!("Failed to run yt-dlp: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("yt-dlp audio download failed: {}", stderr));
    }

    Ok(())
}
```

**Step 3: Verify it compiles**

Run: `cd src-tauri && cargo check`
Expected: compiles (functions unused warnings are fine)

**Step 4: Commit**

```bash
git add src-tauri/src/youtube.rs src-tauri/src/lib.rs
git commit -m "feat: add youtube module with yt-dlp metadata and audio extraction"
```

---

### Task 5: Create `deepgram.rs` module — transcription API

**Files:**
- Create: `src-tauri/src/deepgram.rs`
- Modify: `src-tauri/src/lib.rs` (add `mod deepgram;`)

**Step 1: Add module declaration**

In `src-tauri/src/lib.rs`, add `mod deepgram;` after the other mod declarations.

**Step 2: Create deepgram.rs**

Create `src-tauri/src/deepgram.rs`:

```rust
use reqwest::Client;
use std::path::PathBuf;

pub async fn transcribe_audio(api_key: &str, audio_path: &PathBuf) -> Result<String, String> {
    let audio_bytes = tokio::fs::read(audio_path)
        .await
        .map_err(|e| format!("Failed to read audio file: {}", e))?;

    let client = Client::new();
    let response = client
        .post("https://api.deepgram.com/v1/listen")
        .query(&[("model", "nova-3"), ("smart_format", "true")])
        .header("Authorization", format!("Token {}", api_key))
        .header("Content-Type", "audio/wav")
        .body(audio_bytes)
        .send()
        .await
        .map_err(|e| format!("Deepgram request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("Deepgram API error ({}): {}", status, body));
    }

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse Deepgram response: {}", e))?;

    let transcript = json["results"]["channels"][0]["alternatives"][0]["transcript"]
        .as_str()
        .unwrap_or("")
        .to_string();

    if transcript.is_empty() {
        return Err("Deepgram returned empty transcript".to_string());
    }

    Ok(transcript)
}
```

**Step 3: Verify it compiles**

Run: `cd src-tauri && cargo check`
Expected: compiles successfully

**Step 4: Commit**

```bash
git add src-tauri/src/deepgram.rs src-tauri/src/lib.rs
git commit -m "feat: add deepgram module for audio transcription via REST API"
```

---

### Task 6: Create `scrape_youtube_command` in commands.rs

**Files:**
- Modify: `src-tauri/src/commands.rs` (add new command)
- Modify: `src-tauri/src/lib.rs` (register command)

**Step 1: Add the new command to commands.rs**

Add at the bottom of `src-tauri/src/commands.rs`:

```rust
use crate::youtube;
use crate::deepgram;
use crate::types::ScrapeResult;

#[tauri::command]
pub async fn scrape_youtube_command(
    url: String,
    format_with_llm: bool,
    settings: AppSettings,
) -> Result<ScrapeResult, String> {
    if settings.deepgram_api_key.is_empty() {
        return Err("Deepgram API key not set. Open Settings to add it.".to_string());
    }

    youtube::check_ytdlp_available().await?;

    // Extract metadata and download audio in parallel
    let meta_url = url.clone();
    let meta_handle = tokio::spawn(async move {
        youtube::extract_metadata(&meta_url).await
    });

    let temp_dir = tempfile::tempdir()
        .map_err(|e| format!("Failed to create temp dir: {}", e))?;
    let audio_path = temp_dir.path().join("audio.wav");

    youtube::download_audio(&url, &audio_path).await?;

    let metadata = meta_handle
        .await
        .map_err(|e| format!("Metadata task failed: {}", e))?
        ?;

    // Transcribe with Deepgram
    let transcript = deepgram::transcribe_audio(
        &settings.deepgram_api_key,
        &audio_path,
    ).await?;

    // Assemble markdown
    let title = metadata.title.as_deref().unwrap_or("Untitled Video");
    let channel = metadata.channel.as_deref().unwrap_or("Unknown Channel");
    let duration = metadata.duration_string.as_deref().unwrap_or("Unknown");
    let description = metadata.description.as_deref().unwrap_or("");

    let markdown = format!(
        "# {}\n\n**{}** | Duration: {}\n\n> {}\n\n## Transcript\n\n{}",
        title, channel, duration, description, transcript
    );

    let mut result = ScrapeResult {
        url: url.clone(),
        title: metadata.title.clone(),
        description: metadata.description.clone(),
        markdown,
        formatted_markdown: None,
        scraped_at: chrono::Utc::now().to_rfc3339(),
        metadata: serde_json::to_value(&metadata).unwrap_or(serde_json::Value::Null),
    };

    if format_with_llm {
        let provider = get_provider(&settings)?;
        let formatted = provider
            .format_content(&result.markdown, &settings.system_prompt)
            .await?;
        result.formatted_markdown = Some(formatted);
    }

    Ok(result)
}
```

Note: You'll need to add `use tempfile;` at the top of commands.rs, and derive `Serialize` on `YtMetadata` in youtube.rs.

**Step 2: Register the command in lib.rs**

In `src-tauri/src/lib.rs`, add `commands::scrape_youtube_command` to the `invoke_handler` macro:

```rust
.invoke_handler(tauri::generate_handler![
    commands::scrape_url_command,
    commands::scrape_youtube_command,
    commands::check_is_youtube,
])
```

**Step 3: Verify it compiles**

Run: `cd src-tauri && cargo check`
Expected: compiles successfully

**Step 4: Commit**

```bash
git add src-tauri/src/commands.rs src-tauri/src/lib.rs src-tauri/src/youtube.rs
git commit -m "feat: add scrape_youtube_command with yt-dlp + Deepgram pipeline"
```

---

### Task 7: Route YouTube URLs to new command in frontend

**Files:**
- Modify: `src/routes/+page.svelte:31-48` (handleScrape function)

**Step 1: Update handleScrape to route YouTube URLs**

Replace the `handleScrape` function in `src/routes/+page.svelte`:

```typescript
async function handleScrape() {
    if (!url.trim()) return;
    loading = true;
    error = '';
    result = null;

    try {
        const isYt: boolean = await invoke('check_is_youtube', { url: url.trim() });
        const command = isYt ? 'scrape_youtube_command' : 'scrape_url_command';

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
```

**Step 2: Update the loading text to show YouTube-specific message**

In the loading spinner section, update:

```svelte
<span class="text-zinc-400 text-sm">
    {formatWithLlm ? 'Scraping & formatting...' : 'Scraping...'}
</span>
```

to:

```svelte
<span class="text-zinc-400 text-sm">
    {#await invoke('check_is_youtube', { url: url.trim() }) then isYt}
        {isYt ? 'Transcribing video...' : formatWithLlm ? 'Scraping & formatting...' : 'Scraping...'}
    {:catch}
        {formatWithLlm ? 'Scraping & formatting...' : 'Scraping...'}
    {/await}
</span>
```

Actually, simpler approach — just track `isYoutube` as state:

Add a reactive variable:

```typescript
let isYoutube = $state(false);
```

Update `handleUrlChange`:

```typescript
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
```

Update `handleScrape`:

```typescript
const command = isYoutube ? 'scrape_youtube_command' : 'scrape_url_command';
result = await invoke(command, { ... });
```

Update loading text:

```svelte
{isYoutube ? 'Transcribing video...' : formatWithLlm ? 'Scraping & formatting...' : 'Scraping...'}
```

**Step 3: Verify frontend compiles**

Run: `npm run check`
Expected: no errors

**Step 4: Commit**

```bash
git add src/routes/+page.svelte
git commit -m "feat: route YouTube URLs to scrape_youtube_command"
```

---

### Task 8: Create setup script

**Files:**
- Create: `setup.sh` (project root)

**Step 1: Create the setup script**

Create `setup.sh` in the project root:

```bash
#!/usr/bin/env bash
set -e

echo "Firescraper Setup"
echo "================="

# Check for yt-dlp
if command -v yt-dlp &> /dev/null; then
    echo "✓ yt-dlp found: $(yt-dlp --version)"
else
    echo "✗ yt-dlp not found. Installing..."
    if command -v brew &> /dev/null; then
        brew install yt-dlp
    elif command -v pip3 &> /dev/null; then
        pip3 install yt-dlp
    elif command -v pip &> /dev/null; then
        pip install yt-dlp
    else
        echo "Error: Could not install yt-dlp. Install manually: https://github.com/yt-dlp/yt-dlp#installation"
        exit 1
    fi
    echo "✓ yt-dlp installed: $(yt-dlp --version)"
fi

# Check for ffmpeg (needed by yt-dlp for audio conversion)
if command -v ffmpeg &> /dev/null; then
    echo "✓ ffmpeg found"
else
    echo "✗ ffmpeg not found. Installing..."
    if command -v brew &> /dev/null; then
        brew install ffmpeg
    else
        echo "Error: Could not install ffmpeg. Install manually: https://ffmpeg.org/download.html"
        exit 1
    fi
    echo "✓ ffmpeg installed"
fi

echo ""
echo "Setup complete! You can now scrape YouTube videos."
```

**Step 2: Make it executable**

Run: `chmod +x setup.sh`

**Step 3: Commit**

```bash
git add setup.sh
git commit -m "feat: add setup script for yt-dlp and ffmpeg dependencies"
```

---

### Task 9: End-to-end manual test

**Step 1: Run setup script**

Run: `./setup.sh`
Expected: yt-dlp and ffmpeg are available

**Step 2: Start the dev server**

Run: `cargo tauri dev`

**Step 3: Test the flow**

1. Open Settings, add Deepgram API key
2. Paste a YouTube URL (e.g. a short video)
3. Click Scrape
4. Verify: loading shows "Transcribing video..."
5. Verify: result shows markdown with title, channel, duration, description, transcript
6. Toggle "Format with LLM" and scrape again
7. Verify: formatted tab shows cleaned-up transcript

**Step 4: Test error cases**

1. Remove Deepgram key → should show "Deepgram API key not set"
2. Paste a non-YouTube URL → should use Firecrawl as before
3. Test with yt-dlp removed from PATH → should show helpful error

**Step 5: Commit any fixes from testing**

```bash
git add -A
git commit -m "fix: adjustments from end-to-end testing"
```
