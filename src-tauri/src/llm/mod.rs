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
