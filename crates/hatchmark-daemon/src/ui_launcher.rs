use hatchmark_core::paths::AppPaths;
use std::process::Command;

pub fn spawn(_paths: &AppPaths) {
    let exe = match locate_ui_exe() {
        Some(p) => p,
        None => {
            tracing::warn!("hatchmark-ui.exe not found next to the daemon");
            return;
        }
    };
    if let Err(e) = Command::new(&exe).spawn() {
        tracing::warn!("failed to launch UI at {}: {e}", exe.display());
    } else {
        tracing::info!("launched UI: {}", exe.display());
    }
}

fn locate_ui_exe() -> Option<std::path::PathBuf> {
    let daemon = std::env::current_exe().ok()?;
    let dir = daemon.parent()?.to_path_buf();
    let candidate = dir.join("hatchmark-ui.exe");
    if candidate.exists() {
        return Some(candidate);
    }
    let alt = dir.join("..").join("hatchmark-ui.exe");
    alt.exists().then_some(alt)
}
