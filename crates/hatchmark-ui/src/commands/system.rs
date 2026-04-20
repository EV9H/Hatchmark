use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub fn reveal_data_dir(state: State<'_, AppState>) -> Result<String, String> {
    Ok(state.paths.data_dir.to_string_lossy().into_owned())
}

#[tauri::command]
pub fn reload_daemon(state: State<'_, AppState>) -> Result<(), String> {
    crate::daemon_client::send_reload(&state).map_err(|e| e.to_string())
}
