//! Asynchronous Multi-Client OpenIGTLink Server
//!
//! Demonstrates a production-ready async server using Tokio that can handle
//! multiple concurrent client connections efficiently.
//!
//! # Usage
//!
//! ```bash
//! # Start the async server
//! cargo run --example async_server
//!
//! # In separate terminals, connect multiple clients:
//! cargo run --example client
//! ```
//!
//! # Features
//!
//! - Tokio async runtime for high concurrency
//! - Separate task per client connection
//! - Graceful client disconnection handling
//! - Real-time connection statistics
//! - No blocking operations

use openigtlink_rust::error::Result;
use openigtlink_rust::protocol::header::Header;
use openigtlink_rust::protocol::message::IgtlMessage;
use openigtlink_rust::protocol::types::{StatusMessage, TransformMessage};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::{interval, Duration};

static CLIENT_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[tokio::main]
async fn main() {
    if let Err(e) = run_server().await {
        eprintln!("[ERROR] Server failed: {}", e);
        std::process::exit(1);
    }
}

async fn run_server() -> Result<()> {
    let addr = "127.0.0.1:18944";
    let listener = TcpListener::bind(addr).await?;

    println!("=== Async OpenIGTLink Server ===\n");
    println!("[INFO] Configuration:");
    println!("  Address: {}", addr);
    println!("  Runtime: Tokio (async/await)");
    println!("  Concurrency: Unlimited clients");
    println!("  I/O Mode: Non-blocking\n");

    let active_clients = Arc::new(AtomicUsize::new(0));

    // Spawn statistics reporter
    let stats_clients = active_clients.clone();
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(5));
        loop {
            ticker.tick().await;
            let count = stats_clients.load(Ordering::Relaxed);
            if count > 0 {
                println!("[STATS] Active clients: {}", count);
            }
        }
    });

    println!("[INFO] Server ready, waiting for connections...\n");

    loop {
        let (socket, addr) = listener.accept().await?;
        let client_id = CLIENT_COUNTER.fetch_add(1, Ordering::Relaxed) + 1;
        let clients = active_clients.clone();

        println!("[CONNECT] Client #{} connected from {}", client_id, addr);
        clients.fetch_add(1, Ordering::Relaxed);

        // Spawn a new task for each client
        tokio::spawn(async move {
            let result = handle_client(socket, client_id).await;
            clients.fetch_sub(1, Ordering::Relaxed);

            match result {
                Ok(_) => println!("[DISCONNECT] Client #{} disconnected gracefully", client_id),
                Err(e) => eprintln!("[ERROR] Client #{} error: {}", client_id, e),
            }
        });
    }
}

/// Handle a single client connection asynchronously
async fn handle_client(mut socket: TcpStream, client_id: usize) -> Result<()> {
    println!("  [#{}] Handler started", client_id);

    let mut message_count = 0;

    loop {
        // Read header (58 bytes)
        let mut header_buf = vec![0u8; Header::SIZE];

        match socket.read_exact(&mut header_buf).await {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                // Client closed connection
                break;
            }
            Err(e) => return Err(e.into()),
        }

        // Decode header
        let header = Header::decode(&header_buf)?;

        // Read body
        let mut body_buf = vec![0u8; header.body_size as usize];
        socket.read_exact(&mut body_buf).await?;

        // Reconstruct full message
        let mut full_msg = header_buf;
        full_msg.extend_from_slice(&body_buf);

        message_count += 1;

        // Process message based on type
        match header.type_name.as_str()? {
            "TRANSFORM" => {
                if let Ok(msg) = IgtlMessage::<TransformMessage>::decode(&full_msg) {
                    println!(
                        "  [#{}] TRANSFORM received (device: {})",
                        client_id,
                        msg.header.device_name.as_str().unwrap_or("unknown")
                    );

                    // Send acknowledgment
                    send_ack(&mut socket, client_id, message_count).await?;
                }
            }
            "STATUS" => {
                if let Ok(msg) = IgtlMessage::<StatusMessage>::decode(&full_msg) {
                    println!(
                        "  [#{}] STATUS received: {}",
                        client_id, msg.content.status_string
                    );
                }
            }
            msg_type => {
                println!("  [#{}] {} message received", client_id, msg_type);
            }
        }
    }

    println!(
        "  [#{}] Processed {} messages total",
        client_id, message_count
    );

    Ok(())
}

/// Send acknowledgment message back to client
async fn send_ack(socket: &mut TcpStream, client_id: usize, msg_num: usize) -> Result<()> {
    let status = StatusMessage::ok(&format!(
        "Message #{} processed by async handler #{}",
        msg_num, client_id
    ));

    let response = IgtlMessage::new(status, "AsyncServer")?;
    let data = response.encode()?;

    socket.write_all(&data).await?;
    socket.flush().await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncWriteExt;
    use tokio::net::TcpStream;

    #[tokio::test]
    async fn test_server_bind() {
        // Test that we can bind to a port
        let listener = TcpListener::bind("127.0.0.1:0").await;
        assert!(listener.is_ok());
    }

    #[tokio::test]
    async fn test_concurrent_clients() {
        // Start test server
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        // Spawn server task
        tokio::spawn(async move {
            while let Ok((socket, _)) = listener.accept().await {
                tokio::spawn(async move {
                    let _ = handle_client(socket, 999).await;
                });
            }
        });

        // Give server time to start
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Connect multiple clients concurrently
        let mut handles = vec![];

        for i in 0..5 {
            let handle = tokio::spawn(async move {
                let result = TcpStream::connect(addr).await;
                assert!(result.is_ok(), "Client {} failed to connect", i);
                result.unwrap()
            });
            handles.push(handle);
        }

        // Wait for all connections
        let mut clients = vec![];
        for handle in handles {
            let client = handle.await.unwrap();
            clients.push(client);
        }

        assert_eq!(clients.len(), 5);

        // Close all clients
        for mut client in clients {
            let _ = client.shutdown().await;
        }
    }

    #[tokio::test]
    async fn test_message_handling() {
        // Create a test server
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        // Spawn server
        tokio::spawn(async move {
            let (socket, _) = listener.accept().await.unwrap();
            let _ = handle_client(socket, 1).await;
        });

        tokio::time::sleep(Duration::from_millis(10)).await;

        // Connect client
        let mut client = TcpStream::connect(addr).await.unwrap();

        // Send a TRANSFORM message
        let transform = TransformMessage::identity();
        let msg = IgtlMessage::new(transform, "TestDevice").unwrap();
        let data = msg.encode().unwrap();

        client.write_all(&data).await.unwrap();
        client.flush().await.unwrap();

        // Give server time to process
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Read acknowledgment
        let mut header_buf = vec![0u8; Header::SIZE];
        let read_result = client.read_exact(&mut header_buf).await;
        assert!(read_result.is_ok());
    }

    #[test]
    fn test_client_counter() {
        let count1 = CLIENT_COUNTER.fetch_add(1, Ordering::Relaxed);
        let count2 = CLIENT_COUNTER.fetch_add(1, Ordering::Relaxed);
        assert!(count2 > count1);
    }
}
