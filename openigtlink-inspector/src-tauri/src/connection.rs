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

    pub fn disconnect(&mut self) {
        self.is_connected = false;
    }
}
