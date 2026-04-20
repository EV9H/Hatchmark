pub mod bindings;
pub mod channels;
pub mod events;
pub mod layers;
pub mod schema;
pub mod settings;

use crate::Result;
use rusqlite::Connection;
use std::path::Path;

pub struct Db {
    pub conn: Connection,
}

impl Db {
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        let mut db = Self { conn };
        db.run_migrations()?;
        Ok(db)
    }

    pub fn open_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        let mut db = Self { conn };
        db.run_migrations()?;
        Ok(db)
    }

    fn run_migrations(&mut self) -> Result<()> {
        self.conn.execute_batch(schema::V1)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_memory_creates_all_tables() {
        let db = Db::open_memory().expect("open in-memory db");
        let mut stmt = db
            .conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap();
        let tables: Vec<String> = stmt
            .query_map([], |r| r.get(0))
            .unwrap()
            .collect::<std::result::Result<_, _>>()
            .unwrap();
        assert_eq!(
            tables,
            vec![
                "bindings".to_string(),
                "channels".to_string(),
                "events".to_string(),
                "layers".to_string(),
                "settings".to_string(),
            ]
        );
    }

    #[test]
    fn default_layer_is_seeded() {
        let db = Db::open_memory().unwrap();
        let count: i64 = db
            .conn
            .query_row("SELECT COUNT(*) FROM layers", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn schema_version_set_to_1() {
        let db = Db::open_memory().unwrap();
        let version: String = db
            .conn
            .query_row(
                "SELECT value FROM settings WHERE key='schema_version'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(version, "1");
    }
}
