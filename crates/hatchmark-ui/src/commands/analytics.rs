use crate::state::AppState;
use hatchmark_core::analytics::{self, DailyCount, DateCount, HourCount, Rollup};
use tauri::State;

#[tauri::command]
pub fn today_counts(state: State<'_, AppState>) -> Result<Vec<DailyCount>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    analytics::today_per_channel(&db.conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn history(
    state: State<'_, AppState>,
    channel_id: i64,
    days: i64,
) -> Result<Vec<DateCount>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    analytics::history(&db.conn, channel_id, days).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn heatmap(
    state: State<'_, AppState>,
    channel_id: i64,
    days: i64,
) -> Result<Vec<DateCount>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    analytics::heatmap(&db.conn, channel_id, days).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn hourly(
    state: State<'_, AppState>,
    channel_id: i64,
    from_date: String,
    to_date: String,
) -> Result<Vec<HourCount>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    analytics::hourly(&db.conn, channel_id, &from_date, &to_date).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn rollup(
    state: State<'_, AppState>,
    channel_id: i64,
    days: i64,
) -> Result<Rollup, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    analytics::rollup(&db.conn, channel_id, days).map_err(|e| e.to_string())
}
