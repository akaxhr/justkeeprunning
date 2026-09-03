use anyhow::{Context, Result};
use serde::Deserialize;
use tokio::process::Command;

use crate::models::SongInfo;

#[derive(Debug, Deserialize)]
struct SearchEntry {
    id: Option<String>,
    title: Option<String>,
    uploader: Option<String>,
    channel: Option<String>,
    duration: Option<f64>,
    thumbnail: Option<String>,
    webpage_url: Option<String>,
}

pub async fn download_audio(
    query: &str,
    output: &str,
) -> Result<SongInfo> {
    println!("🔎 Searching YouTube: {query}");

    let search = format!("ytsearch1:{query}");

    // Search YouTube and get the first result as JSON.
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

    let video_url = format!(
        "https://www.youtube.com/watch?v={video_id}"
    );

    println!("🎯 Selected: {video_url}");

    // Get complete metadata from the selected video.
    let metadata_output = Command::new("yt-dlp")
        .args([
            "--dump-single-json",
            "--skip-download",
            &video_url,
        ])
        .output()
        .await
        .context("Failed to get video metadata")?;

    if !metadata_output.status.success() {
        let stderr =
            String::from_utf8_lossy(&metadata_output.stderr);

        anyhow::bail!(
            "yt-dlp metadata failed: {stderr}"
        );
    }

    let metadata: SearchEntry =
        serde_json::from_slice(&metadata_output.stdout)
            .context("Failed to parse video metadata")?;

    let title = metadata
        .title
        .unwrap_or_else(|| query.to_string());

    let artist = metadata
        .uploader
        .or(metadata.channel)
        .unwrap_or_else(|| "Unknown Artist".to_string());

    let duration = metadata
        .duration
        .unwrap_or(0.0) as u64;

    let thumbnail = metadata.thumbnail;

    println!("🎵 Found: {title}");
    println!("👤 Artist/Channel: {artist}");
    println!("⏱️ Duration: {duration}s");

    if thumbnail.is_some() {
        println!("🖼️ Thumbnail found");
    }

    // Download the EXACT selected video.
    println!("⬇️ Downloading audio...");

    let status = Command::new("yt-dlp")
        .args([
            "--no-playlist",
            "--extract-audio",
            "--audio-format",
            "mp3",
            "--audio-quality",
            "192K",
            "--output",
            output,
            &video_url,
        ])
        .status()
        .await
        .context("Failed to run yt-dlp audio download")?;

    if !status.success() {
        anyhow::bail!(
            "yt-dlp audio download failed with status {status}"
        );
    }

    println!("✅ Audio download complete");

    Ok(SongInfo {
        title,
        artist,
        duration,
        thumbnail,
        url: video_url,
    })
}
