use crate::model::Channel;
use crate::{CoreError, Result};
use rusqlite::{params, Connection};

pub fn list(conn: &Connection) -> Result<Vec<Channel>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, color, daily_goal, daily_limit, sort_order
           FROM channels ORDER BY sort_order, id",
    )?;
    let rows = stmt
        .query_map([], |r| {
            Ok(Channel {
                id: r.get(0)?,
                name: r.get(1)?,
                color: r.get(2)?,
                daily_goal: r.get(3)?,
                daily_limit: r.get(4)?,
                sort_order: r.get(5)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn get(conn: &Connection, id: i64) -> Result<Channel> {
    conn.query_row(
        "SELECT id, name, color, daily_goal, daily_limit, sort_order
           FROM channels WHERE id = ?1",
        params![id],
        |r| {
            Ok(Channel {
                id: r.get(0)?,
                name: r.get(1)?,
                color: r.get(2)?,
                daily_goal: r.get(3)?,
                daily_limit: r.get(4)?,
                sort_order: r.get(5)?,
            })
        },
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => CoreError::NotFound {
            kind: "channel",
            id,
        },
        other => other.into(),
    })
}

pub fn create(
    conn: &Connection,
    name: &str,
    color: &str,
    daily_goal: Option<i64>,
    daily_limit: Option<i64>,
) -> Result<i64> {
    let next_sort: i64 = conn.query_row(
        "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM channels",
        [],
        |r| r.get(0),
    )?;
    conn.execute(
        "INSERT INTO channels (name, color, daily_goal, daily_limit, sort_order)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![name, color, daily_goal, daily_limit, next_sort],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn update(conn: &Connection, ch: &Channel) -> Result<()> {
    let n = conn.execute(
        "UPDATE channels SET name=?1, color=?2, daily_goal=?3, daily_limit=?4, sort_order=?5
           WHERE id=?6",
        params![
            ch.name,
            ch.color,
            ch.daily_goal,
            ch.daily_limit,
            ch.sort_order,
            ch.id
        ],
    )?;
    if n == 0 {
        return Err(CoreError::NotFound {
            kind: "channel",
            id: ch.id,
        });
    }
    Ok(())
}

pub fn delete(conn: &Connection, id: i64) -> Result<()> {
    let n = conn.execute("DELETE FROM channels WHERE id=?1", params![id])?;
    if n == 0 {
        return Err(CoreError::NotFound {
            kind: "channel",
            id,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Db;

    fn seed() -> Db {
        let db = Db::open_memory().unwrap();
        create(&db.conn, "Water", "#7dd3fc", Some(8), None).unwrap();
        create(&db.conn, "Vape", "#f87171", None, Some(10)).unwrap();
        db
    }

    #[test]
    fn create_and_list_channels_in_sort_order() {
        let db = seed();
        let channels = list(&db.conn).unwrap();
        assert_eq!(channels.len(), 2);
        assert_eq!(channels[0].name, "Water");
        assert_eq!(channels[0].sort_order, 0);
        assert_eq!(channels[1].name, "Vape");
        assert_eq!(channels[1].sort_order, 1);
    }

    #[test]
    fn get_returns_not_found_for_missing_id() {
        let db = Db::open_memory().unwrap();
        let err = get(&db.conn, 999).unwrap_err();
        assert!(matches!(
            err,
            CoreError::NotFound {
                kind: "channel",
                id: 999
            }
        ));
    }

    #[test]
    fn update_mutates_persisted_row() {
        let db = seed();
        let mut ch = get(&db.conn, 1).unwrap();
        ch.name = "Sparkling Water".into();
        update(&db.conn, &ch).unwrap();
        assert_eq!(get(&db.conn, 1).unwrap().name, "Sparkling Water");
    }

    #[test]
    fn delete_removes_row() {
        let db = seed();
        delete(&db.conn, 1).unwrap();
        assert_eq!(list(&db.conn).unwrap().len(), 1);
    }
}
