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

    let joined_before = calls.is_joined(chat_id).await;

    println!(
        "📡 tgcalls state before play: joined={joined_before}"
    );

    calls.play(chat_id, audio_url).await?;

    println!("✅ Playback started");

    let joined_after = calls.is_joined(chat_id).await;

    println!(
        "📡 tgcalls state after play: joined={joined_after}"
    );

    match calls.unmute(chat_id).await {
        Ok(_) => println!("🔊 Worker automatically unmuted"),
        Err(e) => println!("⚠️ Unmute failed: {e:?}"),
    }

    Ok(())
}
