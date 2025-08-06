use std::path::Path;

use crate::{RawMessage, RosbagResult, Topic};

pub trait BagReader: Send {
    fn open<P: AsRef<Path>>(path: P) -> RosbagResult<Self>
    where
        Self: Sized;

    fn topics(&self) -> RosbagResult<Vec<Topic>>;

    fn read_messages(&self, topic_name: &str) -> RosbagResult<Vec<RawMessage>>;
}
