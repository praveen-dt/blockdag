// src/generate_address.rs

use blockdag::wallet::Wallet;

fn main() {
    // Generate a new wallet
    let wallet = Wallet::new();

    // Get the public address
    let public_address = wallet.get_address();

    // Get the private key
    let private_key = hex::encode(wallet.keypair.secret.to_bytes());

    // Display the public address and private key
    println!("Public Address: {}", public_address);
    println!("Private Key: {}", private_key);
}
