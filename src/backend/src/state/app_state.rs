use std::sync::Arc;

use crate::runtime::store::RuntimeStore;

#[derive(Clone)]
pub struct AppState {
    pub store: Arc<RuntimeStore>,
    pub backend_url: String,
}
