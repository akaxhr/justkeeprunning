use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct PlayRequest {
    pub chat_id: i64,
    pub query: String,
    pub quality: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct SongInfo {
    pub title: String,
    pub artist: String,
    pub duration: u64,
    pub thumbnail: Option<String>,
    pub url: String,
}

#[derive(Debug, Serialize)]
pub struct PlayResponse {
    pub status: String,
    pub song: SongInfo,
}
