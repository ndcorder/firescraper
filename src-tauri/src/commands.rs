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
