use anyhow::Result;
use ferogram::Client;
use std::env;
use tgcalls::Calls;

const CHAT_ID: i64 = -1003843699243;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🎵 Icha Music Worker starting...");

    let api_id: i32 = env::var("API_ID")?.parse()?;
    let api_hash = env::var("API_HASH")?;

    println!("🔐 Loading Telegram session...");

    let (client, _) =
        Client::quick_connect(
            "/app/icha_music.session",
            api_id,
            &api_hash,
        )
        .await?;

    println!("✅ Telegram music account connected!");

    let calls = Calls::new(client);

    println!("🎙️ Joining VC...");
    println!("🎵 Playing test.mp3...");

    calls.play(CHAT_ID, "/app/test.mp3").await?;
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    println!("✅ Playback started!");
    println!("🎵 Worker staying alive...");

    tokio::signal::ctrl_c().await?;

    Ok(())
}
