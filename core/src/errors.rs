use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Network Error")]
    Network,

    #[error("Error serializing/deserializing")]
    IO(#[from] std::io::Error),

    #[error("Protocol Error: {0}")]
    Protocol(#[from] ProtocolError),
}

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("Invalid message format")]
    InvalidMessageFormat,

    #[error("Unsupported command type: {0}")]
    UnsupportedCommand(u8),

    #[error("Unsupported status code: {0}")]
    UnsupportedStatusCode(u8),

    #[error("Header mismatch or payload size mismatch")]
    HeaderMismatch,

    #[error("Unknown protocol version: {0}")]
    UnknownVersion(u16),

    #[error("Error serializing: {0}")]
    SerializationError(String),
}

pub type Result<T> = std::result::Result<T, Error>;
