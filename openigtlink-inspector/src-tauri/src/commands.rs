use crate::connection::ConnectionManager;
use crate::types::Tab;
use std::sync::Mutex;
use tauri::State;

/// Client 연결 명령
#[tauri::command]
pub async fn connect_client(
    host: String,
    port: u16,
    connection: State<'_, Mutex<ConnectionManager>>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let mut conn = connection.lock().map_err(|e| e.to_string())?;
    conn.connect_client(host, port, app).await
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
) -> Result<serde_json::json::Value, String> {
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
