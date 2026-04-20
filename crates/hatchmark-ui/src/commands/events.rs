use crate::state::AppState;
use hatchmark_core::db::events;
use std::fs::File;
use std::io::BufWriter;
use tauri::State;

#[tauri::command]
pub fn adjust(state: State<'_, AppState>, channel_id: i64, delta: i64) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    match delta {
        1 => {
            events::insert(&db.conn, channel_id).map_err(|e| e.to_string())?;
            Ok(())
        }
        -1 => {
            events::delete_last_for_channel(&db.conn, channel_id).map_err(|e| e.to_string())?;
            Ok(())
        }
        _ => Err("only +1/-1 supported".into()),
    }
}

#[tauri::command]
pub fn export_csv(state: State<'_, AppState>, path: String) -> Result<usize, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let file = File::create(&path).map_err(|e| e.to_string())?;
    events::export_all_to_writer(&db.conn, BufWriter::new(file)).map_err(|e| e.to_string())
}
