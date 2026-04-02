# Firescraper Design

## Overview

Desktop app (Tauri + SvelteKit) that turns URLs into clean markdown and JSON. Paste a link, get structured content. YouTube transcripts and messy content get an optional LLM formatting pass.

## Architecture

- **Frontend**: SvelteKit (static adapter for Tauri)
- **Backend**: Rust (Tauri commands)
- **No server component** — everything runs locally

### Data Flow

1. User pastes URL → frontend invokes Tauri command
2. Rust backend calls Firecrawl `/scrape` API → raw markdown + metadata
3. If "Format with LLM" enabled → sends content to configured LLM provider
4. Returns raw + formatted content to frontend
5. Frontend renders markdown preview + JSON view

### API Key Storage

- `tauri-plugin-store` — encrypted local key-value store
- Keys: Firecrawl API key, LLM provider keys
- Never leaves the machine

## LLM Provider Abstraction

Unified trait in Rust:

```rust
#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn format_content(&self, content: &str, system_prompt: &str) -> Result<String>;
    fn name(&self) -> &str;
}
```

First-class providers:
- **Anthropic** (Claude)
- **OpenAI** (GPT-4o, etc.)
- **Google** (Gemini)
- **OpenRouter** (covers everything else)

All use their respective REST APIs directly via `reqwest`.

## UI Layout

Single screen:
- Top bar: URL input + Scrape button
- Below: "Format with LLM" toggle (auto-on for YouTube URLs) + settings gear
- Main area: tabbed output (Markdown rendered | JSON highlighted)
- Bottom: Copy + Save buttons

Settings slide-out panel:
- Firecrawl API key
- LLM provider selector + API key per provider
- Custom system prompt editor (ships with good default)

## YouTube Detection

- URL pattern match: `youtube.com/watch`, `youtu.be/`, `youtube.com/shorts`
- Auto-enables "Format with LLM" toggle
- User can override (disable for YouTube, enable for non-YouTube)

## Default LLM Prompt

```
You are a content formatter. Take the following scraped web content and:
1. Clean up formatting artifacts (navigation menus, ads, boilerplate)
2. Structure the content with proper headings, paragraphs, and lists
3. For video transcripts: add paragraph breaks, remove timestamps, fix punctuation
4. Preserve all meaningful content — do not summarize or omit
5. Output clean, readable markdown
```

## Output Formats

**Markdown tab**: Rendered preview of the formatted (or raw) markdown

**JSON tab**: Structured data including:
```json
{
  "url": "https://...",
  "title": "Page Title",
  "description": "Meta description",
  "markdown": "# Content...",
  "formatted_markdown": "# Cleaned content...",
  "scraped_at": "2026-03-31T...",
  "metadata": { ... }
}
```

## File Save

- "Save" button opens native file dialog (Tauri's `dialog` plugin)
- Saves `.md` and/or `.json` based on active tab
- Default filename: slugified page title + date
