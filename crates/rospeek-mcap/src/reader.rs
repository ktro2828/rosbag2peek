use std::{fs::File, path::Path};

use mcap::MessageStream;
use memmap2::Mmap;
use rospeek_core::{
    BagReader, RawMessage, RosPeekError, RosPeekResult, Topic,
    reader::{BagStats, StorageType},
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
        let stats = BagStats {
            path: path.as_ref().display().to_string(),
            size_bytes: 0.0,
            storage_type: StorageType::Mcap,
            duration_sec: 0.0,
            start_time: "".to_string(),
            end_time: "".to_string(),
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
