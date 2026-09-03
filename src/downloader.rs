use anyhow::{Context, Result};
use serde::Deserialize;
use tokio::process::Command;

use crate::models::SongInfo;

#[derive(Debug, Deserialize)]
struct VideoMetadata {
    title: Option<String>,
    uploader: Option<String>,
    channel: Option<String>,
    duration: Option<f64>,
    thumbnail: Option<String>,
    webpage_url: Option<String>,
}

pub async fn get_audio(
    query: &str,
) -> Result<SongInfo> {
    println!("🔎 Searching YouTube: {query}");

    let search = format!("ytsearch1:{query}");

    // Search and get the first result.
    let search_output = Command::new("yt-dlp")
        .args([
            "--flat-playlist",
            "--dump-single-json",
            "--skip-download",
            &search,
        ])
        .output()
        .await
        .context("Failed to run yt-dlp search")?;

    if !search_output.status.success() {
        let stderr =
            String::from_utf8_lossy(&search_output.stderr);

        anyhow::bail!(
            "yt-dlp search failed: {stderr}"
        );
    }

    let search_result: serde_json::Value =
        serde_json::from_slice(&search_output.stdout)
            .context("Failed to parse YouTube search result")?;

    let entry = search_result
        .get("entries")
        .and_then(|entries| entries.as_array())
        .and_then(|entries| entries.first())
        .ok_or_else(|| anyhow::anyhow!("No YouTube results found"))?;

    let video_id = entry
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("YouTube result has no video ID"))?;

    let video_url =
        format!("https://www.youtube.com/watch?v={video_id}");

    println!("🎯 Selected: {video_url}");

    // Get metadata + direct audio URL in one yt-dlp call.
    println!("🔗 Extracting audio stream...");

    let metadata_output = Command::new("yt-dlp")
        .args([
            "--dump-single-json",
            "--skip-download",
            "-f",
            "bestaudio/best",
            &video_url,
        ])
        .output()
        .await
        .context("Failed to extract audio stream")?;

    if !metadata_output.status.success() {
        let stderr =
            String::from_utf8_lossy(&metadata_output.stderr);

        anyhow::bail!(
            "yt-dlp audio extraction failed: {stderr}"
        );
    }

    let metadata: serde_json::Value =
        serde_json::from_slice(&metadata_output.stdout)
            .context("Failed to parse audio metadata")?;

    let typed: VideoMetadata =
        serde_json::from_value(metadata.clone())
            .context("Failed to parse video metadata")?;

    let audio_url = metadata
        .get("url")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            anyhow::anyhow!("No direct audio URL found")
        })?;

    let title = typed
        .title
        .unwrap_or_else(|| query.to_string());

    let artist = typed
        .uploader
        .or(typed.channel)
        .unwrap_or_else(|| "Unknown Artist".to_string());

    let duration = typed
        .duration
        .unwrap_or(0.0) as u64;

    let thumbnail = typed.thumbnail;

    println!("🎵 Found: {title}");
    println!("👤 Artist/Channel: {artist}");
    println!("⏱️ Duration: {duration}s");
    println!("🔗 Direct audio stream extracted");

    Ok(SongInfo {
        title,
        artist,
        duration,
        thumbnail,
        url: audio_url.to_string(),
    })
}
