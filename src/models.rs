use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TransferRecord {
    pub tx_hash: String,
    pub block_number: u64,
    pub log_index: u64,
    pub token_address: String,
    pub from_addr: String,
    pub to_addr: String,
    pub amount_raw: String,
    pub amount_normalized: f64,
    pub timestamp: i64,
    pub processed_at: i64,
}
