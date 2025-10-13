//! WebSocket client for receiving real-time SMS messages.

pub mod error;
pub mod types;

mod client;
mod connection;
mod tls;
mod worker;

pub use client::WebSocketClient;
pub use error::{WebsocketError, WebsocketResult};
pub use types::{MessageCallback, WebsocketMessage};
