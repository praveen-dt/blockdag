// src/network.rs

use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::collections::HashSet;
use crate::blockdag::BlockDAG;
use crate::messages::Message;
use std::sync::{Arc, Mutex};

pub async fn start_server(dag: Arc<Mutex<BlockDAG>>, port: u16, peers: Arc<Mutex<HashSet<String>>>) {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await.unwrap();
    println!("Server running on port {}", port);

    loop {
        let (socket, addr) = listener.accept().await.unwrap();
        let dag = dag.clone();
        let peers = peers.clone();

        tokio::spawn(async move {
            handle_connection(dag, peers, socket, addr.to_string()).await;
        });
    }
}

pub async fn handle_connection(dag: Arc<Mutex<BlockDAG>>, peers: Arc<Mutex<HashSet<String>>>, mut socket: TcpStream, addr: String) {
    let mut buffer = [0; 1024];

    match socket.read(&mut buffer).await {
        Ok(size) if size > 0 => {
            let message = String::from_utf8_lossy(&buffer[..size]);
            let msg = Message::from_str(&message);
            handle_message(dag, peers, msg, socket, addr).await;
        }
        _ => (),
    }
}

pub async fn handle_message(dag: Arc<Mutex<BlockDAG>>, peers: Arc<Mutex<HashSet<String>>>, msg: Message, mut socket: TcpStream, addr: String) {
    match msg {
        Message::RequestBlock(hash) => {
            let block = {
                match dag.lock() {
                    Ok(dag_guard) => dag_guard.blocks.get(&hash).cloned(),
                    Err(poisoned) => {
                        let dag_guard = poisoned.into_inner();
                        dag_guard.blocks.get(&hash).cloned()
                    }
                }
            };

            if let Some(block) = block {
                let response = Message::NewBlock(block).to_string();
                socket.write_all(response.as_bytes()).await.unwrap();
            }
        }
        Message::NewBlock(block) => {
            match dag.lock() {
                Ok(mut dag_guard) => {
                    if dag_guard.validate_block(&block) {
                        dag_guard.blocks.insert(block.hash.clone(), block.clone());
                        dag_guard.update_tips(block.hash.clone());
                        println!("New block added: {:?}", block);
                    } else {
                        println!("Invalid block received: {:?}", block);
                    }
                }
                Err(poisoned) => {
                    let mut dag_guard = poisoned.into_inner();
                    if dag_guard.validate_block(&block) {
                        dag_guard.blocks.insert(block.hash.clone(), block.clone());
                        dag_guard.update_tips(block.hash.clone());
                        println!("New block added: {:?}", block);
                    } else {
                        println!("Invalid block received: {:?}", block);
                    }
                }
            }
        }
        Message::RequestTip => {
            let tips = {
                match dag.lock() {
                    Ok(dag_guard) => dag_guard.tips.clone(),
                    Err(poisoned) => {
                        let dag_guard = poisoned.into_inner();
                        dag_guard.tips.clone()
                    }
                }
            };

            if !tips.is_empty() {
                let response = Message::Tip(tips[0].clone()).to_string();
                socket.write_all(response.as_bytes()).await.unwrap();
            }
        }
        Message::Tip(hash) => {
            println!("Received tip from peer: {}", hash);
            let need_request_block = {
                match dag.lock() {
                    Ok(dag_guard) => !dag_guard.blocks.contains_key(&hash),
                    Err(poisoned) => {
                        let dag_guard = poisoned.into_inner();
                        !dag_guard.blocks.contains_key(&hash)
                    }
                }
            };
            if need_request_block {
                let request = Message::RequestBlock(hash).to_string();
                socket.write_all(request.as_bytes()).await.unwrap();
            }
        }
        Message::NewTransaction(transaction) => {
            match dag.lock() {
                Ok(mut dag_guard) => {
                    dag_guard.add_transaction(transaction);
                    println!("New transaction added");
                }
                Err(poisoned) => {
                    let mut dag_guard = poisoned.into_inner();
                    dag_guard.add_transaction(transaction);
                    println!("New transaction added");
                }
            }
        }
        Message::Unknown => {
            println!("Received unknown message");
        }
    }

    let mut peers_guard = peers.lock().expect("Mutex lock poisoned");
    peers_guard.insert(addr);
}

pub async fn connect_to_server(address: &str, dag: Arc<Mutex<BlockDAG>>, peers: Arc<Mutex<HashSet<String>>>) {
    match TcpStream::connect(address).await {
        Ok(mut socket) => {
            let request = Message::RequestTip.to_string();
            socket.write_all(request.as_bytes()).await.unwrap();

            let mut buffer = [0; 1024];
            match socket.read(&mut buffer).await {
                Ok(size) if size > 0 => {
                    let response = String::from_utf8_lossy(&buffer[..size]);
                    let msg = Message::from_str(&response);
                    handle_message(dag, peers, msg, socket, address.to_string()).await;
                }
                _ => (),
            }
        }
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }
}
