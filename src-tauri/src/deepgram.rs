use reqwest::Client;
use std::path::PathBuf;

pub async fn transcribe_audio(api_key: &str, audio_path: &PathBuf) -> Result<String, String> {
    let audio_bytes = tokio::fs::read(audio_path)
        .await
        .map_err(|e| format!("Failed to read audio file: {}", e))?;

    let client = Client::new();

    let response = client
        .post("https://api.deepgram.com/v1/listen?model=nova-3&smart_format=true")
        .header("Authorization", format!("Token {}", api_key))
        .header("Content-Type", "audio/wav")
        .body(audio_bytes)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("Deepgram API error ({}): {}", status, body));
    }

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    let transcript = json["results"]["channels"][0]["alternatives"][0]["transcript"]
        .as_str()
        .unwrap_or("")
        .to_string();

    if transcript.is_empty() {
        return Err("Deepgram returned an empty transcript".to_string());
    }

    Ok(transcript)
}
