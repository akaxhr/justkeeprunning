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
    collections::HashMap,
    env,
    sync::Arc,
};
use tgcalls::{CallEvent, Calls};

use state::{AppState, ChatQueue};

#[tokio::main]
async fn main() -> Result<()> {
    println!("🎵 Icha Music Worker starting — MODULAR v2");

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
            HashMap::<i64, ChatQueue>::new(),
        ),
    );

    let event_queues = queues.clone();
    let event_calls = calls.clone();

calls.on_event(move |chat_id, event| {
    println!(
        "📡 CALL EVENT | chat={chat_id} | {:?}",
        event
    );

    let queues = event_queues.clone();
    let calls = event_calls.clone();

        match event {
            // ─────────────────────────────────────
            // VC ENDED
            // ─────────────────────────────────────
            CallEvent::Ended => {
                println!(
                    "🛑 Voice chat ended: {chat_id}"
                );

                tokio::spawn(async move {
                    let mut queues =
                        queues.lock().await;

                    if let Some(queue) =
                        queues.get_mut(&chat_id)
                    {
                        queue.generation += 1;
                        queue.current = None;
                        queue.queue.clear();

                        println!(
                            "🧹 Queue completely cleared: {chat_id}"
                        );
                    }
                });
            }

            // ─────────────────────────────────────
            // WORKER LEFT
            // ─────────────────────────────────────
            CallEvent::Left => {
                println!(
                    "🚪 Worker left voice chat: {chat_id}"
                );

                tokio::spawn(async move {
                    let mut queues =
                        queues.lock().await;

                    if let Some(queue) =
                        queues.get_mut(&chat_id)
                    {
                        queue.generation += 1;
                        queue.current = None;
                        queue.queue.clear();

                        println!(
                            "🧹 Queue cleared after worker left: {chat_id}"
                        );
                    }
                });
            }

            // ─────────────────────────────────────
            // CURRENT TRACK FINISHED
            // ─────────────────────────────────────
            CallEvent::StreamEnded(_, _) => {
                println!(
                    "🎵 Stream ended in chat: {chat_id}"
                );

                tokio::spawn(async move {
                    play_next(
                        chat_id,
                        calls,
                        queues,
                    )
                    .await;
                });
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
    .route(
        "/play",
        post(routes::play::play),
    )
    .route(
        "/queue/{chat_id}",
        axum::routing::get(routes::queue::queue),
    )
    .with_state(state);

    let port = env::var("PORT")
        .unwrap_or_else(|_| "8000".to_string());

    let address = format!(
        "0.0.0.0:{port}"
    );

    println!(
        "🌐 Music worker API listening on {address}"
    );

    let listener =
        tokio::net::TcpListener::bind(&address)
            .await?;

    axum::serve(
        listener,
        app,
    )
    .await?;

    Ok(())
}


// ═══════════════════════════════════════════════
// PLAY NEXT TRACK
// ═══════════════════════════════════════════════

async fn play_next(
    chat_id: i64,
    calls: Arc<Calls>,
    queues: Arc<
        tokio::sync::Mutex<
            HashMap<i64, ChatQueue>,
        >,
    >,
) {
    loop {
        // ─────────────────────────────────────
        // Get next item
        // ─────────────────────────────────────

        let next = {
            let mut queues =
                queues.lock().await;

            let Some(queue) =
                queues.get_mut(&chat_id)
            else {
                println!(
                    "📭 No queue found: {chat_id}"
                );
                return;
            };

            queue.current = None;

            let Some(item) =
                queue.queue.pop_front()
            else {
                println!(
                    "📭 Queue empty: {chat_id}"
                );
                return;
            };

            queue.current =
                Some(item.clone());

            println!(
                "⏭️ Next track: {}",
                item.query
            );

            (
                item.query,
                queue.generation,
            )
        };

        let (query, generation) = next;

        println!(
            "⚡ Preparing next track: {query}"
        );

        // ─────────────────────────────────────
        // Extract audio
        // ─────────────────────────────────────

        let song =
            match crate::downloader::get_audio(
                &query
            )
            .await
            {
                Ok(song) => song,

                Err(e) => {
                    eprintln!(
                        "❌ Failed to extract '{query}': {e:?}"
                    );

                    let should_continue = {
                        let mut queues =
                            queues.lock().await;

                        if let Some(queue) =
                            queues.get_mut(&chat_id)
                        {
                            if queue.generation
                                != generation
                            {
                                false
                            } else {
                                queue.current = None;
                                true
                            }
                        } else {
                            false
                        }
                    };

                    if should_continue {
                        println!(
                            "⏭️ Skipping failed track and trying next..."
                        );

                        continue;
                    }

                    return;
                }
            };

        println!(
            "🎧 Next audio ready: {}",
            song.title
        );

        // ─────────────────────────────────────
        // Make sure queue wasn't stopped/cleared
        // while yt-dlp was working
        // ─────────────────────────────────────

        let valid = {
            let queues =
                queues.lock().await;

            queues
                .get(&chat_id)
                .map(|queue| {
                    queue.generation
                        == generation
                        && queue.current.is_some()
                })
                .unwrap_or(false)
        };

        if !valid {
            println!(
                "🛑 Next track cancelled — queue lifecycle changed"
            );

            return;
        }

        // ─────────────────────────────────────
        // Start playback
        // ─────────────────────────────────────

        match crate::playback::play(
            &calls,
            chat_id,
            &song.url,
        )
        .await
        {
            Ok(_) => {
                println!(
                    "▶️ Now playing: {}",
                    song.title
                );

                return;
            }

            Err(e) => {
                eprintln!(
                    "❌ Playback failed for '{query}': {e:?}"
                );

                let should_continue = {
                    let mut queues =
                        queues.lock().await;

                    if let Some(queue) =
                        queues.get_mut(&chat_id)
                    {
                        if queue.generation
                            != generation
                        {
                            false
                        } else {
                            queue.current = None;
                            true
                        }
                    } else {
                        false
                    }
                };

                if should_continue {
                    println!(
                        "⏭️ Playback failed — trying next queued track..."
                    );

                    continue;
                }

                return;
            }
        }
    }
}
