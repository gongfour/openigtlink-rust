use openigtlink_rust::protocol::AnyMessage;
use std::collections::HashMap;
use tokio::sync::mpsc;

pub enum ConnectionType {
    Client(mpsc::UnboundedSender<AnyMessage>),
    Server(HashMap<usize, mpsc::UnboundedSender<AnyMessage>>), // client_id -> sender
}

pub struct TabConnection {
    pub tab_id: usize,
    pub connection_type: ConnectionType,
}

pub struct ConnectionManager {
    pub connections: HashMap<usize, TabConnection>, // tab_id -> connection
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
        }
    }

    pub fn add_client_connection(
        &mut self,
        tab_id: usize,
        sender: mpsc::UnboundedSender<AnyMessage>,
    ) {
        self.connections.insert(
            tab_id,
            TabConnection {
                tab_id,
                connection_type: ConnectionType::Client(sender),
            },
        );
    }

    pub fn add_server_connection(&mut self, tab_id: usize) {
        self.connections.insert(
            tab_id,
            TabConnection {
                tab_id,
                connection_type: ConnectionType::Server(HashMap::new()),
            },
        );
    }

    pub fn add_server_client(
        &mut self,
        tab_id: usize,
        client_id: usize,
        sender: mpsc::UnboundedSender<AnyMessage>,
    ) {
        if let Some(tab_conn) = self.connections.get_mut(&tab_id) {
            if let ConnectionType::Server(ref mut clients) = tab_conn.connection_type {
                clients.insert(client_id, sender);
            }
        }
    }

    pub fn remove_connection(&mut self, tab_id: usize) {
        self.connections.remove(&tab_id);
    }

    pub fn get_connection(&self, tab_id: usize) -> Option<&TabConnection> {
        self.connections.get(&tab_id)
    }
}
