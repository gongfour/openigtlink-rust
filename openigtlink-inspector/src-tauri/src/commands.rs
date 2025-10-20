use crate::connection::ConnectionManager;
use crate::types::Tab;
use openigtlink_rust::protocol::AnyMessage;
use serde_json::json;
use std::sync::Mutex;
use tauri::{Manager, State};

/// Client 연결 명령
#[tauri::command]
pub async fn connect_client(
    tab_id: usize,
    host: String,
    port: u16,
    connection: State<'_, Mutex<ConnectionManager>>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    // Mutex를 await 전에 해제하기 위해 별도 스코프 사용
    let result = {
        let mut conn = connection.lock().map_err(|e| e.to_string())?;
        // 연결 파라미터를 미리 설정
        conn.host = host.clone();
        conn.port = port;
        Ok::<_, String>((host, port))
    }?;

    // Mutex 해제 후 실제 연결 수행
    let addr = format!("{}:{}", result.0, result.1);

    use openigtlink_rust::io::ClientBuilder;
    use crate::types::ReceivedMessage;

    let mut client = ClientBuilder::new()
        .tcp(&addr)
        .async_mode()
        .build()
        .await
        .map_err(|e| format!("Connection failed: {}", e))?;

    // 연결 성공 후 상태 업데이트
    {
        let mut conn = connection.lock().map_err(|e| e.to_string())?;
        conn.is_connected = true;
        conn.rx_count = 0;
        conn.tx_count = 0;
    }

    // 백그라운드에서 메시지 수신
    tokio::spawn(async move {
        loop {
            match client.receive_any().await {
                Ok(msg) => {
                    let msg_type = msg.message_type().to_string();
                    let device_name = msg.device_name().unwrap_or("unknown").to_string();

                    // 메시지 본문 파싱
                    let body = parse_message_body(&msg);

                    let timestamp = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_millis() as u64)
                        .unwrap_or(0);

                    // 메시지 크기 계산: body 콘텐츠 크기
                    let size_bytes = calculate_message_size(&msg);

                    let received = ReceivedMessage {
                        timestamp,
                        message_type: msg_type,
                        device_name,
                        size_bytes,
                        from_client: None,
                        body,
                    };

                    use crate::types::MessageWithTabId;
                    let message_with_tab = MessageWithTabId {
                        tab_id,
                        message: received,
                    };

                    let _ = app.emit_all("message_received", message_with_tab);
                }
                Err(_e) => {
                    let _ = app.emit_all("connection_closed", serde_json::json!({"tab_id": tab_id}));
                    break;
                }
            }
        }
    });

    Ok(())
}

/// Server 리스닝 시작
#[tauri::command]
pub async fn listen_server(
    tab_id: usize,
    port: u16,
    connection: State<'_, Mutex<ConnectionManager>>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    use openigtlink_rust::io::AsyncIgtlServer;
    use crate::types::ReceivedMessage;

    // Server 생성
    let addr = format!("0.0.0.0:{}", port);
    let server = AsyncIgtlServer::bind(&addr)
        .await
        .map_err(|e| format!("Failed to start server: {}", e))?;

    // 연결 성공 후 상태 업데이트
    {
        let mut conn = connection.lock().map_err(|e| e.to_string())?;
        conn.is_connected = true;
        conn.port = port;
        conn.rx_count = 0;
        conn.tx_count = 0;
    }

    // 백그라운드에서 클라이언트 연결 및 메시지 수신
    tokio::spawn(async move {
        let mut client_counter = 0;
        loop {
            // 클라이언트 연결 대기
            match server.accept().await {
                Ok(client_conn) => {
                    client_counter += 1;
                    // 클라이언트 주소는 간단한 카운터로 표시
                    let client_addr = format!("Client-{}", client_counter);
                    let app_clone = app.clone();
                    let mut client_conn = client_conn;

                    // 각 클라이언트마다 별도 태스크로 메시지 수신
                    tokio::spawn(async move {
                        loop {
                            match client_conn.receive_any().await {
                                Ok(msg) => {
                                    let msg_type = msg.message_type().to_string();
                                    let device_name = msg.device_name().unwrap_or("unknown").to_string();
                                    let body = parse_message_body(&msg);

                                    let timestamp = std::time::SystemTime::now()
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .map(|d| d.as_millis() as u64)
                                        .unwrap_or(0);

                                    let size_bytes = calculate_message_size(&msg);

                                    let received = ReceivedMessage {
                                        timestamp,
                                        message_type: msg_type,
                                        device_name,
                                        size_bytes,
                                        from_client: Some(client_addr.clone()),
                                        body,
                                    };

                                    use crate::types::MessageWithTabId;
                                    let message_with_tab = MessageWithTabId {
                                        tab_id,
                                        message: received,
                                    };

                                    let _ = app_clone.emit_all("message_received", message_with_tab);
                                }
                                Err(_) => {
                                    // 클라이언트 연결 종료
                                    break;
                                }
                            }
                        }
                    });
                }
                Err(_) => {
                    // Server accept 실패
                    let _ = app.emit_all("connection_closed", serde_json::json!({"tab_id": tab_id}));
                    break;
                }
            }
        }
    });

    Ok(())
}

