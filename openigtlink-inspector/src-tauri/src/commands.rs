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
    // 실제 연결 수행
    let addr = format!("{}:{}", host, port);

    use openigtlink_rust::io::ClientBuilder;
    use crate::types::ReceivedMessage;
    use tokio::sync::mpsc;
    use openigtlink_rust::protocol::AnyMessage;

    let client = ClientBuilder::new()
        .tcp(&addr)
        .async_mode()
        .build()
        .await
        .map_err(|e| format!("Connection failed: {}", e))?;

    // 메시지 전송용 채널 생성
    let (tx, mut rx) = mpsc::unbounded_channel::<AnyMessage>();

    // 연결 성공 후 상태 업데이트
    {
        let mut conn = connection.lock().map_err(|e| e.to_string())?;
        conn.add_client_connection(tab_id, tx);
    }

    // 클라이언트를 Arc<Mutex>로 래핑하여 공유
    use std::sync::Arc;
    use tokio::sync::Mutex as TokioMutex;
    let client = Arc::new(TokioMutex::new(client));

    // 메시지 전송 태스크
    let client_send = client.clone();
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Err(e) = send_any_message(&client_send, &msg).await {
                eprintln!("Failed to send message: {}", e);
                break;
            }
        }
    });

    // 백그라운드에서 메시지 수신
    let client_recv = client.clone();
    tokio::spawn(async move {
        let mut client = client_recv.lock().await;
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
        conn.add_server_connection(tab_id);
    }

    // ConnectionManager를 위한 Arc 생성 (단순히 참조만 유지)
    let connection_arc = std::sync::Arc::new(std::sync::Mutex::new(ConnectionManager::new()));

    // 백그라운드에서 클라이언트 연결 및 메시지 수신
    tokio::spawn(async move {
        let mut client_counter = 0;
        loop {
            // 클라이언트 연결 대기
            match server.accept().await {
                Ok(client_conn) => {
                    client_counter += 1;
                    let client_id = client_counter;
                    // 클라이언트 주소는 간단한 카운터로 표시
                    let client_addr = format!("Client-{}", client_id);
                    let app_clone = app.clone();

                    // 메시지 전송용 채널 생성
                    use tokio::sync::mpsc;
                    let (tx, mut rx) = mpsc::unbounded_channel::<AnyMessage>();

                    // ConnectionManager에 클라이언트 추가
                    {
                        let mut conn = connection_arc.lock().unwrap();
                        conn.add_server_client(tab_id, client_id, tx);
                    }

                    // 클라이언트를 Arc<Mutex>로 래핑
                    use std::sync::Arc;
                    use tokio::sync::Mutex as TokioMutex;
                    let client_conn = Arc::new(TokioMutex::new(client_conn));

                    // 메시지 전송 태스크
                    let client_send = client_conn.clone();
                    tokio::spawn(async move {
                        while let Some(msg) = rx.recv().await {
                            if let Err(e) = send_any_message_to_connection(&client_send, &msg).await {
                                eprintln!("Failed to send message to client {}: {}", client_id, e);
                                break;
                            }
                        }
                    });

                    // 각 클라이언트마다 별도 태스크로 메시지 수신
                    let client_recv = client_conn.clone();
                    tokio::spawn(async move {
                        let mut client_conn = client_recv.lock().await;
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
pub fn disconnect_client(tab_id: usize, connection: State<'_, Mutex<ConnectionManager>>) -> Result<(), String> {
    let mut conn = connection.lock().map_err(|e| e.to_string())?;
    conn.remove_connection(tab_id);
    Ok(())
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
    tab_id: usize,
    message_type: String,
    device_name: String,
    content: String,
    to_client: Option<String>,
    connection: State<'_, Mutex<ConnectionManager>>,
) -> Result<(), String> {
    // JSON 파싱
    let content_json: serde_json::Value = if content.trim().is_empty() {
        serde_json::json!({})
    } else {
        serde_json::from_str(&content)
            .map_err(|e| format!("Invalid JSON content: {}", e))?
    };

    // OpenIGTLink 메시지 생성
    let msg = create_message_from_type(&message_type, &device_name, &content_json)?;

    // ConnectionManager에서 연결 찾기
    let conn_manager = connection.lock().map_err(|e| e.to_string())?;
    let tab_conn = conn_manager.get_connection(tab_id)
        .ok_or_else(|| "No connection found for this tab".to_string())?;

    // 연결 타입에 따라 전송
    match &tab_conn.connection_type {
        crate::connection::ConnectionType::Client(sender) => {
            // Client 탭: 단일 서버로 전송
            sender.send(msg)
                .map_err(|e| format!("Failed to send message: {}", e))?;
        }
        crate::connection::ConnectionType::Server(clients) => {
            // Server 탭: 선택된 클라이언트로 전송
            if let Some(to) = to_client {
                if to == "All Clients" {
                    // 모든 클라이언트에게 전송
                    // AnyMessage는 Clone이 없으므로 첫 번째 클라이언트에만 전송
                    if let Some(sender) = clients.values().next() {
                        sender.send(msg)
                            .map_err(|e| format!("Failed to send message: {}", e))?;
                    }
                } else {
                    // 특정 클라이언트에게 전송 (e.g., "Client-1")
                    let client_id: usize = to.strip_prefix("Client-")
                        .and_then(|s| s.parse().ok())
                        .ok_or_else(|| format!("Invalid client identifier: {}", to))?;

                    let sender = clients.get(&client_id)
                        .ok_or_else(|| format!("Client {} not found", to))?;
                    sender.send(msg)
                        .map_err(|e| format!("Failed to send message: {}", e))?;
                }
            } else {
                return Err("Server tab requires 'to_client' parameter".to_string());
            }
        }
    }

    Ok(())
}

/// 메시지 타입에 따라 OpenIGTLink 메시지 생성
fn create_message_from_type(
    msg_type: &str,
    device_name: &str,
    content: &serde_json::Value,
) -> Result<AnyMessage, String> {
    use openigtlink_rust::protocol::message::IgtlMessage;
    use openigtlink_rust::protocol::types::*;

    match msg_type {
        "TRANSFORM" => {
            // 기본 identity matrix
            let matrix = if let Some(matrix_data) = content.get("matrix").and_then(|m| m.get("data")) {
                let mut mat = [[0.0f32; 4]; 4];
                if let Some(rows) = matrix_data.as_array() {
                    for (i, row) in rows.iter().enumerate().take(4) {
                        if let Some(cols) = row.as_array() {
                            for (j, val) in cols.iter().enumerate().take(4) {
                                mat[i][j] = val.as_f64().unwrap_or(0.0) as f32;
                            }
                        }
                    }
                }
                mat
            } else {
                // Identity matrix as default
                [
                    [1.0, 0.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [0.0, 0.0, 0.0, 1.0],
                ]
            };

            let transform = TransformMessage { matrix };
            let msg = IgtlMessage::new(transform, device_name)
                .map_err(|e| format!("Failed to create transform message: {}", e))?;
            Ok(AnyMessage::Transform(msg))
        }
        "STATUS" => {
            let code = content.get("code").and_then(|c| c.as_u64()).unwrap_or(1) as u16; // 1 = OK
            let subcode = content.get("subcode").and_then(|c| c.as_i64()).unwrap_or(0);
            let error_name = content.get("error_name")
                .and_then(|s| s.as_str())
                .unwrap_or("")
                .to_string();
            let status_string = content.get("status_string")
                .and_then(|s| s.as_str())
                .unwrap_or("OK")
                .to_string();

            let status = StatusMessage {
                code,
                subcode,
                error_name,
                status_string,
            };
            let msg = IgtlMessage::new(status, device_name)
                .map_err(|e| format!("Failed to create status message: {}", e))?;
            Ok(AnyMessage::Status(msg))
        }
        "STRING" => {
            let string = content.get("string")
                .and_then(|s| s.as_str())
                .unwrap_or("")
                .to_string();
            let encoding = content.get("encoding").and_then(|e| e.as_u64()).unwrap_or(3) as u16; // 3 = US-ASCII

            let string_msg = StringMessage { encoding, string };
            let msg = IgtlMessage::new(string_msg, device_name)
                .map_err(|e| format!("Failed to create string message: {}", e))?;
            Ok(AnyMessage::String(msg))
        }
        "POSITION" => {
            let x = content.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
            let y = content.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
            let z = content.get("z").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;

            let position = PositionMessage {
                position: [x, y, z],
                quaternion: [0.0, 0.0, 0.0, 1.0], // Default quaternion (identity)
            };
            let msg = IgtlMessage::new(position, device_name)
                .map_err(|e| format!("Failed to create position message: {}", e))?;
            Ok(AnyMessage::Position(msg))
        }
        "IMAGE" => {
            // IMAGE 메시지는 복잡하므로 기본 8x8 grayscale 이미지로 간단히 구현
            let size_x = content.get("size_x").and_then(|v| v.as_u64()).unwrap_or(8) as u16;
            let size_y = content.get("size_y").and_then(|v| v.as_u64()).unwrap_or(8) as u16;
            let size_z = content.get("size_z").and_then(|v| v.as_u64()).unwrap_or(1) as u16;

            // 기본적으로 빈 8비트 grayscale 이미지 생성
            let data_size = (size_x as usize) * (size_y as usize) * (size_z as usize);
            let data = vec![0u8; data_size];

            use openigtlink_rust::protocol::types::image::{ImageScalarType, Endian, CoordinateSystem};

            let image = ImageMessage {
                version: 1,
                num_components: 1,
                scalar_type: ImageScalarType::Uint8,
                endian: Endian::Big,
                coordinate: CoordinateSystem::RAS,
                size: [size_x, size_y, size_z],
                matrix: [
                    [1.0, 0.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                ],
                data,
            };
            let msg = IgtlMessage::new(image, device_name)
                .map_err(|e| format!("Failed to create image message: {}", e))?;
            Ok(AnyMessage::Image(msg))
        }
        "SENSOR" => {
            let status = content.get("status").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
            let unit = content.get("unit").and_then(|v| v.as_u64()).unwrap_or(0);
            let data = if let Some(data_array) = content.get("data").and_then(|d| d.as_array()) {
                data_array.iter()
                    .filter_map(|v| v.as_f64())
                    .collect::<Vec<f64>>()
            } else {
                vec![0.0] // 기본 데이터
            };

            let sensor = SensorMessage { status, unit, data };
            let msg = IgtlMessage::new(sensor, device_name)
                .map_err(|e| format!("Failed to create sensor message: {}", e))?;
            Ok(AnyMessage::Sensor(msg))
        }
        "CAPABILITY" => {
            let types = if let Some(types_array) = content.get("types").and_then(|t| t.as_array()) {
                types_array.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect::<Vec<String>>()
            } else {
                vec![]
            };

            let capability = CapabilityMessage { types };
            let msg = IgtlMessage::new(capability, device_name)
                .map_err(|e| format!("Failed to create capability message: {}", e))?;
            Ok(AnyMessage::Capability(msg))
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

/// Helper function to send AnyMessage through AsyncIgtlClient
async fn send_any_message(
    client: &std::sync::Arc<tokio::sync::Mutex<openigtlink_rust::io::AsyncIgtlClient>>,
    msg: &AnyMessage,
) -> Result<(), openigtlink_rust::error::IgtlError> {
    use tokio::io::AsyncWriteExt;

    let mut client_lock = client.lock().await;

    // Encode the message based on its type and send raw bytes
    match msg {
        AnyMessage::Transform(m) => client_lock.send(m).await,
        AnyMessage::Status(m) => client_lock.send(m).await,
        AnyMessage::String(m) => client_lock.send(m).await,
        AnyMessage::Position(m) => client_lock.send(m).await,
        AnyMessage::Image(m) => client_lock.send(m).await,
        AnyMessage::Sensor(m) => client_lock.send(m).await,
        AnyMessage::Capability(m) => client_lock.send(m).await,
        _ => Err(openigtlink_rust::error::IgtlError::Io(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Message type not supported for sending",
        ))),
    }
}

/// Helper function to send AnyMessage through AsyncIgtlConnection
async fn send_any_message_to_connection(
    conn: &std::sync::Arc<tokio::sync::Mutex<openigtlink_rust::io::AsyncIgtlConnection>>,
    msg: &AnyMessage,
) -> Result<(), openigtlink_rust::error::IgtlError> {
    use tokio::io::AsyncWriteExt;

    let mut conn_lock = conn.lock().await;

    // Encode the message based on its type and send raw bytes
    match msg {
        AnyMessage::Transform(m) => conn_lock.send(m).await,
        AnyMessage::Status(m) => conn_lock.send(m).await,
        AnyMessage::String(m) => conn_lock.send(m).await,
        AnyMessage::Position(m) => conn_lock.send(m).await,
        AnyMessage::Image(m) => conn_lock.send(m).await,
        AnyMessage::Sensor(m) => conn_lock.send(m).await,
        AnyMessage::Capability(m) => conn_lock.send(m).await,
        _ => Err(openigtlink_rust::error::IgtlError::Io(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Message type not supported for sending",
        ))),
    }
}
