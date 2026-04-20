use crate::model::{Binding, BindingAction};
use crate::Result;
use rusqlite::{params, Connection};

pub fn list_for_layer(conn: &Connection, layer_id: i64) -> Result<Vec<Binding>> {
    let mut stmt = conn.prepare(
        "SELECT layer_id, key_code, action, channel_id, target_layer_id
           FROM bindings WHERE layer_id = ?1 ORDER BY key_code",
    )?;
    let rows = stmt
        .query_map(params![layer_id], row_to_binding)?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn upsert(conn: &Connection, b: &Binding) -> Result<()> {
    let (action_str, channel_id, target_layer_id) = match b.action {
        BindingAction::Increment { channel_id } => ("increment", Some(channel_id), None),
        BindingAction::SwitchLayer { target_layer_id } => {
            ("switch_layer", None, Some(target_layer_id))
        }
    };
    conn.execute(
        "INSERT INTO bindings (layer_id, key_code, action, channel_id, target_layer_id)
              VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(layer_id, key_code) DO UPDATE SET
              action=excluded.action,
              channel_id=excluded.channel_id,
              target_layer_id=excluded.target_layer_id",
        params![
            b.layer_id,
            b.key_code,
            action_str,
            channel_id,
            target_layer_id
        ],
    )?;
    Ok(())
}

pub fn delete(conn: &Connection, layer_id: i64, key_code: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM bindings WHERE layer_id=?1 AND key_code=?2",
        params![layer_id, key_code],
    )?;
    Ok(())
}

pub fn resolve(
    conn: &Connection,
    layer_id: i64,
    key_code: &str,
) -> Result<Option<BindingAction>> {
    let result = conn.query_row(
        "SELECT layer_id, key_code, action, channel_id, target_layer_id
           FROM bindings WHERE layer_id=?1 AND key_code=?2",
        params![layer_id, key_code],
        row_to_binding,
    );
    match result {
        Ok(b) => Ok(Some(b.action)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

fn row_to_binding(r: &rusqlite::Row<'_>) -> rusqlite::Result<Binding> {
    let layer_id: i64 = r.get(0)?;
    let key_code: String = r.get(1)?;
    let action_str: String = r.get(2)?;
    let channel_id: Option<i64> = r.get(3)?;
    let target_layer_id: Option<i64> = r.get(4)?;
    let action = match action_str.as_str() {
        "increment" => BindingAction::Increment {
            channel_id: channel_id.ok_or(rusqlite::Error::InvalidQuery)?,
        },
        "switch_layer" => BindingAction::SwitchLayer {
            target_layer_id: target_layer_id.ok_or(rusqlite::Error::InvalidQuery)?,
        },
        other => {
            return Err(rusqlite::Error::FromSqlConversionFailure(
                2,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("unknown binding action {other}"),
                )),
            ));
        }
    };
    Ok(Binding {
        layer_id,
        key_code,
        action,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{channels, layers, Db};

    #[test]
    fn resolve_returns_none_for_unbound_key() {
        let db = Db::open_memory().unwrap();
        assert!(resolve(&db.conn, 1, "F13").unwrap().is_none());
    }

    #[test]
    fn upsert_then_resolve_increment() {
        let db = Db::open_memory().unwrap();
        let cid = channels::create(&db.conn, "Water", "#7dd3fc", None, None).unwrap();
        upsert(
            &db.conn,
            &Binding {
                layer_id: 1,
                key_code: "F13".into(),
                action: BindingAction::Increment { channel_id: cid },
            },
        )
        .unwrap();
        let action = resolve(&db.conn, 1, "F13").unwrap().unwrap();
        assert!(matches!(action, BindingAction::Increment { channel_id } if channel_id == cid));
    }

    #[test]
    fn upsert_replaces_existing_binding() {
        let db = Db::open_memory().unwrap();
        let c1 = channels::create(&db.conn, "Water", "#7dd3fc", None, None).unwrap();
        let c2 = channels::create(&db.conn, "Vape", "#f87171", None, None).unwrap();
        upsert(
            &db.conn,
            &Binding {
                layer_id: 1,
                key_code: "F13".into(),
                action: BindingAction::Increment { channel_id: c1 },
            },
        )
        .unwrap();
        upsert(
            &db.conn,
            &Binding {
                layer_id: 1,
                key_code: "F13".into(),
                action: BindingAction::Increment { channel_id: c2 },
            },
        )
        .unwrap();
        let action = resolve(&db.conn, 1, "F13").unwrap().unwrap();
        assert!(matches!(action, BindingAction::Increment { channel_id } if channel_id == c2));
    }

    #[test]
    fn switch_layer_binding_roundtrips() {
        let db = Db::open_memory().unwrap();
        let other = layers::create(&db.conn, "Work").unwrap();
        upsert(
            &db.conn,
            &Binding {
                layer_id: 1,
                key_code: "F18".into(),
                action: BindingAction::SwitchLayer {
                    target_layer_id: other,
                },
            },
        )
        .unwrap();
        let action = resolve(&db.conn, 1, "F18").unwrap().unwrap();
        assert!(
            matches!(action, BindingAction::SwitchLayer { target_layer_id } if target_layer_id == other)
        );
    }
}
