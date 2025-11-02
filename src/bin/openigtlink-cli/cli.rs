use clap::{Parser, Subcommand};

/// OpenIGTLink Protocol CLI Tool for message testing and debugging
#[derive(Parser, Debug)]
#[command(name = "openigtlink")]
#[command(about = "OpenIGTLink Protocol CLI Tool")]
#[command(version)]
#[command(author = "Wonjin Kang")]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,

    /// Log level: debug, info, warn, error
    #[arg(global = true, long, default_value = "info")]
    pub log_level: String,
}

impl Args {
    pub fn log_level(&self) -> &str {
        &self.log_level
    }
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Run as server (listen for connections)
    #[command(about = "Run OpenIGTLink server")]
    Server(ServerArgs),

    /// Run as client (connect to server)
    #[command(about = "Run OpenIGTLink client")]
    Client(ClientArgs),
}

/// Server command arguments
#[derive(Parser, Debug, Clone)]
pub struct ServerArgs {
    /// Address and port to listen on
    #[arg(long, default_value = "0.0.0.0:18944")]
    pub listen: String,

    /// Enable message sending
    #[arg(long, default_value_t = false)]
    pub send_enable: bool,

    /// Path to JSON file containing message to send
    #[arg(long)]
    pub send_message_file: Option<String>,

    /// Number of times to repeat sending the message
    #[arg(long, default_value = "1")]
    pub send_repeat_count: u32,

    /// Interval between sends in milliseconds
    #[arg(long, default_value = "1000")]
    pub send_interval_ms: u64,

    /// Device name for message (overrides JSON value)
    #[arg(long)]
    pub send_device_name: Option<String>,

    /// Enable message receiving
    #[arg(long, default_value_t = false)]
    pub receive_enable: bool,

    /// Message types to receive (comma-separated, or "all")
    #[arg(long, default_value = "all")]
    pub receive_message_types: String,

    /// Maximum number of messages to receive (0 = unlimited)
    #[arg(long, default_value = "0")]
    pub receive_max_count: u32,

    /// Timeout for receiving messages in seconds
    #[arg(long, default_value = "30")]
    pub receive_timeout_sec: u64,

    /// Path to file where received messages will be saved (JSON)
    #[arg(long)]
    pub receive_output_file: Option<String>,

    /// Output format: json, raw, text
    #[arg(long, default_value = "json")]
    pub receive_output_format: String,
}

/// Client command arguments
#[derive(Parser, Debug, Clone)]
pub struct ClientArgs {
    /// Address and port to connect to
    #[arg(long, required = true)]
    pub connect: String,

    /// Enable message sending
    #[arg(long, default_value_t = false)]
    pub send_enable: bool,

    /// Path to JSON file containing message to send
    #[arg(long)]
    pub send_message_file: Option<String>,

    /// Number of times to repeat sending the message
    #[arg(long, default_value = "1")]
    pub send_repeat_count: u32,

    /// Interval between sends in milliseconds
    #[arg(long, default_value = "1000")]
    pub send_interval_ms: u64,

    /// Device name for message (overrides JSON value)
    #[arg(long)]
    pub send_device_name: Option<String>,

    /// Enable message receiving
    #[arg(long, default_value_t = false)]
    pub receive_enable: bool,

    /// Message types to receive (comma-separated, or "all")
    #[arg(long, default_value = "all")]
    pub receive_message_types: String,

    /// Maximum number of messages to receive (0 = unlimited)
    #[arg(long, default_value = "0")]
    pub receive_max_count: u32,

    /// Timeout for receiving messages in seconds
    #[arg(long, default_value = "30")]
    pub receive_timeout_sec: u64,

    /// Path to file where received messages will be saved (JSON)
    #[arg(long)]
    pub receive_output_file: Option<String>,

    /// Output format: json, raw, text
    #[arg(long, default_value = "json")]
    pub receive_output_format: String,
}
