use anyhow::Result;
use ferogram::Client;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🎵 Icha Music Worker");
    println!("Connecting to Telegram...");

    let api_id: i32 = env::var("API_ID")?
        .parse()?;

    let api_hash = env::var("API_HASH")?;

    let (client, _) =
        Client::quick_connect(
            "icha_music.session",
            api_id,
            &api_hash,
        )
        .await?;

    println!("✅ Telegram authentication successful!");
    println!("🎵 Music worker is ready.");

    tokio::signal::ctrl_c().await?;

    Ok(())
}
