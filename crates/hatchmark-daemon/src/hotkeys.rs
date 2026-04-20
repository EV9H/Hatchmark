use crate::toast::{self, ToastEvent};
use anyhow::{Context, Result};
use hatchmark_core::db::{bindings, events, settings};
use hatchmark_core::ipc::DaemonMsg;
use hatchmark_core::model::BindingAction;
use hatchmark_core::{db::Db, model::Binding};
use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use std::collections::HashMap;
use std::sync::mpsc::Sender;

pub struct HotkeyManager {
    mgr: GlobalHotKeyManager,
    by_id: HashMap<u32, String>,
    current_layer: i64,
}

impl HotkeyManager {
    pub fn new() -> Result<Self> {
        Ok(Self {
            mgr: GlobalHotKeyManager::new().context("creating GlobalHotKeyManager")?,
            by_id: HashMap::new(),
            current_layer: 0,
        })
    }

    pub fn current_layer(&self) -> i64 {
        self.current_layer
    }

    /// Re-register hotkeys for `layer_id`. Returns a list of `(key_code, reason)`
    /// for bindings that couldn't be registered (collision with another app or
    /// unrecognised key name).
    pub fn load_layer(&mut self, db: &Db, layer_id: i64) -> Result<Vec<(String, String)>> {
        // Cheapest way to drop all prior registrations: recreate the manager.
        self.mgr = GlobalHotKeyManager::new()?;
        self.by_id.clear();

        let mut conflicts = Vec::new();
        let bs: Vec<Binding> = bindings::list_for_layer(&db.conn, layer_id)?;
        for b in bs {
            let hk = match parse_key(&b.key_code) {
                Some(hk) => hk,
                None => {
                    conflicts.push((b.key_code.clone(), "unknown key".to_string()));
                    continue;
                }
            };
            match self.mgr.register(hk) {
                Ok(()) => {
                    self.by_id.insert(hk.id(), b.key_code);
                }
                Err(e) => conflicts.push((b.key_code, format!("{e}"))),
            }
        }
        self.current_layer = layer_id;
        Ok(conflicts)
    }

    pub fn on_event(
        &mut self,
        db: &Db,
        broadcaster: &Sender<DaemonMsg>,
        toast_tx: &Sender<ToastEvent>,
        ev: GlobalHotKeyEvent,
    ) -> Result<()> {
        if ev.state != HotKeyState::Pressed {
            return Ok(());
        }
        let Some(key_code) = self.by_id.get(&ev.id).cloned() else {
            return Ok(());
        };
        let Some(action) = bindings::resolve(&db.conn, self.current_layer, &key_code)? else {
            return Ok(());
        };
        match action {
            BindingAction::Increment { channel_id } => {
                let ts = chrono::Utc::now().to_rfc3339();
                let total = events::insert(&db.conn, channel_id)?;
                toast::maybe_send(db, channel_id, total, toast_tx);
                let _ = broadcaster.send(DaemonMsg::Increment {
                    channel_id,
                    new_total_today: total,
                    timestamp: ts,
                });
            }
            BindingAction::SwitchLayer { target_layer_id } => {
                settings::set_current_layer_id(&db.conn, target_layer_id)?;
                let conflicts = self.load_layer(db, target_layer_id)?;
                for (key, reason) in conflicts {
                    let _ = broadcaster.send(DaemonMsg::BindingConflict {
                        layer_id: target_layer_id,
                        key_code: key,
                        reason,
                    });
                }
                let _ = broadcaster.send(DaemonMsg::LayerChanged {
                    current_layer_id: target_layer_id,
                });
            }
        }
        Ok(())
    }
}

fn parse_key(key_code: &str) -> Option<HotKey> {
    let code = match key_code {
        "F13" => Code::F13,
        "F14" => Code::F14,
        "F15" => Code::F15,
        "F16" => Code::F16,
        "F17" => Code::F17,
        "F18" => Code::F18,
        "F19" => Code::F19,
        "F20" => Code::F20,
        "F21" => Code::F21,
        "F22" => Code::F22,
        "F23" => Code::F23,
        "F24" => Code::F24,
        _ => return None,
    };
    Some(HotKey::new(Some(Modifiers::empty()), code))
}
