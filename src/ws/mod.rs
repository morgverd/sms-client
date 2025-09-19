//! WebSocket client for receiving real-time SMS messages.

pub mod error;
pub mod types;

mod client;
mod tls;
mod worker;
mod connection;

pub use client::WebSocketClient;
pub use error::{WebsocketError, WebsocketResult};
pub use types::{WebsocketMessage, MessageCallback};