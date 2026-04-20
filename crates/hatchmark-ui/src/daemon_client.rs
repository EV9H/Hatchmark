use crate::state::AppState;
use hatchmark_core::ipc::{DaemonMsg, UiMsg};
use serde::Deserialize;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::time::Duration;
use tauri::Emitter;

#[derive(Deserialize)]
struct DaemonStateFile {
    port: u16,
}

pub static GLOBAL_HANDLE: std::sync::OnceLock<tauri::AppHandle> = std::sync::OnceLock::new();

pub fn spawn(state: AppState) {
    std::thread::Builder::new()
        .name("daemon-client".into())
        .spawn(move || loop {
            match connect_once(&state) {
                Ok(()) => tracing::info!("daemon connection closed"),
                Err(e) => tracing::debug!("daemon connection: {e}"),
            }
            std::thread::sleep(Duration::from_millis(1500));
        })
        .expect("spawn daemon client");
}

fn connect_once(state: &AppState) -> anyhow::Result<()> {
    let raw = std::fs::read(&state.paths.state_path)?;
    let ds: DaemonStateFile = serde_json::from_slice(&raw)?;
    *state.daemon_port.lock().unwrap() = Some(ds.port);

    let stream = TcpStream::connect(("127.0.0.1", ds.port))?;
    stream.set_read_timeout(Some(Duration::from_secs(120)))?;
    let mut writer = stream.try_clone()?;
    let mut reader = BufReader::new(stream);

    let hello = serde_json::to_string(&UiMsg::Subscribe)?;
    writer.write_all(hello.as_bytes())?;
    writer.write_all(b"\n")?;

    let mut line = String::new();
    loop {
        line.clear();
        let n = reader.read_line(&mut line)?;
        if n == 0 {
            break;
        }
        match serde_json::from_str::<DaemonMsg>(line.trim()) {
            Ok(msg) => emit_to_frontend(&msg),
            Err(e) => tracing::debug!("parse daemon msg: {e}: {line}"),
        }
    }
    Ok(())
}

fn emit_to_frontend(msg: &DaemonMsg) {
    if let Some(handle) = GLOBAL_HANDLE.get() {
        let _ = handle.emit("daemon-msg", msg);
    }
}

pub fn send_reload(state: &AppState) -> anyhow::Result<()> {
    let port = state
        .daemon_port
        .lock()
        .unwrap()
        .ok_or_else(|| anyhow::anyhow!("daemon port not known yet"))?;
    let mut s = TcpStream::connect(("127.0.0.1", port))?;
    let mut msg = serde_json::to_string(&UiMsg::ReloadBindings)?;
    msg.push('\n');
    s.write_all(msg.as_bytes())?;
    Ok(())
}
