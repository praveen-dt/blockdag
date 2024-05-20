// src/block.rs

use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::transaction::Transaction;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub index: u64,
    pub timestamp: u128,
    pub previous_hashes: Vec<String>,
    pub hash: String,
    pub nonce: u64,
    pub transactions: Vec<Transaction>,
    pub weight: u64,
    pub reward: u64,
    pub difficulty: u64,
    pub message: Option<String>,
}

impl Block {
    pub fn new(index: u64, previous_hashes: Vec<String>, transactions: Vec<Transaction>, reward: u64, message: Option<String>, difficulty: u64) -> Block {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        let (nonce, hash, mining_time) = Block::mine_block(index, timestamp, &previous_hashes, &transactions, difficulty);
        println!("Block mined in {} ms with difficulty {}", mining_time, difficulty);
        Block { index, timestamp, previous_hashes, hash, nonce, transactions, weight: 0, reward, difficulty, message }
    }

    pub fn mine_block(index: u64, timestamp: u128, previous_hashes: &Vec<String>, transactions: &Vec<Transaction>, difficulty: u64) -> (u64, String, u128) {
        let start_time = SystemTime::now();
        let mut nonce = 0;
        let target_prefix = "0".repeat(difficulty as usize); // Adjust target prefix based on difficulty
        loop {
            let hash = Block::calculate_hash(index, timestamp, previous_hashes, nonce, transactions);
            if &hash[..difficulty as usize] == target_prefix {
                let end_time = SystemTime::now();
                let mining_time = end_time.duration_since(start_time).unwrap().as_millis();
                return (nonce, hash, mining_time);
            }
            nonce += 1;
        }
    }

    pub fn calculate_hash(index: u64, timestamp: u128, previous_hashes: &Vec<String>, nonce: u64, transactions: &Vec<Transaction>) -> String {
        let mut hasher = Sha256::new();
        hasher.update(format!("{}{}{:?}{}{:?}", index, timestamp, previous_hashes, nonce, transactions));
        let result = hasher.finalize();
        hex::encode(result)
    }
}
