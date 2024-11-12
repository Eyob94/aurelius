use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

#[allow(unused)]
#[derive(Debug, Clone, Deserialize, Serialize, BorshSerialize, BorshDeserialize, PartialEq, Eq)]
pub struct Transaction {
    sender: String,
    receiver: String,
    amount: String,
    timestamp: u32,
    signature: String,
}
