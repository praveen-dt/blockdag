// src/blockdag.rs

use std::collections::{HashMap, HashSet};
use crate::block::Block;
use crate::transaction::Transaction;
use crate::wallet::Wallet;
use crate::constants::{INITIAL_BLOCK_REWARD, HALVING_INTERVAL, TARGET_BLOCK_TIME, DIFFICULTY_ADJUSTMENT_INTERVAL, TOTAL_SUPPLY};

pub struct BlockDAG {
    pub blocks: HashMap<String, Block>,
    pub tips: Vec<String>,
    pub pending_transactions: Vec<Transaction>,
    pub current_supply: u64,
    pub difficulty: u64,
    pub block_times: Vec<u128>, // Track block mining times
    pub block_count: u64,       // Track the number of blocks mined
    pub current_block_reward: u64, // Track the current block reward
}

impl BlockDAG {
    pub fn new() -> BlockDAG {
        let genesis_message = Some("Genesis Block - Welcome to BlockDAG!".to_string());
        let initial_difficulty = 4;
        let genesis_block = Block::new(0, vec!["0".to_string()], vec![], 0, genesis_message, initial_difficulty);
        let genesis_hash = genesis_block.hash.clone();
        let mut blocks = HashMap::new();
        blocks.insert(genesis_hash.clone(), genesis_block);
        BlockDAG {
            blocks,
            tips: vec![genesis_hash],
            pending_transactions: vec![],
            current_supply: 0,
            difficulty: initial_difficulty,
            block_times: vec![],
            block_count: 1, // Start with the genesis block
            current_block_reward: INITIAL_BLOCK_REWARD,
        }
    }

    pub fn add_transaction(&mut self, transaction: Transaction) {
        self.pending_transactions.push(transaction);
    }

    pub fn create_block(&mut self, miner_address: &str) -> Option<Block> {
        if self.current_supply >= TOTAL_SUPPLY {
            println!("Total supply reached. No more blocks can be created.");
            return None;
        }

        let previous_hashes = self.tips.clone();
        let index = self.blocks.len() as u64;
        let transactions = self.pending_transactions.clone();

        // Add the mining reward transaction
        let reward = std::cmp::min(self.current_block_reward, TOTAL_SUPPLY - self.current_supply);
        let reward_transaction = Transaction::new("0".to_string(), miner_address.to_string(), reward, 0, "".to_string());
        let mut block_transactions = transactions;
        block_transactions.push(reward_transaction);

        let new_block = Block::new(index, previous_hashes.clone(), block_transactions, reward, None, self.difficulty);
        let new_hash = new_block.hash.clone();

        // Validate the new block
        if self.validate_block(&new_block) {
            // Update the DAG
            self.blocks.insert(new_hash.clone(), new_block.clone());
            self.update_tips(new_hash);
            self.current_supply += reward;
            self.block_times.push(new_block.timestamp); // Track block timestamp
            self.block_count += 1; // Increment block count

            // Adjust difficulty if needed
            if self.blocks.len() % DIFFICULTY_ADJUSTMENT_INTERVAL == 0 {
                self.adjust_difficulty();
            }

            // Halve the block reward if necessary
            if self.block_count % HALVING_INTERVAL == 0 {
                self.current_block_reward /= 2;
                println!("Block reward halved to {}", self.current_block_reward);
            }

            // Clear pending transactions
            self.pending_transactions.clear();
            return Some(new_block);
        }

        None
    }

    pub fn validate_block(&self, block: &Block) -> bool {
        // Check if all previous hashes exist in the DAG
        for hash in &block.previous_hashes {
            if hash != "0" && !self.blocks.contains_key(hash) {
                return false;
            }
        }

        // Validate the block's hash
        let calculated_hash = Block::calculate_hash(block.index, block.timestamp, &block.previous_hashes, block.nonce, &block.transactions);
        if calculated_hash != block.hash {
            return false;
        }

        // Ensure the hash meets the difficulty target
        let target_prefix = "0".repeat(block.difficulty as usize);
        if &block.hash[..block.difficulty as usize] != target_prefix {
            return false;
        }

        // Validate transactions (simplified for this example)
        for tx in &block.transactions {
            if tx.sender != "0" {
                let sender_pub_key = hex::decode(&tx.sender).expect("Invalid sender hex");
                let public_key = ed25519_dalek::PublicKey::from_bytes(&sender_pub_key).expect("Invalid public key bytes");
                if !Wallet::verify(&public_key, &tx.calculate_hash(), &tx.signature) {
                    return false;
                }
            }
        }

        true
    }

