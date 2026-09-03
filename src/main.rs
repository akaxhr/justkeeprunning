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
            HashMap::<i64, ChatQueue>::new(),
        ),
    );

    // Clone used by the Telegram call-event listener.
    let event_queues = queues.clone();
    let event_calls = calls.clone();

    calls.on_event(move |chat_id, event| {
        let queues = event_queues.clone();
        let calls = event_calls.clone();

        match event {
            // -------------------------------------------------
            // VOICE CHAT ENDED
            // -------------------------------------------------
            CallEvent::Ended => {
                println!(
                    "🛑 Voice chat ended: {chat_id}"
                );

                tokio::spawn(async move {
                    let mut queues = queues.lock().await;

                    if let Some(queue) =
                        queues.get_mut(&chat_id)
                    {
                        // Invalidate every existing background task.
                        queue.generation += 1;

                        queue.current = None;
                        queue.queue.clear();

                        println!(
                            "🧹 Queue cleared after VC ended: {chat_id}"
                        );
                    }
                });
            }

            // -------------------------------------------------
            // WORKER LEFT VC
            // -------------------------------------------------
            CallEvent::Left => {
                println!(
                    "🚪 Worker left voice chat: {chat_id}"
                );

                tokio::spawn(async move {
                    let mut queues = queues.lock().await;

                    if let Some(queue) =
                        queues.get_mut(&chat_id)
                    {
                        queue.generation += 1;

                        queue.current = None;
                        queue.queue.clear();

                        println!(
                            "🧹 Queue cleared after leaving VC: {chat_id}"
                        );
                    }
                });
            }

            // -------------------------------------------------
            // CURRENT TRACK FINISHED
            // -------------------------------------------------
            CallEvent::StreamEnded(_, _) => {
                println!(
                    "🎵 Stream ended in chat: {chat_id}"
                );

                tokio::spawn(async move {
                    let next_query = {
                        let mut queues =
                            queues.lock().await;

                        let Some(queue) =
                            queues.get_mut(&chat_id)
                        else {
                            println!(
                                "📭 No queue found for chat {chat_id}"
                            );
                            return;
                        };

                        // Remove the finished track.
                        queue.current = None;

                        // Get next track.
                        let Some(next) =
                            queue.queue.pop_front()
                        else {
                            println!(
                                "📭 Queue empty: {chat_id}"
                            );
                            return;
                        };

                        let query = next.query.clone();

                        // Reserve the next track immediately.
                        queue.current = Some(next);

                        println!(
                            "⏭️ Next track: {query}"
                        );

                        Some((
                            query,
                            queue.generation,
                        ))
                    };

                    let Some((
                        query,
                        generation,
                    )) = next_query
                    else {
                        return;
                    };

                    println!(
                        "⚡ Starting next-track background task"
                    );

                    match crate::downloader::get_audio(
                        &query
                    )
                    .await
                    {
                        Ok(song) => {
                            println!(
                                "🎧 Next audio ready: {}",
                                song.title
                            );

                            // Check that the VC/queue is
                            // still alive before playing.
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

                            if let Err(e) =
                                crate::playback::play(
                                    &calls,
                                    chat_id,
                                    &song.url,
                                )
                                .await
                            {
                                eprintln!(
                                    "❌ Next-track playback failed: {e:?}"
                                );

                                let mut queues =
                                    queues.lock().await;

                                if let Some(queue) =
                                    queues.get_mut(&chat_id)
                                {
                                    if queue.generation
                                        == generation
                                    {
                                        queue.current = None;
                                    }
                                }
                            }
                        }

                        Err(e) => {
                            eprintln!(
                                "❌ Next-track extraction failed: {e:?}"
                            );

                            let mut queues =
                                queues.lock().await;

                            if let Some(queue) =
                                queues.get_mut(&chat_id)
                            {
                                if queue.generation
                                    == generation
                                {
                                    queue.current = None;
                                }
                            }
                        }
                    }
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
