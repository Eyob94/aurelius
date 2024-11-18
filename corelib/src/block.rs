use std::time::{SystemTime, UNIX_EPOCH};

use crate::{errors::Result, transaction::Transaction};
use borsh::{BorshDeserialize, BorshSerialize};

// Structure of a block
#[derive(Debug, Clone, BorshDeserialize, BorshSerialize, PartialEq, Eq)]
pub struct Block {
    // Block height of the block
    index: u64,
    // Timestamp the block was "Mined"
    timestamp: u128,
    // Collection of transactions included in this block
    transactions: Vec<Transaction>,
    //
    nonce: u64,
    // Hash of the previous block
    previous_hash: String,

    // Hash of the entire block
    hash: [u8; 32],

    difficulty: u32,
}

impl Block {
    pub fn new(
        index: u64,
        transactions: Vec<Transaction>,
        previous_hash: String,
        difficulty: u32,
    ) -> Result<Self> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis();

        let mut block = Block {
            index,
            timestamp,
            transactions,
            nonce: 0,
            previous_hash,
            hash: [0u8; 32],
            difficulty,
        };

        block.mine_block();
        Ok(block)
    }
    pub fn calculate_hash(&self) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();

        hasher.update(&self.index.to_le_bytes());
        hasher.update(&self.timestamp.to_le_bytes());
        self.transactions.iter().for_each(|t| {
            hasher.update(&t.hash_id);
        });

        hasher.update(&self.nonce.to_le_bytes());
        hasher.update(self.previous_hash.as_bytes());

        let result = hasher.finalize();
        *result.as_bytes()
    }

    pub fn mine_block(&mut self) {
        let target_prefix = vec![0u8; (self.difficulty as usize + 7) / 8];
        let target_bits = self.difficulty as usize % 8;

        loop {
            self.hash = self.calculate_hash();

            // Check if the first 'difficulty' bits are zero
            let mut meets_difficulty = true;
            for (i, &byte) in self.hash.iter().enumerate() {
                if i >= target_prefix.len() {
                    break;
                }
                if i == target_prefix.len() - 1 && target_bits > 0 {
                    // Check partial byte
                    if byte >> (8 - target_bits) != 0 {
                        meets_difficulty = false;
                    }
                } else if byte != 0 {
                    meets_difficulty = false;
                }
            }

            if meets_difficulty {
                println!("Block mined! Hash: {}", hex::encode(self.hash));
                break;
            }

            self.nonce = self.nonce.wrapping_add(1);
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        block::*,
        test_utils::{generate_key_pairs, generate_random_utxos},
        transaction::Transaction,
    };

    #[test]
    fn test_block_hashing() {
        let (mut signing_key, _, sender, receiver) = generate_key_pairs().unwrap();
        let mut transactions = vec![];

        let mut txn1 = Transaction::new(&mut signing_key, receiver).unwrap();
        let (input_utxo, output_utxo) = generate_random_utxos(sender, 1_000, 999).unwrap();
        txn1.add_inputs(input_utxo, &mut signing_key).unwrap();
        txn1.add_outputs(output_utxo, &mut signing_key).unwrap();

        let mut txn2 = Transaction::new(&mut signing_key, receiver).unwrap();
        let (input_utxo, output_utxo) = generate_random_utxos(sender, 1_000, 999).unwrap();
        txn2.add_inputs(input_utxo, &mut signing_key).unwrap();
        txn2.add_outputs(output_utxo, &mut signing_key).unwrap();

        transactions.push(txn1);
        transactions.push(txn2);

        let block = Block::new(
            1,
            transactions.clone(),
            "previous_hash_example".to_string(),
            1,
        )
        .unwrap();

        // Calculating hash manually to compare with block's hash
        let mut hasher = blake3::Hasher::new();
        hasher.update(&block.index.to_le_bytes());
        hasher.update(&block.timestamp.to_le_bytes());
        transactions.iter().for_each(|t| {
            hasher.update(&t.hash_id);
        });
        hasher.update(&block.nonce.to_le_bytes());
        hasher.update(block.previous_hash.as_bytes());

        let expected_hash = *hasher.finalize().as_bytes();
        assert_eq!(
            block.hash, expected_hash,
            "Block hash should be correctly calculated."
        );
    }

    #[test]
    fn test_block_mining() {
        let (mut signing_key, _, sender, receiver) = generate_key_pairs().unwrap();
        let mut transactions = vec![];

        let mut txn1 = Transaction::new(&mut signing_key, receiver).unwrap();
        let (input_utxo, output_utxo) = generate_random_utxos(sender, 1_000, 999).unwrap();
        txn1.add_inputs(input_utxo, &mut signing_key).unwrap();
        txn1.add_outputs(output_utxo, &mut signing_key).unwrap();

        transactions.push(txn1);

        let mut block =
            Block::new(1, transactions, "previous_hash_example".to_string(), 1).unwrap();

        // Attempt to mine the block with a given difficulty
        block.mine_block();
        // assert!(block.is_valid(4), "The mined block should be valid.");
    }
}
