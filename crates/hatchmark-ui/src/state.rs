use hatchmark_core::{db::Db, paths::AppPaths};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Mutex<Db>>,
    pub paths: AppPaths,
    pub daemon_port: Arc<Mutex<Option<u16>>>,
}
