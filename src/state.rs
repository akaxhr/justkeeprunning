
use std::sync::Arc;
use tgcalls::Calls;

#[derive(Clone)]
pub struct AppState {
    pub calls: Arc<Calls>,
    pub worker_secret: String,
}

