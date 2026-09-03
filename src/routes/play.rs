use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};

use crate::{
    auth::authorized,
    downloader::get_audio,
    models::{PlayRequest, PlayResponse, SongInfo},
    playback,
    state::{AppState, ChatQueue, QueueItem},
};

pub async fn play(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<PlayRequest>,
) -> Result<
    Json<PlayResponse>,
    (StatusCode, Json<serde_json::Value>),
> {
    if !authorized(&headers, &state.worker_secret) {
        return Err(error(
            StatusCode::UNAUTHORIZED,
            "Unauthorized",
        ));
    }

    let chat_id = request.chat_id;
    let query = request.query.trim().to_string();

    if query.is_empty() {
        return Err(error(
            StatusCode::BAD_REQUEST,
            "No search query supplied",
        ));
    }

    println!("🎵 PLAY request");
    println!("💬 Chat: {chat_id}");
    println!("🔎 Query: {query}");

    if let Some(quality) = &request.quality {
        println!("🎚️ Quality: {quality}");
    }

    let mut queues = state.queues.lock().await;

    let chat_queue = queues
        .entry(chat_id)
        .or_insert_with(ChatQueue::new);

    // If something is already playing,
    // add this song to the waiting queue.
    if chat_queue.current.is_some() {
        chat_queue.queue.push_back(QueueItem {
            query: query.clone(),
        });

        let position = chat_queue.queue.len();

        println!(
            "📥 Added to queue: {query} (position {position})"
        );

        let song = SongInfo {
            title: query.clone(),
            artist: "Waiting in queue".to_string(),
            duration: 0,
            thumbnail: None,
            url: String::new(),
        };

        return Ok(Json(PlayResponse {
            status: "queued".to_string(),
            song,
        }));
    }

    // Nothing is currently playing.
    // Reserve the current slot BEFORE spawning the task.
    chat_queue.current = Some(QueueItem {
        query: query.clone(),
    });

    println!("▶️ Starting first queued track: {query}");

    drop(queues);

    let calls = state.calls.clone();
    let queues = state.queues.clone();

    let background_query = query.clone();

    tokio::spawn(async move {
        println!("⚡ Background playback task started");
        println!("🔎 Extracting: {background_query}");

        match get_audio(&background_query).await {
            Ok(song) => {
                println!("🎧 Audio ready: {}", song.title);

                if let Err(e) = playback::play(
                    &calls,
                    chat_id,
                    &song.url,
                )
                .await
                {
                    eprintln!(
                        "❌ Playback failed: {e:?}"
                    );

                    // Remove the failed current track.
                    let mut queues = queues.lock().await;

                    if let Some(chat_queue) =
                        queues.get_mut(&chat_id)
                    {
                        chat_queue.current = None;
                    }
                }
            }

            Err(e) => {
                eprintln!(
                    "❌ Background audio extraction failed: {e:?}"
                );

                let mut queues = queues.lock().await;

                if let Some(chat_queue) =
                    queues.get_mut(&chat_id)
                {
                    chat_queue.current = None;
                }
            }
        }
    });

    let song = SongInfo {
        title: query.clone(),
        artist: "Searching...".to_string(),
        duration: 0,
        thumbnail: None,
        url: String::new(),
    };

    Ok(Json(PlayResponse {
        status: "playing".to_string(),
        song,
    }))
}

fn error(
    status: StatusCode,
    message: &str,
) -> (
    StatusCode,
    Json<serde_json::Value>,
) {
    (
        status,
        Json(serde_json::json!({
            "detail": message
        })),
    )
}
