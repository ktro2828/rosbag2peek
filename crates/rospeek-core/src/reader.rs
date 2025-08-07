use std::path::Path;

use crate::{RawMessage, RosPeekResult, Topic};

pub trait BagReader: Send {
    fn open<P: AsRef<Path>>(path: P) -> RosPeekResult<Self>
    where
        Self: Sized;

    fn topics(&self) -> RosPeekResult<Vec<Topic>>;

    fn read_messages(&self, topic_name: &str) -> RosPeekResult<Vec<RawMessage>>;
}
