use std::io::Write;

use borsh::{BorshDeserialize, BorshSerialize};

use crate::{block::Block, errors::Result, transaction::Transaction};

#[allow(unused)]
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq, Eq)]
pub enum Message {
    PaymentTransaction(Transaction),
    Utxo(Vec<String>),

    BlockProposal(Block),
    BlockConfirmation(String),

    PeerIntroduction(String),

    BlockRequest(u64),
    BlockResponse(Block),

    InvalidTransactionAlert(String),

    Ping,
}

pub fn deserialize(message: &[u8]) -> Result<Message> {
    let deserialized_msg = borsh::de::from_slice::<Message>(message).map_err(|e| {
        crate::errors::Error::Protocol(crate::errors::ProtocolError::SerializationError(
            e.to_string(),
        ))
    })?;

    Ok(deserialized_msg)
}

pub fn serialize(node_message: &Message, mut writer: impl Write) -> Result<()> {
    node_message.serialize(&mut writer).map_err(|e| {
        crate::errors::Error::Protocol(crate::errors::ProtocolError::SerializationError(
            e.to_string(),
        ))
    })?;
    Ok(())
}
