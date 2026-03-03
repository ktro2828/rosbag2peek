use std::path::Path;

use anyhow::anyhow;
use rospeek_core::{
    BagReader, RawMessage, RosPeekResult, Topic, ns_to_iso,
    reader::{BagStats, StorageType},
    size_gb, to_duration_sec,
};
use rusqlite::{Connection, params_from_iter, types::Value as SqlValue};

pub struct Db3Reader {
    connection: rusqlite::Connection,
    stats: BagStats,
}

impl BagReader for Db3Reader {
    fn open<P: AsRef<Path>>(path: P) -> RosPeekResult<Self>
    where
        Self: Sized,
    {
        let connection = Connection::open(path.as_ref())?;

        let (start_ns, end_ns) = connection.query_row(
            "SELECT COALESCE(MIN(timestamp), 0), COALESCE(MAX(timestamp), 0) FROM messages",
            [],
            |r| {
                let start_ns: u64 = r.get(0)?;
                let end_ns: u64 = r.get(1)?;
                Ok((start_ns, end_ns))
            },
        )?;

        let stats = BagStats {
            path: path.as_ref().display().to_string(),
            size_bytes: size_gb(path.as_ref()),
            storage_type: StorageType::Sqlite3,
            duration_sec: to_duration_sec(start_ns, end_ns),
            start_time: ns_to_iso(start_ns),
            end_time: ns_to_iso(end_ns),
        };

        Ok(Self { connection, stats })
    }

    fn stats(&self) -> &BagStats {
        &self.stats
    }

    fn topics(&self) -> RosPeekResult<Vec<rospeek_core::Topic>> {
        let mut statement = self
            .connection
            .prepare(
                r#"SELECT t.id, t.name, t.type, COUNT(m.id) AS message_count, t.serialization_format, t.offered_qos_profiles
                        FROM topics t
                        LEFT JOIN messages m ON t.id = m.topic_id
                        GROUP BY t.id
                        ORDER BY t.name"#,
            )?;

        let rows = statement.query_map([], |row| {
            Ok(Topic {
                id: row.get(0)?,
                name: row.get(1)?,
                type_name: row.get(2)?,
                count: row.get(3)?,
                serialization_format: row.get(4)?,
                offered_qos_profiles: row.get(5)?,
            })
        })?;

        Ok(rows.collect::<Result<_, _>>()?)
    }

    fn read_messages(&self, topic_name: &str) -> RosPeekResult<Vec<rospeek_core::RawMessage>> {
        self.read_messages_range(topic_name, None, None, None, None)
    }

    fn read_messages_range(
        &self,
        topic_name: &str,
        start_ns: Option<u64>,
        end_ns: Option<u64>,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> RosPeekResult<Vec<rospeek_core::RawMessage>> {
        let topic_id: u16 = self
            .connection
            .query_row(
                "SELECT id FROM topics WHERE name = ?1",
                [topic_name],
                |row| row.get(0),
            )
            .map_err(|_| anyhow!("Topic not found: {topic_name}"))?;

        let mut sql = String::from("SELECT timestamp, data FROM messages WHERE topic_id = ?");
        let mut params: Vec<SqlValue> = vec![SqlValue::from(topic_id as i64)];

        if let Some(start) = start_ns {
            sql.push_str(" AND timestamp >= ?");
            params.push(SqlValue::from(start as i64));
        }
        if let Some(end) = end_ns {
            sql.push_str(" AND timestamp <= ?");
            params.push(SqlValue::from(end as i64));
        }

        sql.push_str(" ORDER BY timestamp ASC");

        let mut has_limit = false;
        if let Some(limit) = limit {
            sql.push_str(" LIMIT ?");
            params.push(SqlValue::from(limit as i64));
            has_limit = true;
        }
        if let Some(offset) = offset {
            if !has_limit {
                sql.push_str(" LIMIT -1");
            }
            sql.push_str(" OFFSET ?");
            params.push(SqlValue::from(offset as i64));
        }

        let mut statement = self.connection.prepare(&sql)?;

        let rows = statement.query_map(params_from_iter(params), |row| {
            Ok(RawMessage {
                timestamp: row.get(0)?,
                topic_id,
                data: row.get(1)?,
            })
        })?;

        Ok(rows.collect::<Result<_, _>>()?)
    }
}
