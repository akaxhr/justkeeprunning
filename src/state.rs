use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

use tokio::sync::Mutex;
use tgcalls::Calls;

#[derive(Clone)]
pub struct QueueItem {
    pub query: String,
}

pub struct ChatQueue {
    pub current: Option<QueueItem>,
    pub queue: VecDeque<QueueItem>,

    // Changes whenever the queue lifecycle is reset.
    // Prevents old background tasks from starting playback
    // after the VC has ended.
    pub generation: u64,
}

impl ChatQueue {
    pub fn new() -> Self {
        Self {
            current: None,
            queue: VecDeque::new(),
            generation: 0,
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub calls: Arc<Calls>,
    pub worker_secret: String,

    pub queues: Arc<Mutex<HashMap<i64, ChatQueue>>>,
}
