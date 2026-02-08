//! Server module for OpenCode communication

pub mod client;
pub mod discovery;

pub use client::Client;
pub use discovery::{discover_server, Server};
