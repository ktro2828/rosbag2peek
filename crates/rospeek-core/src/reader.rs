use std::path::Path;

use crate::{RawMessage, RosPeekResult, Topic};

pub trait BagReader: Send {
    fn open<P: AsRef<Path>>(path: P) -> RosPeekResult<Self>
    where
        Self: Sized;

    fn stats(&self) -> &BagStats;

    fn topics(&self) -> RosPeekResult<Vec<Topic>>;

    fn read_messages(&self, topic_name: &str) -> RosPeekResult<Vec<RawMessage>>;
}

#[derive(Debug)]
pub struct BagStats {
    pub path: String,
    pub size_bytes: f64,
    pub storage_type: StorageType,
    pub duration_sec: f64,
    pub start_time: String,
    pub end_time: String,
}

#[derive(Debug)]
pub enum StorageType {
    Sqlite3,
}
