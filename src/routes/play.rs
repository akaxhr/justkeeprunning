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

    // ─────────────────────────────────────────
    // Check whether something is already playing
    // ─────────────────────────────────────────

    {
        let mut queues = state.queues.lock().await;

        let chat_queue = queues
            .entry(chat_id)
            .or_insert_with(ChatQueue::new);

        if chat_queue.current.is_some() {
            chat_queue.queue.push_back(QueueItem {
                query: query.clone(),
            });

            let position =
                chat_queue.queue.len();

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
    }

    // ─────────────────────────────────────────
    // First track
    // Extract BEFORE returning to JS
    // ─────────────────────────────────────────

    println!(
        "🔎 Extracting first track: {query}"
    );

    let song = match get_audio(&query).await {
        Ok(song) => song,

        Err(e) => {
            eprintln!(
                "❌ Audio extraction failed: {e:?}"
            );

            return Err(error(
                StatusCode::BAD_GATEWAY,
                &format!(
                    "Could not prepare this song: {e}"
                ),
            ));
        }
    };

    println!(
        "🎧 Audio ready: {}",
        song.title
    );

    // ─────────────────────────────────────────
    // Mark current track
    // ─────────────────────────────────────────

    let generation = {
        let mut queues =
            state.queues.lock().await;

        let chat_queue = queues
            .entry(chat_id)
            .or_insert_with(ChatQueue::new);

        chat_queue.current = Some(
            QueueItem {
                query: query.clone(),
            }
        );

        chat_queue.generation
    };

    // ─────────────────────────────────────────
    // Start playback
    // ─────────────────────────────────────────

    if let Err(e) = playback::play(
        &state.calls,
        chat_id,
        &song.url,
    )
    .await
    {
        eprintln!(
            "❌ Playback failed: {e:?}"
        );

        let mut queues =
            state.queues.lock().await;

        if let Some(queue) =
            queues.get_mut(&chat_id)
        {
            if queue.generation == generation {
                queue.current = None;
            }
        }

        return Err(error(
            StatusCode::BAD_GATEWAY,
            &format!(
                "Could not start playback: {e}"
            ),
        ));
    }

    println!(
        "▶️ Now playing: {}",
        song.title
    );

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
