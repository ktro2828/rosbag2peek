use std::path::Path;

use rospeek_core::{
    BagReader, RawMessage, RosPeekError, RosPeekResult, Topic, ns_to_iso,
    reader::{BagStats, StorageType},
    size_gb, to_duration_sec,
};
use rusqlite::Connection;

pub struct Db3Reader {
    connection: rusqlite::Connection,
    stats: BagStats,
}

impl BagReader for Db3Reader {
    fn open<P: AsRef<Path>>(path: P) -> RosPeekResult<Self>
    where
        Self: Sized,
    {
        let connection = Connection::open(path.as_ref())
            .map_err(|e| RosPeekError::Other(format!("SQLite open error: {}", e)))?;

        let (start_ns, end_ns) = connection
            .query_row(
                "SELECT COALESCE(MIN(timestamp), 0), COALESCE(MAX(timestamp), 0) FROM messages",
                [],
                |r| {
                    let start_ns: i64 = r.get(0)?;
                    let end_ns: i64 = r.get(1)?;
                    Ok((start_ns, end_ns))
                },
            )
            .map_err(|e| RosPeekError::Other(format!("Prepare statement failed: {}", e)))?;

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
            )
            .map_err(|e| RosPeekError::Other(format!("Prepare statement failed: {}", e)))?;

        let rows = statement
            .query_map([], |row| {
                Ok(Topic {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    type_name: row.get(2)?,
                    count: row.get(3)?,
                    serialization_format: row.get(4)?,
                    offered_qos_profiles: row.get(5)?,
                })
            })
            .map_err(|e| RosPeekError::Other(format!("Query failed: {}", e)))?;

        rows.collect::<Result<_, _>>()
            .map_err(|e| RosPeekError::Other(format!("Row mapping failed: {}", e)))
    }

    fn read_messages(&self, topic_name: &str) -> RosPeekResult<Vec<rospeek_core::RawMessage>> {
        let topic_id = self
            .connection
            .query_row(
                "SELECT id FROM topics WHERE name = ?1",
                [topic_name],
                |row| row.get(0),
            )
            .map_err(|_| RosPeekError::TopicNotFound(topic_name.to_string()))?;

        let mut statement = self
            .connection
            .prepare(
                "SELECT timestamp, data FROM messages WHERE topic_id = ?1 ORDER BY timestamp ASC",
            )
            .map_err(|e| RosPeekError::Other(format!("Prepare statement failed: {}", e)))?;

        let rows = statement
            .query_map([topic_id], |row| {
                Ok(RawMessage {
                    timestamp: row.get(0)?,
                    topic_id,
                    data: row.get(1)?,
                })
            })
            .map_err(|e| RosPeekError::Other(format!("Query failed: {}", e)))?;

        rows.collect::<Result<_, _>>()
            .map_err(|e| RosPeekError::Other(format!("Row mapping failed: {}", e)))
    }
}
