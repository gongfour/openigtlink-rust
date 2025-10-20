use openigtlink_rust::protocol::AnyMessage;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TabType {
    Client,
    Server,
}

/// 수신된 메시지를 저장하는 구조체
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceivedMessage {
    pub timestamp: u64, // milliseconds since UNIX_EPOCH
    pub message_type: String,
    pub device_name: String,
    pub size_bytes: usize,
    pub from_client: Option<String>,
    pub body: serde_json::Value, // JSON 형식의 메시지 바디
}

/// 연결 관리를 위한 명령 enum
#[derive(Debug)]
#[allow(dead_code)]
pub enum ConnectionCommand {
    Connect { host: String, port: u16 },
    Disconnect,
    SendMessage { message: Box<AnyMessage> },
    Listen { port: u16 },
    StopListening,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Tab {
    pub id: usize,
    pub name: String,
    pub tab_type: TabType,
    pub host: String, // Client only
    pub port: String,
    pub is_connected: bool,
    pub send_panel_expanded: bool,
    pub rx_count: usize,
    pub tx_count: usize,
    pub error_message: Option<String>,
}

impl Tab {
    pub fn new_client(id: usize) -> Self {
        Self {
            id,
            name: format!("Client-{}", id),
            tab_type: TabType::Client,
            host: "127.0.0.1".to_string(),
            port: "18944".to_string(),
            is_connected: false,
            send_panel_expanded: false,
            rx_count: 0,
            tx_count: 0,
            error_message: None,
        }
    }

    pub fn new_server(id: usize) -> Self {
        Self {
            id,
            name: format!("Server-{}", id),
            tab_type: TabType::Server,
            host: String::new(),
            port: "18944".to_string(),
            is_connected: false,
            send_panel_expanded: false,
            rx_count: 0,
            tx_count: 0,
            error_message: None,
        }
    }
}
