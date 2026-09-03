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
    state::AppState,
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

    let calls = state.calls.clone();

    // Keep a separate copy for the background task.
    let background_query = query.clone();

    // Start the slow work in the background.
    tokio::spawn(async move {
        println!("⚡ Background playback task started");

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
                        "❌ Background playback failed: {e:?}"
                    );
                }
            }

            Err(e) => {
                eprintln!(
                    "❌ Background audio extraction failed: {e:?}"
                );
            }
        }
    });

    // Respond immediately.
    let song = SongInfo {
        title: query.clone(),
        artist: "Searching...".to_string(),
        duration: 0,
        thumbnail: None,
        url: String::new(),
    };

    Ok(Json(PlayResponse {
        status: "queued".to_string(),
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
