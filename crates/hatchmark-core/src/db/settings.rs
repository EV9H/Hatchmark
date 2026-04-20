use crate::{CoreError, Result};
use rusqlite::{params, Connection};

pub fn get(conn: &Connection, key: &str) -> Result<Option<String>> {
    let res = conn.query_row(
        "SELECT value FROM settings WHERE key=?1",
        params![key],
        |r| r.get::<_, String>(0),
    );
    match res {
        Ok(v) => Ok(Some(v)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn set(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO settings(key, value) VALUES(?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value=excluded.value",
        params![key, value],
    )?;
    Ok(())
}

pub fn get_i64(conn: &Connection, key: &str) -> Result<Option<i64>> {
    match get(conn, key)? {
        Some(v) => v
            .parse::<i64>()
            .map(Some)
            .map_err(|_| CoreError::Invalid(format!("setting {key} is not an integer: {v}"))),
        None => Ok(None),
    }
}

pub fn get_bool(conn: &Connection, key: &str) -> Result<Option<bool>> {
    Ok(get(conn, key)?.map(|v| matches!(v.as_str(), "true" | "1")))
}

pub fn current_layer_id(conn: &Connection) -> Result<i64> {
    get_i64(conn, "current_layer_id")?
        .ok_or_else(|| CoreError::Invalid("current_layer_id missing".into()))
}

pub fn set_current_layer_id(conn: &Connection, id: i64) -> Result<()> {
    set(conn, "current_layer_id", &id.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Db;

    #[test]
    fn defaults_seeded_by_migration() {
        let db = Db::open_memory().unwrap();
        assert_eq!(current_layer_id(&db.conn).unwrap(), 1);
        assert_eq!(get_bool(&db.conn, "toast_enabled").unwrap(), Some(false));
        assert_eq!(get_bool(&db.conn, "autostart").unwrap(), Some(true));
    }

    #[test]
    fn set_then_get_roundtrip() {
        let db = Db::open_memory().unwrap();
        set(&db.conn, "foo", "bar").unwrap();
        assert_eq!(get(&db.conn, "foo").unwrap().unwrap(), "bar");
        set(&db.conn, "foo", "baz").unwrap();
        assert_eq!(get(&db.conn, "foo").unwrap().unwrap(), "baz");
    }
}
