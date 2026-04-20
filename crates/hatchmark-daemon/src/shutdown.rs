use anyhow::Result;
use rusqlite::Connection;

pub fn checkpoint_wal(conn: &Connection) -> Result<()> {
    conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;
    Ok(())
}
