use crate::state::AppState;
use hatchmark_core::db::channels;
use hatchmark_core::model::Channel;
use tauri::State;

#[tauri::command]
pub fn list_channels(state: State<'_, AppState>) -> Result<Vec<Channel>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    channels::list(&db.conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_channel(
    state: State<'_, AppState>,
    name: String,
    color: String,
    daily_goal: Option<i64>,
    daily_limit: Option<i64>,
) -> Result<i64, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    channels::create(&db.conn, &name, &color, daily_goal, daily_limit)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_channel(state: State<'_, AppState>, channel: Channel) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    channels::update(&db.conn, &channel).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_channel(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    channels::delete(&db.conn, id).map_err(|e| e.to_string())
}
