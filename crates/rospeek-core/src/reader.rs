use chrono::DateTime;
use std::{
    fmt::{Display, Formatter},
    path::Path,
};

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

impl Display for BagStats {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "File:             {}\nBag size:         {:.3} GiB\nStorage type:     {}\nDuration:         {} s\nStart:            {}\nEnd:              {}",
            self.path,
            self.size_bytes,
            self.storage_type,
            self.duration_sec,
            self.start_time,
            self.end_time
        )
    }
}

#[derive(Debug)]
pub enum StorageType {
    Sqlite3,
    Mcap,
}

impl Display for StorageType {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Returns the size of a file in GB.
///
/// # Arguments
/// * `path` - The path to the file.
pub fn size_gb<P: AsRef<Path>>(path: P) -> f64 {
    let size_bytes = path.as_ref().metadata().unwrap().len() as f64;
    size_bytes / (1024.0 * 1024.0 * 1024.0)
}

/// Converts nanoseconds to ISO 8601 format.
///
/// # Arguments
/// * `ns` - The nanoseconds since the Unix epoch.
///
/// # Examples
/// ```
/// use rospeek_core::ns_to_iso;
///
/// assert_eq!(ns_to_iso(1630456800000000000), "2021-09-01 00:40:00");
/// ```
pub fn ns_to_iso(ns: u64) -> String {
    let secs = (ns / 1_000_000_000) as i64;
    let nsecs = (ns % 1_000_000_000) as u32;
    let date = DateTime::from_timestamp(secs, nsecs).unwrap();
    date.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// Converts nanoseconds to duration in seconds.
///
/// # Arguments
/// * `start_ns` - The start time in nanoseconds.
/// * `end_ns` - The end time in nanoseconds.
///
/// # Examples
/// ```
/// use rospeek_core::to_duration_sec;
///
/// assert_eq!(to_duration_sec(1630456800000000000, 1630456860000000000), 60.0);
/// ```
pub fn to_duration_sec(start_ns: u64, end_ns: u64) -> f64 {
    if end_ns > start_ns {
        (end_ns - start_ns) as f64 / 1_000_000_000.0
    } else {
        0.0
    }
}
