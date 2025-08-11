use std::{path::Path, sync::Mutex};

use rospeek_core::{BagReader, RawMessage, RosPeekError, RosPeekResult, Topic};
use rospeek_db3::Db3Reader;

pub trait Backend: Send + Sync {
    fn open<P: AsRef<Path>>(path: P) -> RosPeekResult<Self>
    where
        Self: Sized;

    fn topics(&self) -> RosPeekResult<Vec<Topic>>;

    fn read_messages(
        &self,
        topic: &str,
        start_ns: Option<i64>,
        limit: usize,
    ) -> RosPeekResult<Vec<RawMessage>>;
}

pub struct ReaderBackend {
    inner: Mutex<Box<dyn BagReader>>,
}

impl Backend for ReaderBackend {
    fn open<P: AsRef<Path>>(path: P) -> RosPeekResult<Self>
    where
        Self: Sized,
    {
        let ext = path
            .as_ref()
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        let reader = match ext {
            "db3" => Box::new(Db3Reader::open(path.as_ref())?),
            _ => return Err(RosPeekError::UnsupportedFormat(ext.to_string())),
        };

        Ok(Self {
            inner: Mutex::new(reader),
        })
    }

    fn topics(&self) -> RosPeekResult<Vec<Topic>> {
        self.inner.lock().unwrap().topics()
    }

    fn read_messages(
        &self,
        topic: &str,
        start_ns: Option<i64>,
        limit: usize,
    ) -> RosPeekResult<Vec<RawMessage>> {
        let mut messages = self.inner.lock().unwrap().read_messages(topic)?;

        if let Some(start) = start_ns {
            messages.retain(|m| m.timestamp >= start);
        }
        if messages.len() > limit {
            messages.truncate(limit);
        }

        Ok(messages)
    }
}
