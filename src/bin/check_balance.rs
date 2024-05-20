// src/bin/check_balance.rs

use blockdag::blockdag::BlockDAG;
use std::sync::{Arc, Mutex};
use std::io::{self, Write};

fn main() {
    let dag = Arc::new(Mutex::new(BlockDAG::new()));

    let mut address = String::new();

    println!("Enter address to check balance:");
    io::stdin().read_line(&mut address).unwrap();

    {
        let dag = dag.lock().unwrap();
        
        // Print the DAG structure for debugging
        dag.print_dag();
        
        let balance = dag.get_balance(&address.trim());
        println!("Balance for address {}: {}", address.trim(), balance);
    }
}
