use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Channel {
    pub id: i64,
    pub name: String,
    pub color: String,
    pub daily_goal: Option<i64>,
    pub daily_limit: Option<i64>,
    pub sort_order: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Layer {
    pub id: i64,
    pub name: String,
    pub sort_order: i64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum BindingAction {
    Increment { channel_id: i64 },
    SwitchLayer { target_layer_id: i64 },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Binding {
    pub layer_id: i64,
    pub key_code: String,
    pub action: BindingAction,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Event {
    pub id: i64,
    pub channel_id: i64,
    pub timestamp: String,
}
