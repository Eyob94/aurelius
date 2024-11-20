use borsh::{BorshDeserialize, BorshSerialize};

use crate::{block::Block, mempool::MemPool};

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct BlockChain {
    blocks: Vec<Block>,
    difficulty: u32,
    mempool: MemPool
}

impl BlockChain{

}

