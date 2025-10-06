use std::{fs::File, path::Path};

use mcap::MessageStream;
use memmap2::Mmap;
use rospeek_core::{
    BagReader, BagStats, RawMessage, RosPeekResult, StorageType, Topic, ns_to_iso, size_gb,
    to_duration_sec,
};

pub struct McapReader {
    mmap: Mmap,
    stats: BagStats,
}

impl McapReader {
    fn as_stream(&self) -> RosPeekResult<MessageStream<'_>> {
        Ok(MessageStream::new(&self.mmap)?)
    }
}

impl BagReader for McapReader {
    fn open<P: AsRef<Path>>(path: P) -> RosPeekResult<Self> {
        let fd = File::open(path.as_ref())?;
        let mmap = unsafe { Mmap::map(&fd) }?;

        // TODO(kto2828): Implement stats calculation
        let mut start_ns = u64::MAX;
        let mut end_ns = u64::MIN;

        let stream = MessageStream::new(&mmap)?;

        for message_result in stream.into_iter() {
            let message = message_result?;

            let log_time = message.log_time;

            if log_time < start_ns {
                start_ns = log_time;
            }
            if log_time > end_ns {
                end_ns = log_time;
            }
        }

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
        use std::collections::HashMap;

        let stream = self.as_stream()?;

        let topic_map: Result<HashMap<String, Topic>, anyhow::Error> = stream.into_iter().try_fold(
            HashMap::<String, Topic>::new(),
            |mut acc, message_result| {
                let message = message_result.map_err(anyhow::Error::from)?;
                let topic_name = message.channel.topic.clone();

                acc.entry(topic_name.clone())
                    .and_modify(|topic| topic.count += 1)
                    .or_insert_with(|| Topic {
                        id: message.channel.id,
                        name: topic_name,
                        type_name: message
                            .channel
                            .schema
                            .as_ref()
                            .map(|s| s.name.clone())
                            .unwrap_or_default(),
                        count: 1,
                        serialization_format: message.channel.message_encoding.clone(),
                        offered_qos_profiles: None,
                    });

                Ok(acc)
            },
        );

        topic_map.map(|map| map.into_values().collect())
    }

    fn read_messages(&self, topic_name: &str) -> RosPeekResult<Vec<RawMessage>> {
        let stream = self.as_stream()?;

        stream
            .into_iter()
            .filter_map(|message_result| match message_result {
                Ok(message) if message.channel.topic == topic_name => Some(Ok(RawMessage {
                    timestamp: message.publish_time,
                    topic_id: message.channel.id,
                    data: message.data.into(),
                })),
                Ok(_) => None, // Skip messages from other topics
                Err(e) => Some(Err(anyhow::Error::from(e))), // Convert McapError to anyhow::Error
            })
            .collect::<RosPeekResult<Vec<RawMessage>>>()
    }
}
