use crate::Result;
use chrono::Utc;
use rusqlite::{params, Connection};
use std::io::Write;

pub fn insert(conn: &Connection, channel_id: i64) -> Result<i64> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO events (channel_id, timestamp) VALUES (?1, ?2)",
        params![channel_id, now],
    )?;
    let total: i64 = conn.query_row(
        "SELECT COUNT(*) FROM events
           WHERE channel_id = ?1
             AND date(timestamp, 'localtime') = date('now', 'localtime')",
        params![channel_id],
        |r| r.get(0),
    )?;
    Ok(total)
}

pub fn delete_last_for_channel(conn: &Connection, channel_id: i64) -> Result<bool> {
    let id: Option<i64> = conn
        .query_row(
            "SELECT id FROM events WHERE channel_id=?1 ORDER BY id DESC LIMIT 1",
            params![channel_id],
            |r| r.get(0),
        )
        .ok();
    match id {
        Some(i) => {
            conn.execute("DELETE FROM events WHERE id=?1", params![i])?;
            Ok(true)
        }
        None => Ok(false),
    }
}

pub fn export_all_to_writer<W: Write>(conn: &Connection, mut out: W) -> Result<usize> {
    let mut stmt = conn.prepare(
        "SELECT e.id, e.channel_id, c.name, e.timestamp
           FROM events e JOIN channels c ON c.id = e.channel_id
          ORDER BY e.id",
    )?;
    writeln!(out, "id,channel_id,channel_name,timestamp_utc")?;
    let mut rows = stmt.query([])?;
    let mut n = 0;
    while let Some(row) = rows.next()? {
        let id: i64 = row.get(0)?;
        let cid: i64 = row.get(1)?;
        let name: String = row.get(2)?;
        let ts: String = row.get(3)?;
        let name = if name.contains([',', '"', '\n']) {
            format!("\"{}\"", name.replace('"', "\"\""))
        } else {
            name
        };
        writeln!(out, "{id},{cid},{name},{ts}")?;
        n += 1;
    }
    Ok(n)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{channels, Db};

    #[test]
    fn insert_returns_running_total_for_today() {
        let db = Db::open_memory().unwrap();
        let cid = channels::create(&db.conn, "Water", "#7dd3fc", None, None).unwrap();
        assert_eq!(insert(&db.conn, cid).unwrap(), 1);
        assert_eq!(insert(&db.conn, cid).unwrap(), 2);
        assert_eq!(insert(&db.conn, cid).unwrap(), 3);
    }

    #[test]
    fn each_channel_has_its_own_count() {
        let db = Db::open_memory().unwrap();
        let a = channels::create(&db.conn, "Water", "#7dd3fc", None, None).unwrap();
        let b = channels::create(&db.conn, "Vape", "#f87171", None, None).unwrap();
        insert(&db.conn, a).unwrap();
        insert(&db.conn, a).unwrap();
        insert(&db.conn, b).unwrap();
        let count: i64 = db
            .conn
            .query_row(
                "SELECT COUNT(*) FROM events WHERE channel_id=?1",
                params![a],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn delete_last_removes_most_recent_event() {
        let db = Db::open_memory().unwrap();
        let cid = channels::create(&db.conn, "Water", "#fff", None, None).unwrap();
        insert(&db.conn, cid).unwrap();
        insert(&db.conn, cid).unwrap();
        assert!(delete_last_for_channel(&db.conn, cid).unwrap());
        let n: i64 = db
            .conn
            .query_row("SELECT COUNT(*) FROM events", [], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 1);
    }

    #[test]
    fn export_all_writes_header_and_rows() {
        let db = Db::open_memory().unwrap();
        let cid = channels::create(&db.conn, "Water", "#fff", None, None).unwrap();
        insert(&db.conn, cid).unwrap();
        let mut buf = Vec::new();
        let n = export_all_to_writer(&db.conn, &mut buf).unwrap();
        assert_eq!(n, 1);
        let s = String::from_utf8(buf).unwrap();
        assert!(s.starts_with("id,channel_id,channel_name,timestamp_utc\n"));
        assert!(s.contains(",Water,"));
    }
}
