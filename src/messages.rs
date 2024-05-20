// src/messages.rs

use crate::block::Block;
use crate::transaction::Transaction;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    RequestBlock(String),
    NewBlock(Block),
    RequestTip,
    Tip(String),
    NewTransaction(Transaction),
    Unknown,
}

impl Message {
    pub fn from_str(s: &str) -> Self {
        serde_json::from_str(s).unwrap_or(Message::Unknown)
    }

    pub fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}
