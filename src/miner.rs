// src/bin/miner.rs

use std::sync::{Arc, Mutex};
use std::collections::HashSet;
use tokio::time::{sleep, Duration};
use blockdag::blockdag::BlockDAG;
use blockdag::network::{start_server, connect_to_server};

#[tokio::main]
async fn main() {
    // Hard-coded wallet address
    let wallet_address = "7bf2b2f920a612a724a490b7b2dbea0199f8ae4fa3f595a930cf5f4c0d446308";

    println!("Using Wallet Address: {}", wallet_address);

    let dag = Arc::new(Mutex::new(
        BlockDAG::load_from_file("blockdag.json").unwrap_or_else(|_| BlockDAG::new()),
    ));
    let peers = Arc::new(Mutex::new(HashSet::new()));

    // Start the server
    let dag_server = Arc::clone(&dag);
    let peers_server = Arc::clone(&peers);
    tokio::spawn(async move {
        start_server(dag_server, 8080, peers_server).await;
    });

    // Connect to the server (if needed)
    let dag_client = Arc::clone(&dag);
    let peers_client = Arc::clone(&peers);
    tokio::spawn(async move {
        sleep(Duration::from_secs(1)).await; // Wait a bit for the server to start
        connect_to_server("127.0.0.1:8080", dag_client, peers_client).await;
    });

    // Continuous mining loop
    loop {
        {
            let mut dag = dag.lock().unwrap();
            if let Some(new_block) = dag.create_block(wallet_address) {
                println!("New Block Created: {:?}", new_block);
                dag.save_to_file("blockdag.json").unwrap();
            }
            // Calculate and print balance
            let balance = dag.get_balance(wallet_address);
            println!("Current Balance: {}", balance);
        }
        // Simulate mining time
        sleep(Duration::from_secs(1)).await;
    }
}
