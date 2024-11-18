use std::time::SystemTimeError;

use ed25519_dalek::{ed25519, SignatureError};
use hex::FromHexError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Network Error")]
    Network,

    #[error("Error serializing/deserializing")]
    IO(#[from] std::io::Error),

    #[error("Protocol Error: {0}")]
    Protocol(#[from] ProtocolError),

    #[error("Owner mismatch")]
    OwnerMismatch,

    #[error("Signature Error: {0}")]
    Signature(#[from] ed25519::Error),

    #[error("Error working with system time")]
    TimeError(#[from] SystemTimeError),

    #[error("Insufficient funds to carry out transaction")]
    InsufficientFunds,

    #[error("Unauthorized to perform action")]
    UnAuthorized,

    #[error("UTXO not confirmed")]
    PendingUTXO,

    #[error("UTXO already confirmed")]
    ConfirmedUTXO,

    #[error("Invalid UTXO value")]
    InvalidUTXOValue,

    #[error("Invalid unlocking script used")]
    InvalidUnlockingScript,

    #[error("Invalid u8 length: length {0}")]
    InvalidU8Length(usize),

    #[error("Empty Stack")]
    EmptyStack,

    #[error("Error decoding hexcode")]
    HexcodeError(#[from] FromHexError),
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
