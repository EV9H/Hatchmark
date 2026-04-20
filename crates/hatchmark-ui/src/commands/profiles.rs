use crate::state::AppState;
use hatchmark_core::db::Db;
use hatchmark_core::ipc::UiMsg;
use std::io::Write;
use std::net::TcpStream;
use tauri::State;

#[tauri::command]
pub fn list_profiles(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    state.paths.list_profiles().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_active_profile(state: State<'_, AppState>) -> Result<String, String> {
    state.paths.active_profile().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_profile(state: State<'_, AppState>, name: String) -> Result<(), String> {
    state
        .paths
        .create_profile(&name)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_profile(state: State<'_, AppState>, name: String) -> Result<(), String> {
    state.paths.delete_profile(&name).map_err(|e| e.to_string())
}

/// Tell the daemon to switch profile, then reopen our own DB handle so
/// subsequent Tauri commands read from the new profile too.
#[tauri::command]
pub fn switch_profile(state: State<'_, AppState>, name: String) -> Result<(), String> {
    // Persist the choice up front — in case the daemon isn't running, the
    // next restart still picks the right profile.
    state
        .paths
        .set_active_profile(&name)
        .map_err(|e| e.to_string())?;

    // Best-effort notify the daemon (it will reload its DB + hotkeys).
    if let Some(port) = *state.daemon_port.lock().map_err(|e| e.to_string())? {
        let _ = notify_daemon_switch(port, &name);
    }

    // Reopen our own DB handle to point at the new profile.
    let new_path = state.paths.profile_db(&name);
    let new_db = Db::open(&new_path).map_err(|e| e.to_string())?;
    *state.db.lock().map_err(|e| e.to_string())? = new_db;
    Ok(())
}

/// Clear all events (counts) for the currently active profile. Keeps channels,
/// layers, and bindings intact — just zeros the counters.
#[tauri::command]
pub fn clear_events(state: State<'_, AppState>) -> Result<usize, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let n = db
        .conn
        .execute("DELETE FROM events", [])
        .map_err(|e| e.to_string())?;
    Ok(n)
}

fn notify_daemon_switch(port: u16, name: &str) -> std::io::Result<()> {
    let mut s = TcpStream::connect(("127.0.0.1", port))?;
    let mut msg = serde_json::to_string(&UiMsg::SwitchProfile {
        name: name.to_string(),
    })
    .map_err(std::io::Error::other)?;
    msg.push('\n');
    s.write_all(msg.as_bytes())?;
    Ok(())
}
