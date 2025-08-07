use rusqlite::{Connection, params};
use std::fs;
use std::path::Path;

pub fn generate_test_db<P: AsRef<Path>>(path: P) {
    let path = path.as_ref();

    if path.exists() {
        return;
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("Failed to create test data dir");
    }

    let conn = Connection::open(path).expect("Failed to create test.db3");

    // Create schema
    conn.execute_batch(
        r#"
        CREATE TABLE topics (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            type TEXT NOT NULL,
            serialization_format TEXT NOT NULL,
            offered_qos_profiles TEXT
        );

        CREATE TABLE messages (
            id INTEGER PRIMARY KEY,
            topic_id INTEGER NOT NULL,
            timestamp INTEGER NOT NULL,
            data BLOB NOT NULL
        );
        "#,
    )
    .expect("Failed to create tables");

    // Insert test topic
    conn.execute(
        "INSERT INTO topics (id, name, type, serialization_format) VALUES (?1, ?2, ?3, ?4)",
        params![1, "/test_topic", "std_msgs/msg/String", "cdr"],
    )
    .expect("Failed to insert topic");

    // Insert test message (e.g., serialized "hello")
    let cdr_hello: Vec<u8> = vec![
        0x00, 0x01, 0x00, 0x00, // CDR header
        0x06, 0x00, 0x00, 0x00, // length = 6
        b'h', b'e', b'l', b'l', b'o', 0x00, // "hello\0"
    ];
    conn.execute(
        "INSERT INTO messages (id, topic_id, timestamp, data) VALUES (?1, ?2, ?3, ?4)",
        params![1, 1, 1234567890i64, cdr_hello],
    )
    .expect("Failed to insert message");
}
