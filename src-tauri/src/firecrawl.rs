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
