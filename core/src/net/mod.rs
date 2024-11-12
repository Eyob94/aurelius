pub mod message;
pub mod protocol;

use borsh::{BorshDeserialize, BorshSerialize};
use message::Message;
use tokio::net::TcpListener;

use crate::errors::{self, Result};

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct Payload {
    version: u8,
    message: Message,
    checksum: u64,
}

pub async fn start_listening(port: u16) -> Result<TcpListener> {
    let listener = TcpListener::bind(format!("127.0.0.1:{port}"))
        .await
        .map_err(|_| errors::Error::Network)?;

    Ok(listener)
}
