use std::string::FromUtf8Error;

use thiserror::Error;

/// TODO(ktro2828): Merge Rosbag** and Schema** into RosPeek**

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

#[derive(Error, Debug)]
pub enum SchemaError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("IDL path not found: {0}")]
    IdlNotFound(String),

    #[error("Invalid data: {0}")]
    InvalidData(#[from] FromUtf8Error),
}

pub type SchemaResult<T> = Result<T, SchemaError>;
