//! WebSocket client for receiving real-time SMS messages.

pub mod error;

mod client;
mod connection;
mod tls;
mod worker;

pub use client::WebSocketClient;
pub use error::{WebsocketError, WebsocketResult};

/// A callback to be run when the websocket receives a message.
pub type MessageCallback =
    std::sync::Arc<dyn Fn(sms_types::websocket::WebsocketMessage) + Send + Sync>;
