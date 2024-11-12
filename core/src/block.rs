use crate::transaction::Transaction;
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

// Structure of a block
#[derive(Debug, Clone, Serialize, Deserialize, BorshDeserialize, BorshSerialize, PartialEq, Eq)]
pub struct Block {
    index: u64,
    timestamp: u32,
    transactions: Vec<Transaction>,
    nonce: String,
    previous_hash: String,
    hash: String,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct BlockChain {
    blocks: Vec<Block>,
}

impl Block {}
