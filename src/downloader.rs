use anyhow::Result;

pub async fn download_audio(
    query: &str,
    output: &str,
) -> Result<String> {
    println!("🔎 Searching YouTube: {query}");

    let search = format!("ytsearch1:{query}");

    let status = tokio::process::Command::new("yt-dlp")
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
        .await?;

    if !status.success() {
        anyhow::bail!(
            "yt-dlp exited with status {status}"
        );
    }

    Ok(query.to_string())
}
