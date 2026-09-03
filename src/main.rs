
use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, VecDeque},
    env,
    sync::Arc,
};
use tokio::sync::Mutex;
use tgcalls::Calls;

const DEFAULT_CHAT_ID: i64 = -1003843699243;

#[derive(Clone)]
struct AppState {
    calls: Arc<Calls>,
    worker_secret: String,
    queues: Arc<Mutex<HashMap<i64, VecDeque<Song>>>>,
    current: Arc<Mutex<HashMap<i64, Song>>>,
}

#[derive(Debug, Clone)]
struct Song {
    title: String,
    filename: String,
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
    position: Option<usize>,
}

#[derive(Debug, Serialize)]
struct QueueResponse {
    current: Option<SongInfo>,
    queue: Vec<SongInfo>,
}

#[derive(Debug, Serialize)]
struct SkipResponse {
    status: String,
    title: Option<String>,
}

#[derive(Debug, Serialize)]
struct StopResponse {
    status: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🎵 Icha Music Worker starting — QUEUE v1");

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
        queues: Arc::new(Mutex::new(HashMap::new())),
        current: Arc::new(Mutex::new(HashMap::new())),
    };

    let app = Router::new()
        .route("/play", post(play))
        .route("/skip", post(skip))
        .route("/stop", post(stop))
        .route("/queue/{chat_id}", get(queue))
        .with_state(state);

    let port = env::var("PORT")
        .unwrap_or_else(|_| "8000".to_string());

    let address = format!("0.0.0.0:{port}");

    println!("🌐 Music worker API listening on {address}");

    let listener =
        tokio::net::TcpListener::bind(&address).await?;

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

    /*
     * Download first.
     */
    let filename = format!(
        "/tmp/icha-{}-{}-{}.mp3",
        chat_id,
        std::process::id(),
        unique_id()
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

    let song = Song {
        title,
        filename,
    };

    /*
     * Check whether this chat is already playing something.
     */
    let mut current = state.current.lock().await;

    if current.contains_key(&chat_id) {
        drop(current);

        let mut queues = state.queues.lock().await;

        let queue = queues
            .entry(chat_id)
            .or_insert_with(VecDeque::new);

        queue.push_back(song.clone());

        let position = queue.len();

        println!(
            "🎶 Queued '{}' at position {}",
            song.title, position
        );

        return Ok(Json(PlayResponse {
            status: "queued".to_string(),
            song: SongInfo {
                title: song.title,
            },
            position: Some(position),
        }));
    }

    /*
     * Nothing is currently playing.
     * Start this song immediately.
     */
    current.insert(chat_id, song.clone());
    drop(current);

    println!("🎙️ Starting Telegram playback...");

    if let Err(e) =
        state.calls.play(chat_id, &song.filename).await
    {
        eprintln!("❌ Playback failed: {e:?}");

        let mut current = state.current.lock().await;
        current.remove(&chat_id);

        return Err(error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to start playback",
        ));
    }

    println!("✅ Playback started");

    tokio::time::sleep(
        std::time::Duration::from_secs(2),
    )
    .await;

    match state.calls.unmute(chat_id).await {
        Ok(_) => println!("🔊 Worker automatically unmuted"),
        Err(e) => println!("⚠️ Unmute failed: {e:?}"),
    }

    Ok(Json(PlayResponse {
        status: "playing".to_string(),
        song: SongInfo {
            title: song.title,
        },
        position: None,
    }))
}

async fn skip(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<PlayRequest>,
) -> Result<
    Json<SkipResponse>,
    (StatusCode, Json<serde_json::Value>),
> {
    if !authorized(&headers, &state.worker_secret) {
        return Err(error(
            StatusCode::UNAUTHORIZED,
            "Unauthorized",
        ));
    }

    let chat_id = request.chat_id;

    println!("⏭️ SKIP request for {chat_id}");

    let next_song = {
        let mut queues = state.queues.lock().await;

        queues
            .get_mut(&chat_id)
            .and_then(|queue| queue.pop_front())
    };

    if let Some(song) = next_song {
        println!("🎵 Starting next song: {}", song.title);

        state
            .calls
            .play(chat_id, &song.filename)
            .await
            .map_err(|e| {
                eprintln!("❌ Next playback failed: {e:?}");

                error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to start next song",
                )
            })?;

        tokio::time::sleep(
            std::time::Duration::from_secs(2),
        )
        .await;

        let _ = state.calls.unmute(chat_id).await;

        let mut current = state.current.lock().await;
        current.insert(chat_id, song.clone());

        Ok(Json(SkipResponse {
            status: "playing".to_string(),
            title: Some(song.title),
        }))
    } else {
        state.calls.stop(chat_id).await.ok();

        let mut current = state.current.lock().await;
        current.remove(&chat_id);

        println!("⏹️ Queue empty, playback stopped");

        Ok(Json(SkipResponse {
            status: "stopped".to_string(),
            title: None,
        }))
    }
}

async fn stop(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<PlayRequest>,
) -> Result<
    Json<StopResponse>,
    (StatusCode, Json<serde_json::Value>),
> {
    if !authorized(&headers, &state.worker_secret) {
        return Err(error(
            StatusCode::UNAUTHORIZED,
            "Unauthorized",
        ));
    }

    let chat_id = request.chat_id;

    println!("⏹️ STOP request for {chat_id}");

    state.calls.stop(chat_id).await.map_err(|e| {
        eprintln!("❌ Stop failed: {e:?}");

        error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to stop playback",
        )
    })?;

    {
        let mut current = state.current.lock().await;
        current.remove(&chat_id);
    }

    {
        let mut queues = state.queues.lock().await;

        if let Some(queue) = queues.get_mut(&chat_id) {
            queue.clear();
        }
    }

    println!("🧹 Playback and queue cleared");

    Ok(Json(StopResponse {
        status: "stopped".to_string(),
    }))
}

async fn queue(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(chat_id): Path<i64>,
) -> Result<
    Json<QueueResponse>,
    (StatusCode, Json<serde_json::Value>),
> {
    if !authorized(&headers, &state.worker_secret) {
        return Err(error(
            StatusCode::UNAUTHORIZED,
            "Unauthorized",
        ));
    }

    let current_song = {
        let current = state.current.lock().await;
        current.get(&chat_id).cloned()
    };

    let queued_songs = {
        let queues = state.queues.lock().await;

        queues
            .get(&chat_id)
            .map(|queue| queue.iter().cloned().collect::<Vec<_>>())
            .unwrap_or_default()
    };

    Ok(Json(QueueResponse {
        current: current_song.map(|song| SongInfo {
            title: song.title,
        }),
        queue: queued_songs
            .into_iter()
            .map(|song| SongInfo {
                title: song.title,
            })
            .collect(),
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
