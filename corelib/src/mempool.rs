use std::{
    collections::{BinaryHeap, HashMap},
    time::{SystemTime, UNIX_EPOCH},
};

use borsh::{BorshDeserialize, BorshSerialize};

use crate::{
    errors::{Error, Result},
    transaction::Transaction,
};

#[derive(Debug, Clone)]
pub struct MemPool {
    pub transactions: HashMap<[u8; 32], Transaction>,
    pub priority_queue: BinaryHeap<PriorityEntry>,
    pub max_size: usize,
}

impl BorshSerialize for MemPool {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        // Serialize max_size
        self.max_size.serialize(writer)?;

        // Serialize transactions
        let txn_vec: Vec<(&[u8; 32], &Transaction)> = self.transactions.iter().collect();
        txn_vec.serialize(writer)?;

        // Serialize priority_queue
        let priority_vec: Vec<&PriorityEntry> = self.priority_queue.iter().collect();
        priority_vec.serialize(writer)?;

        Ok(())
    }
}

impl BorshDeserialize for MemPool {
    fn deserialize_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        // Deserialize max_size
        let max_size = usize::deserialize_reader(reader)?;

        // Deserialize transactions
        let txn_vec: Vec<([u8; 32], Transaction)> = Vec::deserialize_reader(reader)?;
        let transactions = txn_vec.into_iter().collect();

        // Deserialize priority_queue
        let priority_vec: Vec<PriorityEntry> = Vec::deserialize_reader(reader)?;
        let priority_queue = BinaryHeap::from(priority_vec);

        Ok(Self {
            transactions,
            priority_queue,
            max_size,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct PriorityEntry {
    pub fee_per_byte: u64,
    pub timestamp: u128,
    pub size: u64,
    pub txn_hash: [u8; 32],
}

impl PartialOrd for PriorityEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PriorityEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other
            .fee_per_byte
            .cmp(&self.fee_per_byte)
            .then_with(|| self.timestamp.cmp(&other.timestamp))
    }
}

impl MemPool {
    pub fn new(max_size: usize) -> Self {
        MemPool {
            transactions: HashMap::new(),
            priority_queue: BinaryHeap::new(),
            max_size,
        }
    }

    pub fn add_transaction(&mut self, txn: Transaction, fee: u64) -> Result<()> {
        let txn_hash = txn.hash_id;

        if self.transactions.contains_key(&txn_hash) {
            return Err(Error::TxnExistInMempool);
        }

        let size = txn.size() as u64;
        let fee_per_byte = fee / size;

        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();

        let entry = PriorityEntry {
            fee_per_byte,
            size,
            timestamp,
            txn_hash,
        };

        // If number of transactions is greater than max size,
        // check for the least prioritized transaction
        // if the new transaction has a higher fee per byte remove the older and
        // add the new or vice versa
        if self.transactions.len() >= self.max_size {
            if let Some(lowest_priority) = self.priority_queue.peek() {
                if lowest_priority.fee_per_byte < entry.fee_per_byte {
                    if let Some(removed) = self.priority_queue.pop() {
                        self.remove_transaction(&removed.txn_hash);
                    }
                } else {
                    return Err(Error::TxnLowFee);
                }
            }
        }

        self.transactions.insert(txn_hash, txn);
        self.priority_queue.push(entry);

        Ok(())
    }

    pub fn remove_transaction(&mut self, tx_hash: &[u8; 32]) -> Option<Transaction> {
        self.priority_queue = self
            .priority_queue
            .clone()
            .into_iter()
            .filter(|entry| &entry.txn_hash != tx_hash)
            .collect::<BinaryHeap<_>>();
        self.transactions.remove(tx_hash)
    }

    pub fn get_transactions_for_block(&mut self, max_block_size: usize) -> Vec<Transaction> {
        let mut block_txns = vec![];
        let mut block_size = 0;

        while let Some(entry) = self.priority_queue.peek() {
            if block_size + entry.size < max_block_size as u64 {
                if let Some(txn) = self.transactions.get(&entry.txn_hash) {
                    block_txns.push(txn.clone());
                    block_size += entry.size;
                } else {
                    self.priority_queue.pop();
                }
            } else {
                break;
            }
        }

        block_txns.iter().for_each(|t| {
            self.remove_transaction(&t.hash_id);
        });

        block_txns
    }
}

#[cfg(test)]
mod test {

    use crate::test_utils::create_mock_transaction;

    use super::*;

    #[test]
    fn test_add_transaction() {
        let mut mempool = MemPool::new(5);
        let (txn1, us1) = create_mock_transaction(1000, 999);
        let (_, _, fee) = txn1.verify(&us1).unwrap();
        assert!(mempool.add_transaction(txn1, fee).is_ok());

        assert!(mempool.transactions.len() == 1);

        let (txn2, us2) = create_mock_transaction(1000, 996);
        let (_, _, fee) = txn2.verify(&us2).unwrap();
        assert!(mempool.add_transaction(txn2.clone(), fee).is_ok());
        assert!(mempool.transactions.len() == 2);

        let result = mempool.add_transaction(txn2, fee);

        match result {
            Ok(_) => panic!("Shouldn't work"),
            Err(Error::TxnExistInMempool) => println!("Passed"),
            Err(e) => panic!("shoundn't have given this error:{}", e),
        }
    }

    #[test]
    fn reject_low_fee() {
        let mut mempool = MemPool::new(1);
        let (txn1, us1) = create_mock_transaction(1000000, 99000);
        let (_, _, fee) = txn1.verify(&us1).unwrap();
        mempool.add_transaction(txn1.clone(), fee).unwrap();

        let (txn2, us2) = create_mock_transaction(1000, 996);
        let (_, _, fee) = txn2.verify(&us2).unwrap();
        assert!(mempool.add_transaction(txn2.clone(), fee).is_err());

        assert!(mempool.transactions.contains_key(&txn1.hash_id))
    }
}
