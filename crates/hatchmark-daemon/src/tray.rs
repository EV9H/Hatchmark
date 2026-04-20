use crate::hotkeys::HotkeyManager;
use crate::toast::{self, ToastEvent};
use anyhow::Result;
use hatchmark_core::db::{layers, settings, Db};
use hatchmark_core::ipc::DaemonMsg;
use hatchmark_core::paths::AppPaths;
use global_hotkey::GlobalHotKeyEvent;
use std::sync::mpsc::{channel, Receiver, Sender};
use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu};
use tray_icon::{TrayIconBuilder, TrayIconEvent};
use winit::application::ApplicationHandler;
use winit::event::StartCause;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};

const MENU_ID_SHOW: &str = "show";
const MENU_ID_QUIT: &str = "quit";
const MENU_ID_LAYER_PREFIX: &str = "layer:";

pub fn run_event_loop(paths: AppPaths, db: Db) -> Result<()> {
    let event_loop = EventLoop::builder().build()?;
    event_loop.set_control_flow(ControlFlow::Wait);

    let (broadcast_tx, broadcast_rx) = channel::<DaemonMsg>();
    let (reload_tx, reload_rx) = channel::<()>();

    let mut hotkeys = HotkeyManager::new()?;
    let current = settings::current_layer_id(&db.conn)?;
    let conflicts = hotkeys.load_layer(&db, current)?;
    for (key, reason) in conflicts {
        let _ = broadcast_tx.send(DaemonMsg::BindingConflict {
            layer_id: current,
            key_code: key,
            reason,
        });
    }

    // IPC server takes ownership of broadcast_rx.
    crate::ipc_server::spawn(paths.clone(), broadcast_rx, reload_tx)?;

    let toast_tx = toast::spawn();

    let mut app = App {
        tray: None,
        paths,
        db,
        hotkeys,
        broadcast_tx,
        reload_rx,
        toast_tx,
    };
    event_loop.run_app(&mut app)?;
    Ok(())
}

struct App {
    tray: Option<tray_icon::TrayIcon>,
    paths: AppPaths,
    db: Db,
    hotkeys: HotkeyManager,
    broadcast_tx: Sender<DaemonMsg>,
    reload_rx: Receiver<()>,
    toast_tx: Sender<ToastEvent>,
}

impl ApplicationHandler for App {
    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {
        if matches!(cause, StartCause::Init) {
            if let Err(e) = self.install_tray() {
                eprintln!("tray init failed: {e}");
                event_loop.exit();
                return;
            }
        }

        while let Ok(ev) = TrayIconEvent::receiver().try_recv() {
            if let TrayIconEvent::Click {
                button: tray_icon::MouseButton::Left,
                button_state: tray_icon::MouseButtonState::Up,
                ..
            } = ev
            {
                crate::ui_launcher::spawn(&self.paths);
            }
        }

        while let Ok(ev) = MenuEvent::receiver().try_recv() {
            self.handle_menu(event_loop, ev);
        }

        while let Ok(ev) = GlobalHotKeyEvent::receiver().try_recv() {
            if let Err(e) =
                self.hotkeys
                    .on_event(&self.db, &self.broadcast_tx, &self.toast_tx, ev)
            {
                tracing::warn!("hotkey handler error: {e}");
            }
        }

        while self.reload_rx.try_recv().is_ok() {
            let layer = settings::current_layer_id(&self.db.conn).unwrap_or(1);
            match self.hotkeys.load_layer(&self.db, layer) {
                Ok(conflicts) => {
                    for (key, reason) in conflicts {
                        let _ = self.broadcast_tx.send(DaemonMsg::BindingConflict {
                            layer_id: layer,
                            key_code: key,
                            reason,
                        });
                    }
                    if let Err(e) = self.install_tray() {
                        tracing::warn!("refresh tray after reload: {e}");
                    }
                }
                Err(e) => tracing::warn!("reload bindings: {e}"),
            }
        }
    }

    fn resumed(&mut self, _: &ActiveEventLoop) {}
    fn window_event(
        &mut self,
        _: &ActiveEventLoop,
        _: winit::window::WindowId,
        _: winit::event::WindowEvent,
    ) {
    }
}

impl App {
    fn install_tray(&mut self) -> Result<()> {
        let icon = load_icon();
        let menu = Menu::new();
        menu.append(&MenuItem::with_id(
            MENU_ID_SHOW,
            "Show analytics",
            true,
            None,
        ))?;
        let layers_menu = Submenu::new("Switch layer", true);
        let current = settings::current_layer_id(&self.db.conn)?;
        for layer in layers::list(&self.db.conn)? {
            let label = if layer.id == current {
                format!("● {}", layer.name)
            } else {
                format!("  {}", layer.name)
            };
            let id = format!("{MENU_ID_LAYER_PREFIX}{}", layer.id);
            layers_menu.append(&MenuItem::with_id(id, label, true, None))?;
        }
        menu.append(&layers_menu)?;
        menu.append(&PredefinedMenuItem::separator())?;
        menu.append(&MenuItem::with_id(MENU_ID_QUIT, "Quit", true, None))?;

        let tray = TrayIconBuilder::new()
            .with_tooltip("Hatchmark")
            .with_icon(icon)
            .with_menu(Box::new(menu))
            .build()?;
        self.tray = Some(tray);
        Ok(())
    }

    fn handle_menu(&mut self, event_loop: &ActiveEventLoop, ev: MenuEvent) {
        let id = ev.id.0.as_str();
        match id {
            MENU_ID_SHOW => crate::ui_launcher::spawn(&self.paths),
            MENU_ID_QUIT => {
                let _ = crate::shutdown::checkpoint_wal(&self.db.conn);
                let _ = self.toast_tx.send(ToastEvent::Quit);
                event_loop.exit();
            }
            s if s.starts_with(MENU_ID_LAYER_PREFIX) => {
                if let Ok(target) = s[MENU_ID_LAYER_PREFIX.len()..].parse::<i64>() {
                    if let Err(e) = settings::set_current_layer_id(&self.db.conn, target) {
                        tracing::warn!("set current layer: {e}");
                        return;
                    }
                    if let Ok(conflicts) = self.hotkeys.load_layer(&self.db, target) {
                        for (key, reason) in conflicts {
                            let _ = self.broadcast_tx.send(DaemonMsg::BindingConflict {
                                layer_id: target,
                                key_code: key,
                                reason,
                            });
                        }
                    }
                    let _ = self.broadcast_tx.send(DaemonMsg::LayerChanged {
                        current_layer_id: target,
                    });
                    if let Err(e) = self.install_tray() {
                        tracing::warn!("refresh tray after layer switch: {e}");
                    }
                }
            }
            _ => {}
        }
    }
}

fn load_icon() -> tray_icon::Icon {
    let bytes = include_bytes!("../resources/icon.ico");
    let image = image::load_from_memory(bytes)
        .expect("icon decode")
        .into_rgba8();
    let (w, h) = image.dimensions();
    tray_icon::Icon::from_rgba(image.into_raw(), w, h).expect("icon build")
}
