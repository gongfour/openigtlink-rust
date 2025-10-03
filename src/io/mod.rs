//! Network I/O module for OpenIGTLink communication
//!
//! Provides client and server implementations for OpenIGTLink connections.

pub mod client;
pub mod server;

pub use client::IgtlClient;
pub use server::{IgtlConnection, IgtlServer};
