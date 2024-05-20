// src/bin/send_transaction.rs

use blockdag::blockdag::BlockDAG;
use blockdag::transaction::Transaction;
use blockdag::wallet::Wallet;
use ed25519_dalek::Keypair;
use hex;
use std::sync::{Arc, Mutex};
use std::io::{self, Write};

fn main() {
    let dag = Arc::new(Mutex::new(BlockDAG::new()));

    let mut sender_address = String::new();
    let mut receiver_address = String::new();
    let mut amount = String::new();
    let mut private_key_hex = String::new();
    let mut public_key_hex = String::new();

    println!("Enter sender address:");
    io::stdin().read_line(&mut sender_address).unwrap();

    println!("Enter receiver address:");
    io::stdin().read_line(&mut receiver_address).unwrap();

    println!("Enter amount:");
    io::stdin().read_line(&mut amount).unwrap();

    println!("Enter private key:");
    io::stdin().read_line(&mut private_key_hex).unwrap();

    println!("Enter public key:");
    io::stdin().read_line(&mut public_key_hex).unwrap();

    let amount = amount.trim().parse::<u64>().expect("Invalid amount");

    let private_key_bytes = hex::decode(private_key_hex.trim()).expect("Invalid private key hex");
    let public_key_bytes = hex::decode(public_key_hex.trim()).expect("Invalid public key hex");

    let keypair = Keypair::from_bytes(&[&private_key_bytes[..], &public_key_bytes[..]].concat()).expect("Invalid keypair");
    let wallet = Wallet { keypair };

    let transaction = Transaction::new(
        sender_address.trim().to_string(),
        receiver_address.trim().to_string(),
        amount,
        0, // transaction fee
        wallet.sign(&format!("{}{}{}", sender_address.trim(), receiver_address.trim(), amount)),
    );

    {
        let mut dag = dag.lock().unwrap();
        dag.add_transaction(transaction);
        println!("Transaction added successfully!");
    }
}
