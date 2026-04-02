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