/// 연결 종료 명령
#[tauri::command]
pub fn disconnect_client(connection: State<'_, Mutex<ConnectionManager>>) -> Result<(), String> {
    let mut conn = connection.lock().map_err(|e| e.to_string())?;
    conn.disconnect();
    Ok(())
}

/// 연결 상태 조회
#[tauri::command]
pub fn get_connection_status(
    connection: State<'_, Mutex<ConnectionManager>>,
) -> Result<serde_json::Value, String> {
    let conn = connection.lock().map_err(|e| e.to_string())?;

    Ok(serde_json::json!({
        "is_connected": conn.is_connected,
        "host": conn.host,
        "port": conn.port,
        "rx_count": conn.rx_count,
        "tx_count": conn.tx_count,
    }))
}

/// 새로운 탭 생성 (Client)
#[tauri::command]
pub fn create_client_tab(id: usize) -> Tab {
    Tab::new_client(id)
}

/// 새로운 탭 생성 (Server)
#[tauri::command]
pub fn create_server_tab(id: usize) -> Tab {
    Tab::new_server(id)
}

/// 메시지 전송 명령
#[tauri::command]
pub async fn send_message(
    _tab_id: usize,
    message_type: String,
    device_name: String,
    content: serde_json::Value,
    _connection: State<'_, Mutex<ConnectionManager>>,
) -> Result<(), String> {
    // 현재는 ConnectionManager에 저장된 클라이언트로 전송
    // 나중에 탭별 연결 관리로 확장 필요

    let _msg = create_message_from_type(&message_type, &device_name, &content)?;

    // 실제 전송 로직은 여기에 구현됨
    // 현재는 메시지 검증만 수행
    println!("Send message: type={}, device={}, content={:?}", message_type, device_name, content);

    Ok(())
}

/// 메시지 타입에 따라 OpenIGTLink 메시지 검증
fn create_message_from_type(
    msg_type: &str,
    device_name: &str,
    _content: &serde_json::Value,
) -> Result<AnyMessage, String> {
    // 유효한 메시지 타입 검증
    match msg_type {
        "TRANSFORM" | "STATUS" | "STRING" | "POSITION" | "IMAGE" | "SENSOR" => {
            // 메시지 타입이 유효함
            // 실제 메시지 생성은 나중에 구현
            println!("Validated message type: {} for device: {}", msg_type, device_name);
            Err("Message sending not fully implemented yet".to_string())
        }
        _ => Err(format!("Unsupported message type: {}", msg_type)),
    }
}

