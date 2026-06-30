use std::sync::{Arc, RwLock};

use rusqlite::{params, Connection, Result};
use tracing::info;

use crate::models;
use crate::Error;

#[derive(Clone, Debug)]
pub struct Db {
    conn: Arc<RwLock<Connection>>,
}

impl Db {
    pub fn connect(path: &str) -> Result<Self> {
        let mut this = Self {
            conn: Arc::new(RwLock::new(Connection::open(path)?)),
        };
        {
            this.conn.read().unwrap().execute_batch(
                "PRAGMA foreign_keys = ON;
        PRAGMA journal_mode = wal;",
            )?;
        }
        this.migrate()?;
        Ok(this)
    }
    fn migrate(&mut self) -> Result<()> {
        let conn = self.conn.write().unwrap();
        let version: i32 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
        
        if version < 1 {
            conn.execute_batch(include_str!("./migrations/00.sql"))?;
            conn.pragma_update(None, "user_version", 1)?;
        }
        if version < 2 {
            conn.execute_batch(include_str!("./migrations/01.sql"))?;
            conn.pragma_update(None, "user_version", 2)?;
        }
        if version < 3 {
            conn.execute_batch(include_str!("./migrations/02.sql"))?;
            conn.pragma_update(None, "user_version", 3)?;
        }
        Ok(())
    }
    fn get_or_insert_server(&mut self, server: &str) -> Result<i64> {
        let mut conn = self.conn.write().unwrap();
        let tx = conn.transaction()?;
        let mut res = tx.query_row(
            "SELECT id
                FROM server
                WHERE endpoint = ?1",
            params![server,],
            |row| {
                let id: i64 = row.get(0)?;
                Ok(id)
            },
        );
        if let Err(rusqlite::Error::QueryReturnedNoRows) = res {
            tx.execute(
                "INSERT INTO server (id, endpoint) VALUES (NULL, ?1)",
                params![server,],
            )?;
            res = Ok(tx.last_insert_rowid());
        }
        tx.commit()?;
        res
    }
    pub fn insert_message(&mut self, server: &str, json_data: &str) -> Result<(), Error> {
        let server_id = self.get_or_insert_server(server)?;
        let res = self.conn.read().unwrap().execute(
            "INSERT INTO message (server, data) VALUES (?1, ?2)",
            params![server_id, json_data],
        );
        match res {
            Err(rusqlite::Error::SqliteFailure(_, Some(text)))
                if text.starts_with("UNIQUE constraint failed") =>
            {
                Err(Error::DuplicateMessage)
            }
            Err(e) => Err(Error::Db(e)),
            Ok(_) => Ok(()),
        }
    }
    pub fn list_messages(
        &self,
        server: &str,
        topic: &str,
        since: u64,
    ) -> Result<Vec<String>, rusqlite::Error> {
        let conn = self.conn.read().unwrap();
        let mut stmt = conn.prepare(
            "
            SELECT data
            FROM subscription sub
            JOIN server s ON sub.server = s.id
            JOIN message m ON m.server = sub.server AND m.topic = sub.topic
            WHERE s.endpoint = ?1 AND m.topic = ?2 AND m.data ->> 'time' >= ?3
            ORDER BY m.data ->> 'time'
        ",
        )?;
        let msgs: Result<Vec<String>, _> = stmt
            .query_map(params![server, topic, since as i64], |row| row.get(0))?
            .collect();
        msgs
    }
    pub fn insert_subscription(&mut self, sub: models::Subscription) -> Result<(), Error> {
        let server_id = self.get_or_insert_server(&sub.server)?;
        // Create JSON strings for new fields
        let rules = serde_json::to_string(&sub.rules).unwrap_or_default();
        let schedule = serde_json::to_string(&sub.schedule).unwrap_or_default();

        self.conn.read().unwrap().execute(
            "INSERT INTO subscription (server, topic, display_name, reserved, muted, archived, symbolic_icon, read_until, listen_since, rules, schedule) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                server_id,
                sub.topic,
                sub.display_name,
                sub.reserved,
                sub.muted,
                sub.archived,
                sub.symbolic_icon,
                sub.read_until as i64,
                sub.listen_since as i64,
                rules,
                schedule
            ],
        )?;
        Ok(())
    }
    pub fn remove_subscription(&mut self, server: &str, topic: &str) -> Result<(), Error> {
        let server_id = self.get_or_insert_server(server)?;
        let res = self.conn.read().unwrap().execute(
            "DELETE FROM subscription
            WHERE server = ?1 AND topic = ?2",
            params![server_id, topic],
        )?;
        if res == 0 {
            return Err(Error::SubscriptionNotFound("removing subscription".into()));
        }
        Ok(())
    }
    pub fn list_subscriptions(&mut self) -> Result<Vec<models::Subscription>, Error> {
        let conn = self.conn.read().unwrap();
        let mut stmt = conn.prepare(
            "SELECT server.endpoint, sub.topic, sub.display_name, sub.reserved, sub.muted, sub.archived, sub.symbolic_icon, sub.read_until, sub.listen_since, sub.rules, sub.schedule
            FROM subscription sub
            JOIN server ON server.id = sub.server
            ORDER BY server.endpoint, sub.display_name, sub.topic
            ",
        )?;
        let rows = stmt.query_map(params![], |row| {
            let rules_str: Option<String> = row.get(9)?;
            let schedule_str: Option<String> = row.get(10)?;
            
            Ok(models::Subscription {
                server: row.get(0)?,
                topic: row.get(1)?,
                display_name: row.get(2)?,
                reserved: row.get(3)?,
                muted: row.get(4)?,
                archived: row.get(5)?,
                symbolic_icon: row.get(6)?,
                read_until: row.get::<_, i64>(7)? as u64,
                listen_since: row.get::<_, i64>(8)? as u64,
                rules: rules_str.and_then(|s| serde_json::from_str(&s).ok()),
                schedule: schedule_str.and_then(|s| serde_json::from_str(&s).ok()),
            })
        })?;
        let subs: Result<Vec<_>, rusqlite::Error> = rows.collect();
        Ok(subs?)
    }

    fn server_id_or_insert_tx(
        tx: &rusqlite::Transaction<'_>,
        endpoint: &str,
    ) -> rusqlite::Result<i64> {
        match tx.query_row(
            "SELECT id FROM server WHERE endpoint = ?1",
            params![endpoint],
            |row| row.get(0),
        ) {
            Ok(id) => Ok(id),
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                tx.execute(
                    "INSERT INTO server (id, endpoint) VALUES (NULL, ?1)",
                    params![endpoint],
                )?;
                Ok(tx.last_insert_rowid())
            }
            Err(e) => Err(e),
        }
    }

    /// Single transaction: bump `read_until` then delete cached messages so we never persist
    /// read state if the delete fails.
    pub fn bump_read_until_and_clear_messages_atomic(
        &mut self,
        server: &str,
        topic: &str,
        bump: u64,
    ) -> Result<(), Error> {
        let mut conn = self.conn.write().unwrap();
        let tx = conn.transaction().map_err(Error::Db)?;
        let server_id = Self::server_id_or_insert_tx(&tx, server).map_err(Error::Db)?;
        let n = tx.execute(
            "UPDATE subscription
            SET read_until = ?3, listen_since = ?3
            WHERE server = ?1 AND topic = ?2",
            params![server_id, topic, bump as i64],
        )
        .map_err(Error::Db)?;
        if n == 0 {
            tx.rollback().ok();
            return Err(Error::SubscriptionNotFound(
                "bump read_until and clear messages".into(),
            ));
        }
        tx.execute(
            "DELETE FROM message
            WHERE server = ?1 AND topic = ?2",
            params![server_id, topic],
        )
        .map_err(Error::Db)?;
        tx.commit().map_err(Error::Db)?;
        Ok(())
    }

    pub fn update_subscription(&mut self, sub: models::Subscription) -> Result<(), Error> {
        let server_id = self.get_or_insert_server(&sub.server)?;
        let rules = serde_json::to_string(&sub.rules).unwrap_or_default();
        let schedule = serde_json::to_string(&sub.schedule).unwrap_or_default();

        let res = self.conn.read().unwrap().execute(
            "UPDATE subscription
            SET display_name = ?1, reserved = ?2, muted = ?3, archived = ?4, symbolic_icon = ?5, read_until = ?6, listen_since = ?11, rules = ?9, schedule = ?10
            WHERE server = ?7 AND topic = ?8",
            params![
                sub.display_name,
                sub.reserved,
                sub.muted,
                sub.archived,
                sub.symbolic_icon,
                sub.read_until as i64,
                server_id,
                sub.topic,
                rules,
                schedule,
                sub.listen_since as i64,
            ],
        )?;
        if res == 0 {
            return Err(Error::SubscriptionNotFound("updating subscription".into()));
        }
        info!(info = ?sub, "stored subscription info");
        Ok(())
    }

    pub fn update_read_until(
        &mut self,
        server: &str,
        topic: &str,
        value: u64,
    ) -> Result<(), Error> {
        let server_id = self.get_or_insert_server(server).unwrap();
        let conn = self.conn.read().unwrap();
        let res = conn.execute(
            "UPDATE subscription
            SET read_until = ?3
            WHERE topic = ?2 AND server = ?1
            ",
            params![server_id, topic, value as i64],
        )?;
        if res == 0 {
            return Err(Error::SubscriptionNotFound("updating read_until".into()));
        }
        Ok(())
    }
    pub fn delete_message(&mut self, server: &str, topic: &str, id: &str) -> Result<(), Error> {
        let server_id = self.get_or_insert_server(server).unwrap();
        let conn = self.conn.read().unwrap();
        conn.execute(
            "DELETE FROM message
            WHERE server = ?1 AND topic = ?2 AND data ->> 'id' = ?3
            ",
            params![server_id, topic, id],
        )?;
        Ok(())
    }

    /// Remove every message belonging to a notification sequence: the original
    /// (id == seq_key) and any updates (sequence_id == seq_key).
    pub fn delete_by_seq_key(&mut self, server: &str, topic: &str, seq_key: &str) -> Result<(), Error> {
        let server_id = self.get_or_insert_server(server).unwrap();
        let conn = self.conn.read().unwrap();
        conn.execute(
            "DELETE FROM message
            WHERE server = ?1 AND topic = ?2
              AND (data ->> 'id' = ?3 OR data ->> 'sequence_id' = ?3)
            ",
            params![server_id, topic, seq_key],
        )?;
        Ok(())
    }

    pub fn delete_messages(&mut self, server: &str, topic: &str) -> Result<(), Error> {
        let server_id = self.get_or_insert_server(server).unwrap();
        let conn = self.conn.read().unwrap();
        conn.execute(
            "DELETE FROM message
            WHERE topic = ?2 AND server = ?1
            ",
            params![server_id, topic],
        )?;
        Ok(())
    }

    pub fn get_last_message_time(
        &self,
        server: &str,
        topic: &str,
    ) -> Result<Option<u64>, Error> {
        let conn = self.conn.read().unwrap();
        let mut stmt = conn.prepare(
            "SELECT MAX(m.data ->> 'time')
            FROM message m
            JOIN server s ON m.server = s.id
            WHERE s.endpoint = ?1 AND m.topic = ?2
            ",
        )?;
        let mut rows = stmt.query(params![server, topic])?;
        if let Some(row) = rows.next()? {
            Ok(row.get::<_, Option<i64>>(0)?.map(|v| v as u64))
        } else {
            Ok(None)
        }
    }
}

