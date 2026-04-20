use crate::model::Layer;
use crate::{CoreError, Result};
use rusqlite::{params, Connection};

pub fn list(conn: &Connection) -> Result<Vec<Layer>> {
    let mut stmt = conn.prepare("SELECT id, name, sort_order FROM layers ORDER BY sort_order, id")?;
    let rows = stmt
        .query_map([], |r| {
            Ok(Layer {
                id: r.get(0)?,
                name: r.get(1)?,
                sort_order: r.get(2)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn get(conn: &Connection, id: i64) -> Result<Layer> {
    conn.query_row(
        "SELECT id, name, sort_order FROM layers WHERE id=?1",
        params![id],
        |r| {
            Ok(Layer {
                id: r.get(0)?,
                name: r.get(1)?,
                sort_order: r.get(2)?,
            })
        },
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => CoreError::NotFound { kind: "layer", id },
        other => other.into(),
    })
}

pub fn create(conn: &Connection, name: &str) -> Result<i64> {
    let next_sort: i64 = conn.query_row(
        "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM layers",
        [],
        |r| r.get(0),
    )?;
    conn.execute(
        "INSERT INTO layers (name, sort_order) VALUES (?1, ?2)",
        params![name, next_sort],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn rename(conn: &Connection, id: i64, name: &str) -> Result<()> {
    let n = conn.execute("UPDATE layers SET name=?1 WHERE id=?2", params![name, id])?;
    if n == 0 {
        return Err(CoreError::NotFound { kind: "layer", id });
    }
    Ok(())
}

pub fn delete(conn: &Connection, id: i64) -> Result<()> {
    let remaining: i64 = conn.query_row("SELECT COUNT(*) FROM layers", [], |r| r.get(0))?;
    if remaining <= 1 {
        return Err(CoreError::Invalid("cannot delete the last layer".into()));
    }
    let n = conn.execute("DELETE FROM layers WHERE id=?1", params![id])?;
    if n == 0 {
        return Err(CoreError::NotFound { kind: "layer", id });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Db;

    #[test]
    fn default_layer_present_and_listable() {
        let db = Db::open_memory().unwrap();
        let layers = list(&db.conn).unwrap();
        assert_eq!(layers.len(), 1);
        assert_eq!(layers[0].name, "Default");
    }

    #[test]
    fn cannot_delete_last_layer() {
        let db = Db::open_memory().unwrap();
        let err = delete(&db.conn, 1).unwrap_err();
        assert!(matches!(err, CoreError::Invalid(_)));
    }

    #[test]
    fn create_and_rename() {
        let db = Db::open_memory().unwrap();
        let id = create(&db.conn, "Work").unwrap();
        rename(&db.conn, id, "Office").unwrap();
        assert_eq!(get(&db.conn, id).unwrap().name, "Office");
    }
}
