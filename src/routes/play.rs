use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};

use crate::{
    auth::authorized,
    downloader::download_audio,
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

    let filename = format!(
        "/tmp/icha-{}-{}.mp3",
        chat_id,
        unique_id()
    );

   let song = download_audio(&query, &filename)
    .await
    .map_err(|e| {
        eprintln!("❌ Download failed: {e:?}");

        error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to download audio",
        )
    })?;

println!("🎧 Download complete: {}", song.title);

playback::play(
    &state.calls,
    chat_id,
    &filename,
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

fn unique_id() -> u64 {
    use std::time::{
        SystemTime,
        UNIX_EPOCH,
    };

    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
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
