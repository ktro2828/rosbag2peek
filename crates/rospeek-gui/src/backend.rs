use std::{path::Path, sync::Mutex};

use anyhow::bail;
use rospeek_core::{BagReader, RawMessage, RosPeekResult, Topic};
use rospeek_db3::Db3Reader;
use rospeek_mcap::McapReader;

pub trait Backend: Send + Sync {
    fn open<P: AsRef<Path>>(path: P) -> RosPeekResult<Self>
    where
        Self: Sized;

    fn topics(&self) -> RosPeekResult<Vec<Topic>>;

    fn read_messages(
        &self,
        topic: &str,
        start_ns: Option<u64>,
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
        let reader = create_reader(path)?;

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
        start_ns: Option<u64>,
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

pub fn create_reader<P: AsRef<Path>>(bag: P) -> RosPeekResult<Box<dyn BagReader>> {
    let reader: Box<dyn BagReader> = match bag.as_ref().extension().and_then(|ext| ext.to_str()) {
        Some("db3") => Box::new(Db3Reader::open(bag)?),
        Some("mcap") => Box::new(McapReader::open(bag)?),
        _ => bail!("Unsupported bag format: {}", bag.as_ref().display()),
    };

    Ok(reader)
}
