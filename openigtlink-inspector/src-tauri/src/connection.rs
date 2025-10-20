use openigtlink_rust::io::ClientBuilder;
use openigtlink_rust::protocol::AnyMessage;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use tauri::Manager;

use crate::types::ReceivedMessage;

pub struct ConnectionManager {
    pub is_connected: bool,
    pub host: String,
    pub port: u16,
    pub rx_count: usize,
    pub tx_count: usize,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            is_connected: false,
            host: String::new(),
            port: 18944,
            rx_count: 0,
            tx_count: 0,
        }
    }

    pub async fn connect_client(
        &mut self,
        host: String,
        port: u16,
        app: tauri::AppHandle,
    ) -> Result<(), String> {
        let addr = format!("{}:{}", host, port);

        let mut client = ClientBuilder::new()
            .tcp(&addr)
            .async_mode()
            .build()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        self.is_connected = true;
        self.host = host;
        self.port = port;
        self.rx_count = 0;
        self.tx_count = 0;

        // 백그라운드에서 메시지 수신
        tokio::spawn(async move {
            loop {
                match client.receive_any().await {
                    Ok(msg) => {
                        let msg_type = msg.message_type().to_string();
                        let device_name = msg.device_name().unwrap_or("unknown").to_string();

                        let received = ReceivedMessage {
                            timestamp: SystemTime::now(),
                            message_type: msg_type,
                            device_name,
                            size_bytes: 0, // TODO: 실제 크기 계산
                            from_client: None,
                            body: format!("{:?}", msg), // TODO: 실제 JSON 파싱
                        };

                        let _ = app.emit_all("message_received", received);
                    }
                    Err(_e) => {
                        // 연결 끊김
                        let _ = app.emit_all("connection_closed", ());
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    pub fn disconnect(&mut self) {
        self.is_connected = false;
    }
}
