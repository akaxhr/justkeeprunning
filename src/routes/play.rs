use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};

use crate::{
    auth::authorized,
    downloader::get_audio,
    models::{PlayRequest, PlayResponse},
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

    // Find the YouTube video and extract its direct audio URL.
    let song = get_audio(&query)
        .await
        .map_err(|e| {
            eprintln!("❌ Audio extraction failed: {e:?}");

            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to find playable audio",
            )
        })?;

    println!("🎧 Audio ready: {}", song.title);

    // Start Telegram playback immediately.
    playback::play(
        &state.calls,
        chat_id,
        &song.url,
    )
    .await
    .map_err(|e| {
        eprintln!("❌ Playback failed: {e:?}");

        error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to start playback",
        )
    })?;

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
