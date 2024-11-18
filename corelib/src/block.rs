use crate::{errors::Result, transaction::Transaction};
use borsh::{BorshDeserialize, BorshSerialize};

// Structure of a block
#[derive(Debug, Clone, BorshDeserialize, BorshSerialize, PartialEq, Eq)]
pub struct Block {
    // Block height of the block
    index: u64,
    // Timestamp the block was "Mined"
    timestamp: u32,
    // Collection of transactions included in this block
    transactions: Vec<Transaction>,
    //
    nonce: String,
    // Hash of the previous block
    previous_hash: String,

    hash: String,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct BlockChain {
    blocks: Vec<Block>,
}

impl Block {
    pub fn new(transactions: Vec<Transaction>, index: u64) -> Result<Self> {
        todo!()
    }
    fn create_merkle_root(&mut self) -> Result<()> {
        todo!()
    }
}
