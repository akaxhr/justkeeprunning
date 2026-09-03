mod auth;
mod downloader;
mod models;
mod playback;
mod state;
mod routes;

use anyhow::Result;
use axum::{
    routing::post,
    Router,
};
use std::{
    env,
    sync::Arc,
};
use tgcalls::{Calls, CallEvent};

use state::AppState;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🎵 Icha Music Worker starting — MODULAR v1");

    let api_id: i32 = env::var("API_ID")?.parse()?;
    let api_hash = env::var("API_HASH")?;
    let worker_secret = env::var("WORKER_SECRET")?;

    println!("🔐 Loading Telegram session...");

    let (client, _) =
        ferogram::Client::quick_connect(
            "/app/icha_music.session",
            api_id,
            &api_hash,
        )
        .await?;

    println!("✅ Telegram music account connected!");

    let calls = Arc::new(Calls::new(client));

    let queues = Arc::new(
        tokio::sync::Mutex::new(
            std::collections::HashMap::new(),
        ),
    );

    // Listen for Telegram voice-chat lifecycle events.
    let event_queues = queues.clone();

    calls.on_event(move |chat_id, event| {
        let queues = event_queues.clone();

        match event {
            CallEvent::Ended => {
                println!(
                    "🛑 Voice chat ended: {chat_id}"
                );

                tokio::spawn(async move {
                    let mut queues = queues.lock().await;

                    if let Some(queue) =
                        queues.get_mut(&chat_id)
                    {
                        queue.current = None;
                        queue.queue.clear();

                        println!(
                            "🧹 Queue cleared after VC ended: {chat_id}"
                        );
                    }
                });
            }

            CallEvent::Left => {
                println!(
                    "🚪 Worker left voice chat: {chat_id}"
                );

                tokio::spawn(async move {
                    let mut queues = queues.lock().await;

                    if let Some(queue) =
                        queues.get_mut(&chat_id)
                    {
                        queue.current = None;
                        queue.queue.clear();

                        println!(
                            "🧹 Queue cleared after leaving VC: {chat_id}"
                        );
                    }
                });
            }

            CallEvent::StreamEnded(_, _) => {
                println!(
                    "🎵 Stream ended in chat: {chat_id}"
                );

                // Automatic next-track playback
                // will be connected here next.
            }

            _ => {}
        }
    });

    let state = AppState {
        calls,
        worker_secret,
        queues,
    };

    let app = Router::new()
        .route("/play", post(routes::play::play))
        .with_state(state);

    let port = env::var("PORT")
        .unwrap_or_else(|_| "8000".to_string());

    let address = format!("0.0.0.0:{port}");

    println!(
        "🌐 Music worker API listening on {address}"
    );

    let listener =
        tokio::net::TcpListener::bind(&address).await?;

    axum::serve(listener, app).await?;

    Ok(())
}