/// 메시지 실제 바이트 크기 계산
fn calculate_message_size(msg: &AnyMessage) -> usize {
    match msg {
        AnyMessage::Transform(_) => {
            // TRANSFORM: Header (58) + Body (48)
            58 + 48
        }
        AnyMessage::Status(status_msg) => {
            let status = &status_msg.content;
            // STATUS: Header + (code(1) + subcode(1) + error_name(20) + status_string(length))
            let status_string_len = status.status_string.len();
            58 + 1 + 1 + 20 + status_string_len
        }
        AnyMessage::String(string_msg) => {
            let string = &string_msg.content.string;
            // STRING: Header + string content
            58 + string.len()
        }
        AnyMessage::Position(pos_msg) => {
            // POSITION: Header (58) + Body (12 floats = 48 bytes)
            let _pos = &pos_msg.content;
            58 + 48
        }
        AnyMessage::Image(image_msg) => {
            let image = &image_msg.content;
            // IMAGE: Header (58) + Header (60) + pixel data
            58 + 60 + image.data.len()
        }
        AnyMessage::Sensor(sensor_msg) => {
            let sensor = &sensor_msg.content;
            // SENSOR: Header (58) + Body (unit(8) + sensor data)
            58 + 8 + (sensor.data.len() * 8) // each f64 is 8 bytes
        }
        AnyMessage::Capability(cap_msg) => {
            let cap = &cap_msg.content;
            // CAPABILITY: Header (58) + count(1) + type_names
            let mut size = 58 + 1;
            for type_name in &cap.types {
                size += 4 + type_name.len(); // 4-byte length + name
            }
            size
        }
        _ => {
            // 알 수 없는 타입은 최소 헤더 크기로 반환
            58
        }
    }
}

/// OpenIGTLink 메시지를 파싱하여 JSON으로 변환
fn parse_message_body(msg: &AnyMessage) -> serde_json::Value {
    match msg {
        AnyMessage::Transform(transform_msg) => {
            let matrix = &transform_msg.content.matrix;
            json!({
                "type": "TRANSFORM",
                "description": "Transformation matrix",
                "matrix": {
                    "rows": 4,
                    "cols": 4,
                    "data": vec![
                        vec![matrix[0][0], matrix[0][1], matrix[0][2], matrix[0][3]],
                        vec![matrix[1][0], matrix[1][1], matrix[1][2], matrix[1][3]],
                        vec![matrix[2][0], matrix[2][1], matrix[2][2], matrix[2][3]],
                        vec![matrix[3][0], matrix[3][1], matrix[3][2], matrix[3][3]],
                    ]
                }
            })
        }
        AnyMessage::Status(status_msg) => {
            let status = &status_msg.content;
            json!({
                "type": "STATUS",
                "description": "Status message",
                "code": status.code,
                "subcode": status.subcode,
                "status_string": status.status_string.clone(),
                "error_name": status.error_name.clone()
            })
        }
        AnyMessage::String(string_msg) => {
            let string_data = &string_msg.content.string;
            json!({
                "type": "STRING",
                "description": "String message",
                "string": string_data.clone()
            })
        }
        AnyMessage::Position(pos_msg) => {
            let pos = &pos_msg.content.position;
            json!({
                "type": "POSITION",
                "description": "Position data",
                "position": {
                    "x": pos[0],
                    "y": pos[1],
                    "z": pos[2]
                }
            })
        }
        AnyMessage::Image(image_msg) => {
            let image = &image_msg.content;
            json!({
                "type": "IMAGE",
                "description": "Image data",
                "num_components": image.num_components,
                "scalar_type": format!("{:?}", image.scalar_type),
                "size": {
                    "x": image.size[0],
                    "y": image.size[1],
                    "z": image.size[2]
                },
                "data_size_bytes": image.data.len()
            })
        }
        AnyMessage::Sensor(sensor_msg) => {
            let sensor = &sensor_msg.content;
            json!({
                "type": "SENSOR",
                "description": "Sensor data",
                "unit": sensor.unit,
                "data_length": sensor.data.len(),
                "sample_data": sensor.data.iter().take(10).map(|v| format!("{:.6}", v)).collect::<Vec<_>>()
            })
        }
        AnyMessage::Capability(cap_msg) => {
            let cap = &cap_msg.content;
            json!({
                "type": "CAPABILITY",
                "description": "Capability message",
                "types": cap.types.clone()
            })
        }
        _ => {
            json!({
                "type": msg.message_type().to_string(),
                "description": "Message data",
                "note": "This message type is not fully parsed yet"
            })
        }
    }
}