pub(crate) fn compute_listen_since(db_max_message_time: u64, listen_since: u64) -> u64 {
    db_max_message_time.max(listen_since)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models;

    fn sample_message_json(topic: &str, id: &str, time: u64) -> String {
        format!(
            r#"{{"id":"{id}","time":{time},"event":"message","topic":"{topic}","message":"hello"}}"#
        )
    }

    #[test]
    fn compute_listen_since_prefers_local_cache() {
        assert_eq!(compute_listen_since(100, 0), 100);
        assert_eq!(compute_listen_since(100, 200), 200);
    }

    #[test]
    fn new_subscription_with_read_until_still_fetches_history() {
        let mut db = Db::connect(":memory:").unwrap();
        let read_until = 1_700_000_000;
        let sub = models::Subscription::builder("alerts".into())
            .server("https://ntfy.example".into())
            .read_until(read_until)
            .build()
            .unwrap();
        db.insert_subscription(sub.clone()).unwrap();

        let db_max = db
            .get_last_message_time(&sub.server, &sub.topic)
            .unwrap()
            .unwrap_or(0);
        let since = compute_listen_since(db_max, sub.listen_since);

        assert_eq!(sub.read_until, read_until);
        assert_eq!(sub.listen_since, 0);
        assert_eq!(since, 0);
    }

    fn update_message_json(topic: &str, id: &str, seq: &str, time: u64) -> String {
        format!(
            r#"{{"id":"{id}","sequence_id":"{seq}","time":{time},"event":"message","topic":"{topic}","message":"updated"}}"#
        )
    }

    #[test]
    fn delete_by_seq_key_replaces_a_notification_sequence() {
        let mut db = Db::connect(":memory:").unwrap();
        let sub = models::Subscription::builder("alerts".into())
            .server("https://ntfy.example".into())
            .build()
            .unwrap();
        db.insert_subscription(sub.clone()).unwrap();

        // Original notification, then an update referencing it via sequence_id.
        db.insert_message(&sub.server, &sample_message_json("alerts", "orig", 1_700_000_010))
            .unwrap();
        db.delete_by_seq_key(&sub.server, &sub.topic, "orig").unwrap();
        db.insert_message(
            &sub.server,
            &update_message_json("alerts", "upd", "orig", 1_700_000_020),
        )
        .unwrap();

        // Only the updated message survives.
        let stored = db.list_messages(&sub.server, &sub.topic, 0).unwrap();
        assert_eq!(stored.len(), 1);
        assert!(stored[0].contains("\"id\":\"upd\""));

        // A clear/delete for the sequence removes the update too (matches by sequence_id).
        db.delete_by_seq_key(&sub.server, &sub.topic, "orig").unwrap();
        assert!(db.list_messages(&sub.server, &sub.topic, 0).unwrap().is_empty());
    }

    #[test]
    fn cleared_subscription_skips_replayed_history() {
        let mut db = Db::connect(":memory:").unwrap();
        let sub = models::Subscription::builder("alerts".into())
            .server("https://ntfy.example".into())
            .build()
            .unwrap();
        db.insert_subscription(sub.clone()).unwrap();

        let msg = sample_message_json("alerts", "abc", 1_700_000_010);
        db.insert_message(&sub.server, &msg).unwrap();

        db.bump_read_until_and_clear_messages_atomic(&sub.server, &sub.topic, 1_700_000_050)
            .unwrap();

        let stored = db.list_subscriptions().unwrap().pop().unwrap();
        assert_eq!(stored.listen_since, 1_700_000_050);
        assert!(db
            .list_messages(&sub.server, &sub.topic, 0)
            .unwrap()
            .is_empty());

        let db_max = db
            .get_last_message_time(&sub.server, &sub.topic)
            .unwrap()
            .unwrap_or(0);
        let since = compute_listen_since(db_max, stored.listen_since);
        assert_eq!(since, 1_700_000_050);
    }
}
