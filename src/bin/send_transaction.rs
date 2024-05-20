// src/bin/send_transaction.rs

use blockdag::transaction::Transaction;
use ed25519_dalek::{Keypair, PublicKey, SecretKey, Signature, Signer};
use std::io::{self, BufRead, Write};
use std::net::TcpStream;
use blockdag::messages::Message;
use hex;

fn main() {
    let mut sender_address = String::new();
    let mut receiver_address = String::new();
    let mut amount = String::new();
    let mut private_key_hex = String::new();
    let mut fee = String::new();

    let stdin = io::stdin();
    let mut stdin_lock = stdin.lock();

    println!("Enter sender address:");
    stdin_lock.read_line(&mut sender_address).unwrap();
    sender_address = sender_address.trim().to_string();

    println!("Enter receiver address:");
    stdin_lock.read_line(&mut receiver_address).unwrap();
    receiver_address = receiver_address.trim().to_string();

    println!("Enter amount:");
    stdin_lock.read_line(&mut amount).unwrap();
    let amount: u64 = amount.trim().parse().unwrap();

    println!("Enter private key:");
    stdin_lock.read_line(&mut private_key_hex).unwrap();
    private_key_hex = private_key_hex.trim().to_string();

    println!("Enter fee:");
    stdin_lock.read_line(&mut fee).unwrap();
    let fee: u64 = fee.trim().parse().unwrap();

    // Convert private key hex to SecretKey
    let private_key_bytes = hex::decode(&private_key_hex).expect("Invalid private key hex");
    let private_key = SecretKey::from_bytes(&private_key_bytes).expect("Invalid private key bytes");

    // Derive public key from private key
    let public_key: PublicKey = (&private_key).into();
    let sender_address_derived = hex::encode(public_key.to_bytes());

    // Ensure the derived public key matches the provided sender address
    if sender_address != sender_address_derived {
        panic!("Provided sender address does not match derived public key from the private key");
    }

    // Create and sign transaction
    let transaction = Transaction::new(sender_address.clone(), receiver_address, amount, fee, "".to_string());
    let message = transaction.calculate_hash();
    let signature = Keypair { public: public_key, secret: private_key }.sign(message.as_bytes());
    let signed_transaction = Transaction {
        signature: hex::encode(signature.to_bytes()),
        ..transaction
    };

    // Connect to server and send transaction
    let mut stream = TcpStream::connect("127.0.0.1:8080").expect("Could not connect to server");
    let message = Message::NewTransaction(signed_transaction);
    let serialized_message = serde_json::to_string(&message).unwrap();
    stream.write_all(serialized_message.as_bytes()).expect("Failed to send transaction");

    println!("Transaction added successfully!");
}
