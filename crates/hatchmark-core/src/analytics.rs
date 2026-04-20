use crate::Result;
use rusqlite::{params, Connection};
use serde::Serialize;

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct DailyCount {
    pub channel_id: i64,
    pub count: i64,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct DateCount {
    pub date: String,
    pub count: i64,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct HourCount {
    pub hour: i64,
    pub count: i64,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct Rollup {
    pub channel_id: i64,
    pub total: i64,
    pub daily_average: f64,
    pub previous_total: i64,
}

pub fn today_per_channel(conn: &Connection) -> Result<Vec<DailyCount>> {
    let mut stmt = conn.prepare(
        "SELECT c.id, COALESCE(COUNT(e.id), 0)
           FROM channels c
           LEFT JOIN events e
             ON e.channel_id = c.id
            AND date(e.timestamp, 'localtime') = date('now', 'localtime')
          GROUP BY c.id
          ORDER BY c.sort_order, c.id",
    )?;
    let rows = stmt
        .query_map([], |r| {
            Ok(DailyCount {
                channel_id: r.get(0)?,
                count: r.get(1)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn history(conn: &Connection, channel_id: i64, days: i64) -> Result<Vec<DateCount>> {
    let mut stmt = conn.prepare(
        "SELECT date(timestamp, 'localtime') AS d, COUNT(*)
           FROM events
          WHERE channel_id = ?1
            AND date(timestamp, 'localtime') >= date('now', 'localtime', ?2)
          GROUP BY d
          ORDER BY d",
    )?;
    let offset = format!("-{} days", days - 1);
    let rows = stmt
        .query_map(params![channel_id, offset], |r| {
            Ok(DateCount {
                date: r.get(0)?,
                count: r.get(1)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn heatmap(conn: &Connection, channel_id: i64, days: i64) -> Result<Vec<DateCount>> {
    history(conn, channel_id, days)
}

pub fn hourly(
    conn: &Connection,
    channel_id: i64,
    from_date: &str,
    to_date: &str,
) -> Result<Vec<HourCount>> {
    let mut stmt = conn.prepare(
        "SELECT CAST(strftime('%H', timestamp, 'localtime') AS INTEGER) AS hr, COUNT(*)
           FROM events
          WHERE channel_id = ?1
            AND date(timestamp, 'localtime') BETWEEN ?2 AND ?3
          GROUP BY hr
          ORDER BY hr",
    )?;
    let rows = stmt
        .query_map(params![channel_id, from_date, to_date], |r| {
            Ok(HourCount {
                hour: r.get(0)?,
                count: r.get(1)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn rollup(conn: &Connection, channel_id: i64, window_days: i64) -> Result<Rollup> {
    let total: i64 = conn.query_row(
        "SELECT COUNT(*) FROM events
          WHERE channel_id = ?1
            AND date(timestamp,'localtime') >= date('now','localtime', ?2)",
        params![channel_id, format!("-{} days", window_days - 1)],
        |r| r.get(0),
    )?;
    let prev_total: i64 = conn.query_row(
        "SELECT COUNT(*) FROM events
          WHERE channel_id = ?1
            AND date(timestamp,'localtime') >= date('now','localtime', ?2)
            AND date(timestamp,'localtime') <  date('now','localtime', ?3)",
        params![
            channel_id,
            format!("-{} days", window_days * 2 - 1),
            format!("-{} days", window_days - 1),
        ],
        |r| r.get(0),
    )?;
    Ok(Rollup {
        channel_id,
        total,
        daily_average: total as f64 / window_days as f64,
        previous_total: prev_total,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{channels, events, Db};

    fn db_with_channel() -> (Db, i64) {
        let db = Db::open_memory().unwrap();
        let cid = channels::create(&db.conn, "Water", "#7dd3fc", None, None).unwrap();
        (db, cid)
    }

    #[test]
    fn today_counts_include_channels_with_zero_events() {
        let (db, cid) = db_with_channel();
        let rows = today_per_channel(&db.conn).unwrap();
        assert_eq!(
            rows,
            vec![DailyCount {
                channel_id: cid,
                count: 0
            }]
        );
    }

    #[test]
    fn today_counts_reflect_inserts() {
        let (db, cid) = db_with_channel();
        events::insert(&db.conn, cid).unwrap();
        events::insert(&db.conn, cid).unwrap();
        let rows = today_per_channel(&db.conn).unwrap();
        assert_eq!(rows[0].count, 2);
    }

    #[test]
    fn history_groups_by_local_date() {
        let (db, cid) = db_with_channel();
        events::insert(&db.conn, cid).unwrap();
        events::insert(&db.conn, cid).unwrap();
        let rows = history(&db.conn, cid, 7).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].count, 2);
    }
}
