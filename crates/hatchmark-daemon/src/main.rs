#![cfg_attr(all(target_os = "windows", not(debug_assertions)), windows_subsystem = "windows")]

mod autostart;
mod hotkeys;
mod ipc_server;
mod shutdown;
mod single_instance;
mod toast;
mod tray;
mod ui_launcher;

use anyhow::Result;
use hatchmark_core::{db::Db, paths::AppPaths};
use tracing::info;

fn main() -> Result<()> {
    init_logging()?;
    let paths = AppPaths::resolve()?;
    let _guard = match single_instance::acquire() {
        Ok(g) => g,
        Err(e) => {
            eprintln!("daemon: {e}");
            return Ok(());
        }
    };

    let db_path = paths.active_profile_db()?;
    info!("opening db at {}", db_path.display());
    let db = Db::open(&db_path)?;

    if let Some(enabled) = hatchmark_core::db::settings::get_bool(&db.conn, "autostart")? {
        if let Err(e) = autostart::set_enabled(enabled) {
            tracing::warn!("could not sync autostart: {e}");
        }
    }

    tray::run_event_loop(paths, db)
}

fn init_logging() -> Result<()> {
    use tracing_subscriber::EnvFilter;
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with_writer(std::io::stderr)
        .try_init();
    Ok(())
}
