// src/main.rs

mod block;
mod blockdag;
mod messages;
mod network;
mod transaction;
mod wallet;
mod utils;
mod constants;

use crate::blockdag::BlockDAG;
use crate::network::{start_server, connect_to_server};
use crate::wallet::Wallet;
use crate::transaction::Transaction;
use crate::messages::Message;
use std::sync::{Arc, Mutex};
use std::collections::HashSet;
use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;
use tokio::runtime::Runtime;

fn main() {
    let dag = Arc::new(Mutex::new(BlockDAG::new()));
    let peers = Arc::new(Mutex::new(HashSet::new()));

    // Create a wallet and generate an address
    let wallet = Wallet::new();
    let address = wallet.get_address();
    println!("Wallet Address: {}", address);

    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let dag_server = dag.clone();
        let peers_server = peers.clone();

        tokio::spawn(async move {
            start_server(dag_server, 8080, peers_server).await;
        });

        // Simulate a client connection and create transactions
        tokio::spawn(async move {
            connect_to_server("127.0.0.1:8080", dag.clone(), peers.clone()).await;

            // Create a new transaction
            let receiver_address = "receiver_public_key".to_string();
            let amount = 10;
            let signature = wallet.sign(&format!("{}{}{}", address, receiver_address, amount));
            let transaction = Transaction::new(address.clone(), receiver_address, amount, signature);

            let msg = Message::NewTransaction(transaction).to_string();
            if let Ok(mut socket) = TcpStream::connect("127.0.0.1:8080").await {
                socket.write_all(msg.as_bytes()).await.unwrap();
            }

            let mut dag = dag.lock().unwrap();
            if let Some(new_block) = dag.create_block() {
                println!("New Block Created: {:?}", new_block);
            } else {
                println!("Failed to create a new block. Total supply might have been reached.");
            }

            dag.ghostdag();
            dag.display();
        });

        loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    });
}
