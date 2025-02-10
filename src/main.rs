use crabble::broker::Broker;

use std::collections::HashMap;
use std::sync::Arc;

use bytes::Bytes;
use tokio::io::{self, AsyncBufReadExt, BufReader};

#[tokio::main]
async fn main() {
    let broker = Arc::new(Broker::new());
    broker
        .create_channel("general".to_string())
        .await
        .expect("Could not create channel 'general'");

    println!("Welcome to crabble - in-memory message broker!");
    println!("Available commands:");
    println!("  subscribe <channel>          - subscribe to channel");
    println!("  unsubscribe <channel>        - unsubscribe from channel");
    println!("  publish <channel> <message>  - send message");
    println!("  exit                         - exit :)");

    let mut subscriptions: HashMap<String, (u64, tokio::task::JoinHandle<()>)> = HashMap::new();
    let broker_main = Arc::clone(&broker);

    let stdin = BufReader::new(io::stdin());
    let mut lines = stdin.lines();

    while let Some(line) = lines.next_line().await.expect("Error line reading") {
        let parts: Vec<&str> = line.trim().splitn(3, ' ').collect();
        if parts.is_empty() {
            continue;
        }
        match parts[0] {
            "subscribe" => {
                if parts.len() < 2 {
                    println!("Using: subscribe <channel>");
                    continue;
                }
                let channel = parts[1];
                let broker_inner = Arc::clone(&broker_main);
                match broker_inner.subscribe(channel).await {
                    Ok((sub_id, mut rx)) => {
                        println!(
                            "Subscribed to channel '{}' with subscribe id {}",
                            channel, sub_id
                        );
                        let channel_name = channel.to_string();
                        let handle = tokio::spawn(async move {
                            while let Some(msg) = rx.recv().await {
                                println!(
                                    "Received from channel '{}': {}",
                                    channel_name,
                                    String::from_utf8_lossy(&msg)
                                );
                            }
                        });
                        subscriptions.insert(channel.to_string(), (sub_id, handle));
                    }
                    Err(e) => println!("Subscribe error: {}", e),
                }
            }
            "unsubscribe" => {
                if parts.len() < 2 {
                    println!("Using: unsubscribe <channel>");
                    continue;
                }
                let channel = parts[1];
                if let Some((sub_id, handle)) = subscriptions.remove(channel) {
                    match broker_main.unsubscribe(channel, sub_id).await {
                        Ok(_) => {
                            println!("Unsubscribed from channel '{}'", channel);
                            handle.abort();
                        }
                        Err(e) => println!("Unsubscribe error: {}", e),
                    }
                } else {
                    println!("Subscription to channel '{}' not found", channel);
                }
            }
            "publish" => {
                if parts.len() < 3 {
                    println!("Using: publish <channel> <message>");
                    continue;
                }
                let channel = parts[1];
                let msg = parts[2];
                let bytes_msg = Bytes::from(msg.to_string());
                match broker_main.publish(channel, bytes_msg).await {
                    Ok(_) => println!("Message sent to channel '{}'", channel),
                    Err(e) => println!("Publish error: {}", e),
                }
            }
            "exit" => {
                println!("Exit...");
                break;
            }
            _ => {
                println!("Unknown command");
            }
        }
    }
}
