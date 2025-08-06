pub mod error;
pub mod model;
pub mod reader;

pub use error::{RosbagError, RosbagResult};
pub use model::{RawMessage, Topic};
pub use reader::BagReader;
