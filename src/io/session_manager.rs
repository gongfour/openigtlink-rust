//! Multi-client session management for OpenIGTLink servers
//!
//! Provides a high-level abstraction for managing multiple concurrent client
//! connections with message routing, broadcasting, and handler registration.
//!
//! # Features
//!
//! - Concurrent client session management
//! - Message broadcasting to all/selected clients
//! - Per-client message handlers
//! - Automatic disconnection handling
//! - Thread-safe client registry
//!
//! # Example
//!
//! ```no_run
//! use openigtlink_rust::io::SessionManager;
//! use openigtlink_rust::protocol::types::StatusMessage;
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let manager = Arc::new(SessionManager::new("127.0.0.1:18944").await?);
//!
//!     // Spawn client acceptor
//!     let mgr = manager.clone();
//!     tokio::spawn(async move {
//!         mgr.accept_clients().await;
//!     });
//!
//!     // Broadcast status to all clients
//!     let status = StatusMessage::ok("Server ready");
//!     manager.broadcast(&status).await?;
//!
//!     Ok(())
//! }
//! ```

use crate::error::{IgtlError, Result};
use crate::protocol::header::Header;
use crate::protocol::message::{IgtlMessage, Message};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, RwLock};

/// Unique identifier for each client session
pub type ClientId = u64;

/// Client session state
#[derive(Debug)]
struct ClientSession {
    /// Client ID
    id: ClientId,
    /// Client socket address
    addr: SocketAddr,
    /// Channel to send messages to this client
    tx: mpsc::UnboundedSender<Vec<u8>>,
    /// Connection start time
    connected_at: std::time::Instant,
}

impl ClientSession {
    /// Send a raw message to this client
    async fn send_raw(&self, data: Vec<u8>) -> Result<()> {
        self.tx.send(data).map_err(|_| {
            IgtlError::Io(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "Client disconnected",
            ))
        })?;
        Ok(())
    }

    /// Get connection duration
    fn uptime(&self) -> std::time::Duration {
        self.connected_at.elapsed()
    }
}

/// Multi-client session manager
///
/// Manages multiple concurrent OpenIGTLink client connections with automatic
/// message routing and broadcasting capabilities.
pub struct SessionManager {
    /// TCP listener for accepting new clients
    listener: TcpListener,
    /// Active client sessions (ClientId -> ClientSession)
    clients: Arc<RwLock<HashMap<ClientId, ClientSession>>>,
    /// Client ID counter
    next_client_id: AtomicU64,
    /// Message handlers (optional)
    handlers: Arc<RwLock<Vec<Box<dyn MessageHandler>>>>,
}

/// Trait for handling incoming messages
///
/// Implement this trait to process messages from clients.
pub trait MessageHandler: Send + Sync {
    /// Handle a message from a specific client
    ///
    /// # Arguments
    /// * `client_id` - ID of the client that sent the message
    /// * `type_name` - Message type name (e.g., "TRANSFORM")
    /// * `data` - Raw message data (header + body)
    fn handle_message(&self, client_id: ClientId, type_name: &str, data: &[u8]);
}

impl SessionManager {
    /// Create a new session manager bound to the specified address
    ///
    /// # Arguments
    /// * `addr` - Address to bind (e.g., "127.0.0.1:18944")
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use openigtlink_rust::io::SessionManager;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let manager = SessionManager::new("0.0.0.0:18944").await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn new(addr: &str) -> Result<Self> {
        let listener = TcpListener::bind(addr).await?;
        Ok(SessionManager {
            listener,
            clients: Arc::new(RwLock::new(HashMap::new())),
            next_client_id: AtomicU64::new(1),
            handlers: Arc::new(RwLock::new(Vec::new())),
        })
    }

    /// Get the local address this manager is bound to
    pub fn local_addr(&self) -> Result<SocketAddr> {
        Ok(self.listener.local_addr()?)
    }

