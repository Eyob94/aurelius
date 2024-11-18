use corelib::{block::Block, blockchain::BlockChain, transaction::Transaction, utxo::UTXO};
use std::{collections::HashSet, io::Read, time::Duration};

use anyhow::{anyhow, bail};
use tokio::{
    io::{AsyncReadExt as _, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
use tracing::{error, info};

use crate::mempool::MemPool;

#[derive(Default, Debug, Clone)]
pub struct Node {
    id: String,
    mem_pool: MemPool,
    utxo_set: HashSet<UTXO>,
    peers: Vec<Node>,
    blockchain: Option<BlockChain>,
    current_block: Option<Block>,
    pending_blocks: Vec<Block>,
}

impl Node {
    pub fn new() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            mem_pool: MemPool::new(),
            utxo_set: HashSet::new(),
            peers: Vec::new(),
            blockchain: None,
            current_block: None,
            pending_blocks: Vec::new(),
        }
    }

    fn validate_transaction(&self, transaction: &Transaction) -> anyhow::Result<()> {
        let n = transaction.verify("")?;

        Ok(())
    }
}
