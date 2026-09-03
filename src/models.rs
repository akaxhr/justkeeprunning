use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct PlayRequest {
    pub chat_id: i64,
    pub query: String,
    pub quality: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SongInfo {
    pub title: String,
}

#[derive(Debug, Serialize)]
pub struct PlayResponse {
    pub status: String,
    pub song: SongInfo,
}

