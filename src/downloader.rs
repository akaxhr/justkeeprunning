use anyhow::{Context, Result};
use serde::Deserialize;
use tokio::process::Command;

use crate::models::SongInfo;

#[derive(Debug, Deserialize)]
struct YtDlpInfo {
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

    /*
     * First get metadata.
     */
    let metadata_output = Command::new("yt-dlp")
        .args([
            "--no-playlist",
            "--dump-single-json",
            "--skip-download",
            &search,
        ])
        .output()
        .await
        .context("Failed to run yt-dlp metadata search")?;

    if !metadata_output.status.success() {
        let stderr =
            String::from_utf8_lossy(&metadata_output.stderr);

        anyhow::bail!(
            "yt-dlp metadata search failed: {stderr}"
        );
    }

    let metadata: YtDlpInfo =
        serde_json::from_slice(&metadata_output.stdout)
            .context("Failed to parse yt-dlp metadata")?;

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

    let url = metadata
        .webpage_url
        .unwrap_or_else(|| search.clone());

    println!("🎵 Found: {title}");
    println!("👤 Artist/Channel: {artist}");
    println!("⏱️ Duration: {duration}s");

    if thumbnail.is_some() {
        println!("🖼️ Thumbnail found");
    }

    /*
     * Now download the audio.
     */
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
            &search,
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
        url,
    })
}
