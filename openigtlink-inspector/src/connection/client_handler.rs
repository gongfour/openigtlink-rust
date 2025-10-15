use openigtlink_rust::io::ClientBuilder;
use std::sync::mpsc::{Receiver, Sender};
use std::time::SystemTime;

use crate::types::{ConnectionCommand, ReceivedMessage};

/// Client 연결을 관리하는 비동기 함수
///
/// 백그라운드 tokio task에서 실행되며, OpenIGTLink 서버에 연결하고
/// 메시지를 수신하여 UI 스레드로 전달합니다.
///
/// # Arguments
/// * `host` - 서버 호스트명 (예: "127.0.0.1")
/// * `port` - 서버 포트 (예: 18944)
/// * `message_tx` - 수신된 메시지를 UI로 전송하는 채널
/// * `command_rx` - UI로부터 명령을 받는 채널
pub async fn run_client_connection(
    host: String,
    port: u16,
    message_tx: Sender<ReceivedMessage>,
    command_rx: Receiver<ConnectionCommand>,
) {
    // ClientBuilder로 OpenIGTLink 서버에 연결
    let addr = format!("{}:{}", host, port);
    let mut client = match ClientBuilder::new().tcp(&addr).async_mode().build().await {
        Ok(c) => c,
        Err(_e) => {
            // 연결 실패 시 종료
            // TODO: 에러 메시지를 UI로 전달
            return;
        }
    };

    // 메시지 수신 루프
    loop {
        // UI로부터 명령 확인 (논블로킹)
        if let Ok(cmd) = command_rx.try_recv() {
            match cmd {
                ConnectionCommand::Disconnect => {
                    // Disconnect 명령 시 루프 종료
                    break;
                }
                ConnectionCommand::SendMessage { message: _ } => {
                    // TODO: Phase 2에서 메시지 송신 구현
                }
                _ => {}
            }
        }

        // OpenIGTLink 메시지 수신
        match client.receive_any().await {
            Ok(msg) => {
                // 수신된 메시지를 ReceivedMessage로 래핑
                let received = ReceivedMessage {
                    timestamp: SystemTime::now(),
                    message: msg,
                    size_bytes: 0, // TODO: 실제 메시지 크기 계산
                    from_client: None,
                };

                // UI 스레드로 메시지 전송
                if message_tx.send(received).is_err() {
                    // UI 스레드가 종료된 경우 루프 종료
                    break;
                }
            }
            Err(_e) => {
                // 연결 끊김 또는 수신 에러 시 루프 종료
                // TODO: 에러 처리 개선
                break;
            }
        }
    }
}
