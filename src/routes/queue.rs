use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};

use crate::{
    auth::authorized,
    models::{QueueItemResponse, QueueResponse},
    state::AppState,
};

pub async fn queue(
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

    let queues = state.queues.lock().await;

    let Some(chat_queue) = queues.get(&chat_id) else {
        return Ok(Json(QueueResponse {
            current: None,
            queue: Vec::new(),
        }));
    };

    let current = chat_queue
        .current
        .as_ref()
        .map(|item| item.query.clone());

    let queue = chat_queue
        .queue
        .iter()
        .enumerate()
        .map(|(index, item)| QueueItemResponse {
            position: index + 1,
            query: item.query.clone(),
        })
        .collect();

    Ok(Json(QueueResponse {
        current,
        queue,
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
