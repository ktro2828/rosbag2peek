use std::string::{FromUtf8Error, FromUtf16Error};

use thiserror::Error;

/// TODO(ktro2828): Merge Rosbag** and Schema** into RosPeek**

#[derive(Debug, Error)]
pub enum RosPeekError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Topic not found: {0}")]
    TopicNotFound(String),

    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("IDL path not found: {0}")]
    IdlNotFound(String),

    #[error("Invalid UTF-8 data: {0}")]
    InvalidUtf8(#[from] FromUtf8Error),

    #[error("Invalid UTF-16 data: {0}")]
    InvalidUtf16(#[from] FromUtf16Error),

    #[error("Other: {0}")]
    Other(String),
}

pub type RosPeekResult<T> = Result<T, RosPeekError>;
