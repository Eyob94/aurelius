use borsh::{BorshDeserialize, BorshSerialize};

use crate::block::Block;

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct BlockChain {
    blocks: Vec<Block>,
}
