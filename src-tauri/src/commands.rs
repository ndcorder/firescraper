use crate::deepgram;
use crate::firecrawl;
use crate::llm::{
    anthropic::AnthropicProvider, google::GoogleProvider, openai::OpenAIProvider,
    openrouter::OpenRouterProvider, LlmProvider,
};
use crate::types::{AppSettings, LlmProviderType, ScrapeResult};
use crate::youtube;

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
            Ok(Box::new(AnthropicProvider::new(
                settings.anthropic_api_key.clone(),
            )))
        }
        LlmProviderType::OpenAI => {
            if settings.openai_api_key.is_empty() {
                return Err("OpenAI API key not set".to_string());
            }
            Ok(Box::new(OpenAIProvider::new(
                settings.openai_api_key.clone(),
            )))
        }
        LlmProviderType::Google => {
            if settings.google_api_key.is_empty() {
                return Err("Google API key not set".to_string());
            }
            Ok(Box::new(GoogleProvider::new(
                settings.google_api_key.clone(),
            )))
        }
        LlmProviderType::OpenRouter => {
            if settings.openrouter_api_key.is_empty() {
                return Err("OpenRouter API key not set".to_string());
            }
            Ok(Box::new(OpenRouterProvider::new(
                settings.openrouter_api_key.clone(),
            )))
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
        return Err("Firecrawl API key not set. Open Settings to add it.".to_string());
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

    let temp_dir = tempfile::tempdir().map_err(|e| format!("Failed to create temp directory: {}", e))?;
    let audio_path = temp_dir.path().join("audio.wav");

    // Run metadata extraction and audio download in parallel
    let meta_url = url.clone();
    let audio_url = url.clone();
    let audio_path_clone = audio_path.clone();

    let (meta_result, audio_result) = tokio::join!(
        youtube::extract_metadata(&meta_url),
        youtube::download_audio(&audio_url, &audio_path_clone),
    );

    let metadata = meta_result?;
    audio_result?;

    let transcript = deepgram::transcribe_audio(&settings.deepgram_api_key, &audio_path).await?;

    let title = metadata.title.as_deref().unwrap_or("Untitled Video");
    let channel = metadata.channel.as_deref().unwrap_or("Unknown Channel");
    let duration = metadata.duration_string.as_deref().unwrap_or("Unknown");
    let description = metadata.description.as_deref().unwrap_or("");

    let markdown = format!(
        "# {title}\n\n\
         **{channel}** | Duration: {duration}\n\n\
         > {description}\n\n\
         ## Transcript\n\n\
         {transcript}"
    );

    let metadata_json = serde_json::to_value(&metadata)
        .unwrap_or_else(|_| serde_json::json!({}));

    let mut result = ScrapeResult {
        url,
        title: Some(title.to_string()),
        description: metadata.description.clone(),
        markdown,
        formatted_markdown: None,
        scraped_at: chrono::Utc::now().to_rfc3339(),
        metadata: metadata_json,
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
