use std::sync::Arc;

use crate::storage::file_store::FileStore;

pub struct AppContext {
    pub store: Arc<FileStore>,
}
