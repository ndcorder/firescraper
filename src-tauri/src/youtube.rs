use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::process::Command;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct YtMetadata {
    pub title: Option<String>,
    pub channel: Option<String>,
    pub description: Option<String>,
    pub duration_string: Option<String>,
    pub upload_date: Option<String>,
    pub view_count: Option<u64>,
}

pub async fn check_ytdlp_available() -> Result<(), String> {
    Command::new("yt-dlp")
        .arg("--version")
        .output()
        .await
        .map_err(|e| {
            format!(
                "yt-dlp is not installed or not found in PATH: {}. \
                 Install it from https://github.com/yt-dlp/yt-dlp",
                e
            )
        })?;

    Ok(())
}

pub async fn extract_metadata(url: &str) -> Result<YtMetadata, String> {
    let output = Command::new("yt-dlp")
        .args(["--dump-json", "--no-download", url])
        .output()
        .await
        .map_err(|e| format!("Failed to run yt-dlp: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("yt-dlp metadata extraction failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&stdout).map_err(|e| format!("Failed to parse yt-dlp JSON output: {}", e))
}

pub async fn download_audio(url: &str, output_path: &PathBuf) -> Result<(), String> {
    let output_str = output_path
        .to_str()
        .ok_or_else(|| "Output path contains invalid UTF-8".to_string())?;

    let output = Command::new("yt-dlp")
        .args([
            "-x",
            "--audio-format",
            "wav",
            "--audio-quality",
            "0",
            "-o",
            output_str,
            url,
        ])
        .output()
        .await
        .map_err(|e| format!("Failed to run yt-dlp: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("yt-dlp audio download failed: {}", stderr));
    }

    Ok(())
}
