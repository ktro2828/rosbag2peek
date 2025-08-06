use thiserror::Error;

#[derive(Error, Debug)]
pub enum RosbagError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Topic not found: {0}")]
    TopicNotFound(String),

    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("Other: {0}")]
    Other(String),
}

pub type RosbagResult<T> = Result<T, RosbagError>;
