# YouTube Transcription via Deepgram

**Date:** 2026-04-01
**Status:** Approved

## Problem

Firecrawl can't scrape YouTube videos. The app detects YouTube URLs but can't extract useful content from them.

## Solution

When a YouTube URL is detected, bypass Firecrawl entirely. Extract audio with `yt-dlp`, transcribe with Deepgram Nova-3, return transcript + metadata as markdown.

## Flow

```
YouTube URL detected
  → yt-dlp: extract audio (best audio stream, opus/m4a)
  → yt-dlp: extract metadata (title, channel, description, duration)
  → Deepgram Nova-3: transcribe audio → text
  → Assemble markdown (metadata header + transcript body)
  → Optional LLM formatting (already exists)
```

## Changes

1. **Settings** — Add `deepgram_api_key` field to settings store + UI
2. **Setup script** — Shell script that installs `yt-dlp` (via brew/pip) if missing
3. **Rust backend** — New `scrape_youtube` command that:
   - Calls `yt-dlp` to download audio to a temp file + extract metadata as JSON
   - Sends audio to Deepgram's `/v1/listen` endpoint (Nova-3, smart formatting on)
   - Assembles markdown: `# Title\n\nChannel | Duration | Date\n\n> Description\n\n## Transcript\n\n...`
   - Cleans up temp audio file
4. **Frontend** — Route YouTube URLs to the new command instead of `scrape_url_command`

## Dependencies

- `yt-dlp` on PATH (installed via setup script)
- Deepgram API key (user-provided via settings UI)

## Decisions

- **Deepgram over AssemblyAI**: 4x cheaper ($0.01/min vs $0.04/min), comparable accuracy
- **yt-dlp on PATH over bundled**: Simpler, setup script handles installation
- **User-provided API key**: Consistent with existing Firecrawl/LLM key pattern
