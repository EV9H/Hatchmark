use crate::state::AppState;
use hatchmark_core::db::bindings;
use hatchmark_core::model::Binding;
use tauri::State;

#[tauri::command]
pub fn list_bindings(state: State<'_, AppState>, layer_id: i64) -> Result<Vec<Binding>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    bindings::list_for_layer(&db.conn, layer_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn upsert_binding(state: State<'_, AppState>, binding: Binding) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    bindings::upsert(&db.conn, &binding).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_binding(
    state: State<'_, AppState>,
    layer_id: i64,
    key_code: String,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    bindings::delete(&db.conn, layer_id, &key_code).map_err(|e| e.to_string())
}
