use serde::{Deserialize, Serialize};

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
