use clap::Parser;

mod cli;
mod config;
mod server;
mod client;
mod msg_loader;

use cli::{Args, Commands};
use openigtlink_rust::error::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize tracing
    init_tracing(&args.log_level());

    match args.command {
        Commands::Server(server_args) => server::run_server(server_args).await,
        Commands::Client(client_args) => client::run_client(client_args).await,
    }
}

fn init_tracing(log_level: &str) {
    use tracing_subscriber::filter::EnvFilter;

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(log_level));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .init();
}
