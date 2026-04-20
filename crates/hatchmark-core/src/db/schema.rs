pub const V1: &str = r#"
CREATE TABLE IF NOT EXISTS channels (
    id           INTEGER PRIMARY KEY,
    name         TEXT    NOT NULL,
    color        TEXT    NOT NULL,
    daily_goal   INTEGER,
    daily_limit  INTEGER,
    sort_order   INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS layers (
    id         INTEGER PRIMARY KEY,
    name       TEXT    NOT NULL,
    sort_order INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS bindings (
    layer_id        INTEGER NOT NULL REFERENCES layers(id) ON DELETE CASCADE,
    key_code        TEXT    NOT NULL,
    action          TEXT    NOT NULL,
    channel_id      INTEGER REFERENCES channels(id) ON DELETE SET NULL,
    target_layer_id INTEGER REFERENCES layers(id)   ON DELETE SET NULL,
    PRIMARY KEY (layer_id, key_code)
);

CREATE TABLE IF NOT EXISTS events (
    id         INTEGER PRIMARY KEY,
    channel_id INTEGER NOT NULL REFERENCES channels(id) ON DELETE CASCADE,
    timestamp  TEXT    NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_events_channel_time ON events(channel_id, timestamp);
CREATE INDEX IF NOT EXISTS idx_events_time         ON events(timestamp);

CREATE TABLE IF NOT EXISTS settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

INSERT OR IGNORE INTO layers (id, name, sort_order) VALUES (1, 'Default', 0);
INSERT OR IGNORE INTO settings (key, value) VALUES ('schema_version', '1');
INSERT OR IGNORE INTO settings (key, value) VALUES ('current_layer_id', '1');
INSERT OR IGNORE INTO settings (key, value) VALUES ('toast_enabled', 'false');
INSERT OR IGNORE INTO settings (key, value) VALUES ('autostart', 'true');
"#;
