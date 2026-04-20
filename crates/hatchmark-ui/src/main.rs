#![cfg_attr(all(target_os = "windows", not(debug_assertions)), windows_subsystem = "windows")]

mod commands;
mod daemon_client;
mod state;

use anyhow::Result;
use hatchmark_core::{db::Db, paths::AppPaths};
use state::AppState;
use std::sync::{Arc, Mutex};
use tauri::Manager;

#[cfg(windows)]
use windows::core::HSTRING;
#[cfg(windows)]
use windows::Win32::Foundation::{CloseHandle, GetLastError, ERROR_ALREADY_EXISTS};
#[cfg(windows)]
use windows::Win32::System::Threading::CreateMutexW;

fn main() -> Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")))
        .with_writer(std::io::stderr)
        .try_init();

    #[cfg(windows)]
    if !acquire_ui_mutex() {
        eprintln!("UI already running.");
        return Ok(());
    }

    let paths = AppPaths::resolve()?;
    let db = Db::open(&paths.active_profile_db()?)?;
    let state = AppState {
        db: Arc::new(Mutex::new(db)),
        paths: paths.clone(),
        daemon_port: Arc::new(Mutex::new(None)),
    };
    let state_for_setup = state.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(state)
        .setup(move |app| {
            let _ = daemon_client::GLOBAL_HANDLE.set(app.handle().clone());
            daemon_client::spawn(state_for_setup.clone());
            #[cfg(windows)]
            {
                if let Some(win) = app.get_webview_window("main") {
                    apply_mica(&win);
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::channels::list_channels,
            commands::channels::create_channel,
            commands::channels::update_channel,
            commands::channels::delete_channel,
            commands::layers::list_layers,
            commands::layers::create_layer,
            commands::layers::rename_layer,
            commands::layers::delete_layer,
            commands::layers::get_current_layer,
            commands::layers::set_current_layer,
            commands::bindings::list_bindings,
            commands::bindings::upsert_binding,
            commands::bindings::delete_binding,
            commands::analytics::today_counts,
            commands::analytics::history,
            commands::analytics::heatmap,
            commands::analytics::hourly,
            commands::analytics::rollup,
            commands::events::adjust,
            commands::events::export_csv,
            commands::settings::get_setting,
            commands::settings::set_setting,
            commands::system::reveal_data_dir,
            commands::system::reload_daemon,
            commands::profiles::list_profiles,
            commands::profiles::get_active_profile,
            commands::profiles::create_profile,
            commands::profiles::delete_profile,
            commands::profiles::switch_profile,
            commands::profiles::clear_events,
        ])
        .run(tauri::generate_context!())
        .expect("tauri run");
    Ok(())
}

#[cfg(windows)]
fn acquire_ui_mutex() -> bool {
    use windows::core::PCWSTR;
    let name = HSTRING::from("Global\\HatchmarkUI");
    unsafe {
        let handle = match CreateMutexW(None, true, PCWSTR(name.as_ptr())) {
            Ok(h) => h,
            Err(_) => return false,
        };
        let err = GetLastError();
        if err == ERROR_ALREADY_EXISTS {
            let _ = CloseHandle(handle);
            return false;
        }
        // Leak the handle intentionally; it is released when the process exits.
        std::mem::forget(handle);
        true
    }
}

#[cfg(windows)]
fn apply_mica(window: &tauri::WebviewWindow) {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::Graphics::Dwm::{
        DwmSetWindowAttribute, DWMSBT_MAINWINDOW, DWMWA_SYSTEMBACKDROP_TYPE,
    };
    if let Ok(raw) = window.hwnd() {
        let hwnd = HWND(raw.0 as _);
        let backdrop = DWMSBT_MAINWINDOW.0;
        unsafe {
            let _ = DwmSetWindowAttribute(
                hwnd,
                DWMWA_SYSTEMBACKDROP_TYPE,
                &backdrop as *const _ as _,
                std::mem::size_of_val(&backdrop) as u32,
            );
        }
    }
}
