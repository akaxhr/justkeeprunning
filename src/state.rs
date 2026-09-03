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
}

impl ChatQueue {
    pub fn new() -> Self {
        Self {
            current: None,
            queue: VecDeque::new(),
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub calls: Arc<Calls>,
    pub worker_secret: String,

    pub queues: Arc<Mutex<HashMap<i64, ChatQueue>>>,
}
