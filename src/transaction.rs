// src/transaction.rs

use serde::{Serialize, Deserialize};
use sha2::Digest;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub sender: String,
    pub receiver: String,
    pub amount: u64,
    pub fee: u64,
    pub signature: String,
}

impl Transaction {
    pub fn new(sender: String, receiver: String, amount: u64, fee: u64, signature: String) -> Self {
        Transaction { sender, receiver, amount, fee, signature }
    }

    pub fn calculate_hash(&self) -> String {
        let data = format!("{}{}{}{}", self.sender, self.receiver, self.amount, self.fee);
        let mut hasher = sha2::Sha256::new();
        hasher.update(data);
        hex::encode(hasher.finalize())
    }
}
