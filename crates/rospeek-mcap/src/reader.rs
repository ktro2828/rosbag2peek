use std::{fs::File, i64, path::Path};

use mcap::MessageStream;
use memmap2::Mmap;
use rospeek_core::{
    BagReader, BagStats, RawMessage, RosPeekError, RosPeekResult, StorageType, Topic, ns_to_iso,
    size_gb, to_duration_sec,
};

pub struct McapReader {
    mmap: Mmap,
    stats: BagStats,
}

impl BagReader for McapReader {
    fn open<P: AsRef<Path>>(path: P) -> RosPeekResult<Self> {
        let fd = File::open(path.as_ref())?;
        let mmap = unsafe { Mmap::map(&fd) }?;

        // TODO(kto2828): Implement stats calculation
        let start_ns = i64::MAX;
        let end_ns = i64::MIN;

        let stats = BagStats {
            path: path.as_ref().display().to_string(),
            size_bytes: size_gb(path),
            storage_type: StorageType::Mcap,
            duration_sec: to_duration_sec(start_ns, end_ns),
            start_time: ns_to_iso(start_ns),
            end_time: ns_to_iso(end_ns),
        };

        Ok(Self { mmap, stats })
    }

    fn stats(&self) -> &BagStats {
        &self.stats
    }

    fn topics(&self) -> RosPeekResult<Vec<Topic>> {
        for message in
            MessageStream::new(&self.mmap).map_err(|_| RosPeekError::Other("".to_string()))?
        {
            println!(
                "{:?}",
                message.map_err(|_| RosPeekError::Other("".to_string()))?
            );
        }
        Ok(vec![])
    }

    fn read_messages(&self, topic_name: &str) -> RosPeekResult<Vec<RawMessage>> {
        println!("Topic: {}", topic_name);

        for message in
            MessageStream::new(&self.mmap).map_err(|_| RosPeekError::Other("".to_string()))?
        {
            println!(
                "{:?}",
                message.map_err(|_| RosPeekError::Other("".to_string()))?
            );
        }
        Ok(vec![])
    }
}
