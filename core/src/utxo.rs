use serde::{Deserialize, Serialize};

#[allow(clippy::style)]
#[derive(Deserialize, Serialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct UTXO {
    value: u64,
    txn_id: String,
    index: u32,
    key: String,
    created_at: u32,
    block_height: u32,
}
