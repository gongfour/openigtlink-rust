#[derive(Debug, Clone, PartialEq)]
pub enum TabType {
    Client,
    Server,
}

#[derive(Debug, Clone)]
pub struct Tab {
    pub id: usize,
    pub name: String,
    pub tab_type: TabType,
    pub host: String, // Client only
    pub port: String,
    pub is_connected: bool,
    pub send_panel_expanded: bool,
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
        }
    }
}
