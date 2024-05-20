// src/bin/check_balance.rs

use blockdag::blockdag::BlockDAG;
use tokio::io::{self, AsyncBufReadExt, BufReader};

#[tokio::main]
async fn main() {
    // Load BlockDAG from file
    let dag = BlockDAG::load_from_file("blockdag.json").expect("Failed to load BlockDAG from file");

    // Read address from stdin
    let mut reader = BufReader::new(io::stdin());
    println!("Enter address to check balance:");
    let mut address = String::new();
    reader.read_line(&mut address).await.expect("Failed to read address");
    let address = address.trim();

    // Get balance
    let balance = dag.get_balance(address);
    println!("Balance for address {}: {}", address, balance);
}
