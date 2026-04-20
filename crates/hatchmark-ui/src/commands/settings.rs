use crate::state::AppState;
use hatchmark_core::db::settings;
use tauri::State;

#[tauri::command]
pub fn get_setting(state: State<'_, AppState>, key: String) -> Result<Option<String>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    settings::get(&db.conn, &key).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_setting(state: State<'_, AppState>, key: String, value: String) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    settings::set(&db.conn, &key, &value).map_err(|e| e.to_string())
}
