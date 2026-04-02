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
