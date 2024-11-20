#![allow(unused)]

use corelib::{block::Block, transaction::Transaction, utxo::UTXO};
use std::{collections::HashSet, io::Read, time::Duration};

use anyhow::anyhow;
use node::Node;
use tokio::{
    io::{AsyncReadExt as _, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
use tracing::{error, info};

pub mod errors;
mod node;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let node = Node::new();
}
