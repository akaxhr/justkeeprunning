use anyhow::Result;
use std::sync::Arc;
use tgcalls::Calls;

pub async fn play(
    calls: &Arc<Calls>,
    chat_id: i64,
    audio_url: &str,
) -> Result<()> {
    println!("🎙️ Starting Telegram playback...");
    println!("🔗 Audio stream URL ready");

    calls.play(chat_id, audio_url).await?;

    println!("✅ Playback started");

    tokio::time::sleep(
        std::time::Duration::from_secs(2),
    )
    .await;

    match calls.unmute(chat_id).await {
        Ok(_) => println!("🔊 Worker automatically unmuted"),
        Err(e) => println!("⚠️ Unmute failed: {e:?}"),
    }

    Ok(())
}
