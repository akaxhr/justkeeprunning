use tgcalls::Calls;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎵 Icha Music Worker starting...");

    // Telegram client/session setup goes here.
    // We will connect this to your existing music-account session.

    println!("🎵 Worker ready.");

    tokio::signal::ctrl_c().await?;

    Ok(())
}
