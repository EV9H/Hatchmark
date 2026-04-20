use crate::model::Channel;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DaemonMsg {
    Hello {
        version: String,
        current_layer_id: i64,
    },
    Increment {
        channel_id: i64,
        new_total_today: i64,
        timestamp: String,
    },
    LayerChanged {
        current_layer_id: i64,
    },
    BindingConflict {
        layer_id: i64,
        key_code: String,
        reason: String,
    },
    ChannelsUpdated {
        channels: Vec<Channel>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UiMsg {
    Subscribe,
    ReloadBindings,
    SwitchLayer { target_layer_id: i64 },
    Ping,
}

pub const IPC_HOST: &str = "127.0.0.1";
pub const IPC_PROTO_VERSION: &str = "1";
