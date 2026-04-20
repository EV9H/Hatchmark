use crate::state::AppState;
use hatchmark_core::db::{layers, settings};
use hatchmark_core::model::Layer;
use tauri::State;

#[tauri::command]
pub fn list_layers(state: State<'_, AppState>) -> Result<Vec<Layer>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    layers::list(&db.conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_layer(state: State<'_, AppState>, name: String) -> Result<i64, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    layers::create(&db.conn, &name).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn rename_layer(state: State<'_, AppState>, id: i64, name: String) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    layers::rename(&db.conn, id, &name).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_layer(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    layers::delete(&db.conn, id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_current_layer(state: State<'_, AppState>) -> Result<i64, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    settings::current_layer_id(&db.conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_current_layer(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    settings::set_current_layer_id(&db.conn, id).map_err(|e| e.to_string())
}
