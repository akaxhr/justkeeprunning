
use anyhow::Result;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::{
    env,
    sync::Arc,
};
use tgcalls::Calls;

const DEFAULT_CHAT_ID: i64 = -1003843699243;

#[derive(Clone)]
struct AppState {
    calls: Arc<Calls>,
    worker_secret: String,
}

#[derive(Debug, Deserialize)]
struct PlayRequest {
    chat_id: i64,
    query: String,
    quality: Option<String>,
}

#[derive(Debug, Serialize)]
struct SongInfo {
    title: String,
}

#[derive(Debug, Serialize)]
struct PlayResponse {
    status: String,
    song: SongInfo,
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🎵 Icha Music Worker starting — REAL MUSIC v2");

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

    let state = AppState {
        calls,
        worker_secret,
    };

    let app = Router::new()
        .route("/play", post(play))
        .with_state(state);

    let port = env::var("PORT")
        .unwrap_or_else(|_| "8000".to_string());

    let address = format!("0.0.0.0:{port}");

    println!("🌐 Music worker API listening on {address}");

    let listener = tokio::net::TcpListener::bind(&address).await?;

    axum::serve(listener, app).await?;

    Ok(())
}

async fn play(
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

    let chat_id = if request.chat_id == 0 {
        DEFAULT_CHAT_ID
    } else {
        request.chat_id
    };

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
        std::process::id()
    );

    let title = download_audio(&query, &filename)
        .await
        .map_err(|e| {
            eprintln!("❌ Download failed: {e:?}");

            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to download audio",
            )
        })?;

    println!("🎧 Download complete: {title}");
    println!("🎙️ Starting Telegram playback...");

    state
        .calls
        .play(chat_id, &filename)
        .await
        .map_err(|e| {
            eprintln!("❌ Playback failed: {e:?}");

            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to start playback",
            )
        })?;

    println!("✅ Playback started");

    tokio::time::sleep(
        std::time::Duration::from_secs(2)
    )
    .await;

    match state.calls.unmute(chat_id).await {
        Ok(_) => println!("🔊 Worker automatically unmuted"),
        Err(e) => println!("⚠️ Unmute failed: {e:?}"),
    }

    Ok(Json(PlayResponse {
        status: "playing".to_string(),
        song: SongInfo {
            title,
        },
    }))
}

async fn download_audio(
    query: &str,
    output: &str,
) -> Result<String> {
    println!("🔎 Searching YouTube: {query}");

    let search = format!("ytsearch1:{query}");

    let status = tokio::process::Command::new("yt-dlp")
        .args([
            "--no-playlist",
            "--extract-audio",
            "--audio-format",
            "mp3",
            "--audio-quality",
            "192K",
            "--output",
            output,
            &search,
        ])
        .status()
        .await?;

    if !status.success() {
        anyhow::bail!(
            "yt-dlp exited with status {status}"
        );
    }

    Ok(query.to_string())
}

fn authorized(
    headers: &HeaderMap,
    expected: &str,
) -> bool {
    let Some(value) = headers.get("authorization") else {
        return false;
    };

    let Ok(value) = value.to_str() else {
        return false;
    };

    value == format!("Bearer {expected}")
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

