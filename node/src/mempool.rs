use corelib::transaction::Transaction;

#[derive(Default, Debug, Clone)]
pub struct MemPool {
    unverified_transactions: Vec<Transaction>,
    verified_transactions: Vec<Transaction>,
}

impl MemPool {
    pub fn new() -> Self {
        Self {
            unverified_transactions: Vec::new(),
            verified_transactions: Vec::new(),
        }
    }
}