    /// Get the number of active client connections
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use openigtlink_rust::io::SessionManager;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let manager = SessionManager::new("127.0.0.1:18944").await?;
    /// println!("Active clients: {}", manager.client_count().await);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn client_count(&self) -> usize {
        self.clients.read().await.len()
    }

    /// Get a list of all active client IDs
    pub async fn client_ids(&self) -> Vec<ClientId> {
        self.clients.read().await.keys().copied().collect()
    }

    /// Get information about a specific client
    pub async fn client_info(&self, client_id: ClientId) -> Option<ClientInfo> {
        let clients = self.clients.read().await;
        clients.get(&client_id).map(|session| ClientInfo {
            id: session.id,
            addr: session.addr,
            uptime: session.uptime(),
        })
    }

    /// Register a message handler
    ///
    /// Handlers are called in the order they were registered.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use openigtlink_rust::io::{SessionManager, MessageHandler, ClientId};
    ///
    /// struct MyHandler;
    ///
    /// impl MessageHandler for MyHandler {
    ///     fn handle_message(&self, client_id: ClientId, type_name: &str, data: &[u8]) {
    ///         println!("Client {} sent {}", client_id, type_name);
    ///     }
    /// }
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut manager = SessionManager::new("127.0.0.1:18944").await?;
    /// manager.add_handler(Box::new(MyHandler)).await;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn add_handler(&self, handler: Box<dyn MessageHandler>) {
        self.handlers.write().await.push(handler);
    }

    /// Accept new client connections in a loop
    ///
    /// This method runs forever, accepting new clients and spawning handler tasks.
    /// It should be run in a separate task.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use openigtlink_rust::io::SessionManager;
    /// use std::sync::Arc;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let manager = Arc::new(SessionManager::new("127.0.0.1:18944").await?);
    ///
    ///     // Spawn acceptor in background
    ///     let mgr = manager.clone();
    ///     tokio::spawn(async move {
    ///         mgr.accept_clients().await;
    ///     });
    ///
    ///     // Do other work...
    ///     Ok(())
    /// }
    /// ```
    pub async fn accept_clients(&self) {
        loop {
            match self.listener.accept().await {
                Ok((socket, addr)) => {
                    let client_id = self.next_client_id.fetch_add(1, Ordering::SeqCst);
                    println!("[SessionManager] Client #{} connected from {}", client_id, addr);

                    if let Err(e) = self.handle_client(client_id, socket, addr).await {
                        eprintln!("[SessionManager] Failed to setup client #{}: {}", client_id, e);
                    }
                }
                Err(e) => {
                    eprintln!("[SessionManager] Accept error: {}", e);
                }
            }
        }
    }

    /// Handle a single client connection
    async fn handle_client(
        &self,
        client_id: ClientId,
        socket: TcpStream,
        addr: SocketAddr,
    ) -> Result<()> {
        let (tx, mut rx) = mpsc::unbounded_channel::<Vec<u8>>();

        // Register client session
        {
            let session = ClientSession {
                id: client_id,
                addr,
                tx,
                connected_at: std::time::Instant::now(),
            };
            self.clients.write().await.insert(client_id, session);
        }

        // Split socket into read/write halves (consuming ownership)
        let (mut reader, mut writer) = socket.into_split();

        // Spawn sender task (sends messages to client)
        let sender_task = tokio::spawn(async move {
            while let Some(data) = rx.recv().await {
                if writer.write_all(&data).await.is_err() {
                    break;
                }
                if writer.flush().await.is_err() {
                    break;
                }
            }
        });

        // Receiver task (reads messages from client)
        let handlers = self.handlers.clone();

        let receiver_task = tokio::spawn(async move {
            loop {
                // Read header
                let mut header_buf = vec![0u8; Header::SIZE];
                if reader.read_exact(&mut header_buf).await.is_err() {
                    break;
                }

                let header = match Header::decode(&header_buf) {
                    Ok(h) => h,
                    Err(_) => break,
                };

                // Read body
                let mut body_buf = vec![0u8; header.body_size as usize];
                if reader.read_exact(&mut body_buf).await.is_err() {
                    break;
                }

                // Reconstruct full message
                let mut full_msg = header_buf.clone();
                full_msg.extend_from_slice(&body_buf);

                // Call message handlers
                let type_name = header.type_name.as_str().unwrap_or("UNKNOWN");
                let handlers_guard = handlers.read().await;
                for handler in handlers_guard.iter() {
                    handler.handle_message(client_id, type_name, &full_msg);
                }
            }
        });

        // Wait for either task to finish (indicates disconnection)
        tokio::select! {
            _ = sender_task => {},
            _ = receiver_task => {},
        }

        // Cleanup: remove client from registry
        self.clients.write().await.remove(&client_id);
        println!("[SessionManager] Client #{} disconnected", client_id);

        Ok(())
    }

    /// Broadcast a message to all connected clients
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use openigtlink_rust::io::SessionManager;
    /// use openigtlink_rust::protocol::types::StatusMessage;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let manager = SessionManager::new("127.0.0.1:18944").await?;
    /// let status = StatusMessage::ok("System ready");
    /// manager.broadcast(&status).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn broadcast<T: Message + Clone>(&self, message: &T) -> Result<()> {
        let igtl_msg = IgtlMessage::new(message.clone(), "Server")?;
        let data = igtl_msg.encode()?;

        let clients_guard = self.clients.read().await;
        for session in clients_guard.values() {
            let _ = session.send_raw(data.clone()).await;
        }

        Ok(())
    }

    /// Send a message to a specific client
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use openigtlink_rust::io::SessionManager;
    /// use openigtlink_rust::protocol::types::StatusMessage;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let manager = SessionManager::new("127.0.0.1:18944").await?;
    /// let status = StatusMessage::ok("Personal message");
    /// manager.send_to(42, &status).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn send_to<T: Message + Clone>(&self, client_id: ClientId, message: &T) -> Result<()> {
        let igtl_msg = IgtlMessage::new(message.clone(), "Server")?;
        let data = igtl_msg.encode()?;

        let clients_guard = self.clients.read().await;
        if let Some(session) = clients_guard.get(&client_id) {
            session.send_raw(data).await?;
            Ok(())
        } else {
            Err(IgtlError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Client {} not found", client_id),
            )))
        }
    }

    /// Disconnect a specific client
    pub async fn disconnect(&self, client_id: ClientId) -> Result<()> {
        let mut clients = self.clients.write().await;
        if clients.remove(&client_id).is_some() {
            println!("[SessionManager] Forcibly disconnected client #{}", client_id);
            Ok(())
        } else {
            Err(IgtlError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Client {} not found", client_id),
            )))
        }
    }

    /// Disconnect all clients and shut down
    pub async fn shutdown(&self) {
        let mut clients = self.clients.write().await;
        let count = clients.len();
        clients.clear();
        println!("[SessionManager] Shutdown: disconnected {} clients", count);
    }
}

/// Client information snapshot
#[derive(Debug, Clone)]
pub struct ClientInfo {
    pub id: ClientId,
    pub addr: SocketAddr,
    pub uptime: std::time::Duration,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::types::StatusMessage;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_session_manager_create() {
        let manager = SessionManager::new("127.0.0.1:0").await;
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_client_count() {
        let manager = SessionManager::new("127.0.0.1:0").await.unwrap();
        assert_eq!(manager.client_count().await, 0);
    }

    #[tokio::test]
    async fn test_broadcast_no_clients() {
        let manager = SessionManager::new("127.0.0.1:0").await.unwrap();
        let status = StatusMessage::ok("test");
        let result = manager.broadcast(&status).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_send_to_nonexistent_client() {
        let manager = SessionManager::new("127.0.0.1:0").await.unwrap();
        let status = StatusMessage::ok("test");
        let result = manager.send_to(999, &status).await;
        assert!(result.is_err());
    }
}
