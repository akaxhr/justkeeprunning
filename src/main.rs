use anyhow::Result;
use ferogram::Client;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🎵 Icha Music Worker starting...");

    let api_id: i32 = env::var("API_ID")?.parse()?;
    let api_hash = env::var("API_HASH")?;

    println!("🔐 Connecting to Telegram...");

    let (_client, _) =
        Client::quick_connect(
            "icha_music.session",
            api_id,
            &api_hash,
        )
        .await?;

    println!("✅ Telegram account connected!");
    println!("🎵 Music worker ready.");

    tokio::signal::ctrl_c().await?;

    Ok(())
}