    pub fn adjust_difficulty(&mut self) {
        let len = self.block_times.len();
        if len < DIFFICULTY_ADJUSTMENT_INTERVAL {
            return; // Not enough blocks to adjust difficulty yet
        }

        let start_time = self.block_times[len - DIFFICULTY_ADJUSTMENT_INTERVAL];
        let end_time = self.block_times[len - 1];
        let actual_time = end_time - start_time;
        let expected_time = TARGET_BLOCK_TIME * DIFFICULTY_ADJUSTMENT_INTERVAL as u128;

        if actual_time < expected_time / 2 {
            self.difficulty += 1;
            println!("Difficulty increased to {}", self.difficulty);
        } else if actual_time > expected_time * 2 {
            if self.difficulty > 1 {
                self.difficulty -= 1;
                println!("Difficulty decreased to {}", self.difficulty);
            }
        }
    }

    pub fn update_tips(&mut self, new_hash: String) {
        // Remove blocks that are parents of the new block from tips
        if let Some(new_block) = self.blocks.get(&new_hash) {
            let mut new_tips = self.tips.clone();
            for parent_hash in &new_block.previous_hashes {
                if parent_hash != "0" {
                    new_tips.retain(|hash| hash != parent_hash);
                }
            }
            // Add the new block to the tips
            new_tips.push(new_hash);
            self.tips = new_tips;
        } else {
            println!("Error: Block not found in DAG during tip update");
        }
    }

    pub fn ghostdag(&mut self) {
        // Implement the GHOSTDAG algorithm to determine the heaviest sub-tree
        let mut visited = HashSet::new();
        let mut weights = HashMap::new();

        // Initialize weights
        for (hash, block) in &self.blocks {
            weights.insert(hash.clone(), block.weight);
        }

        // Compute weights recursively
        for tip in &self.tips {
            self.compute_weights(tip, &mut visited, &mut weights);
        }

        // Update block weights
        for (hash, weight) in weights {
            if let Some(block) = self.blocks.get_mut(&hash) {
                block.weight = weight;
            }
        }
    }

    fn compute_weights(&self, hash: &String, visited: &mut HashSet<String>, weights: &mut HashMap<String, u64>) -> u64 {
        if visited.contains(hash) {
            return *weights.get(hash).unwrap_or(&0);
        }

        visited.insert(hash.clone());

        let block = match self.blocks.get(hash) {
            Some(block) => block,
            None => {
                if hash == "0" {
                    return 0; // Genesis block parent hash is "0", so return 0 weight
                }
                println!("Error: Block not found in DAG with hash {}", hash);
                return 0;
            }
        };

        let mut weight = block.transactions.len() as u64 + 1; // Including the block itself

        for parent_hash in &block.previous_hashes {
            weight += self.compute_weights(parent_hash, visited, weights);
        }

        weights.insert(hash.clone(), weight);
        weight
    }

    pub fn get_balance(&self, address: &str) -> u64 {
        let mut balance = 0;
        for block in self.blocks.values() {
            for tx in &block.transactions {
                if tx.receiver == address {
                    balance += tx.amount;
                }
                if tx.sender == address {
                    balance -= tx.amount;
                }
            }
        }
        balance
    }

    pub fn display(&self) {
        for (hash, block) in &self.blocks {
            println!("Block Hash: {}", hash);
            println!("Block Data: {:?}", block);
            println!("Block Weight: {}", block.weight);
            println!("Block Reward: {}", block.reward);
            println!("Block Difficulty: {}", block.difficulty);
            if let Some(ref message) = block.message {
                println!("Block Message: {}", message);
            }
        }
        println!("Current Supply: {}", self.current_supply);
        println!("Current Block Reward: {}", self.current_block_reward);
        println!("Block Count: {}", self.block_count);
    }
}
