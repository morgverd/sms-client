//! A client library for SMS-API, via HTTP and an optional websocket connection.
//! https://github.com/morgverd/sms-api

#![deny(missing_docs)]
#![deny(unsafe_code)]
#![warn(clippy::all, clippy::pedantic)]

use crate::error::*;

pub mod http;
pub mod ws;
pub mod config;
pub mod error;
pub mod types;

/// SMS Client with HTTP and optional WebSocket support.
#[derive(Clone)]
pub struct Client {
    http: std::sync::Arc<http::HttpClient>,
    ws_client: std::sync::Arc<tokio::sync::RwLock<Option<ws::WebsocketClient>>>,
    ws_config: Option<config::WebsocketConfig>
}
impl Client {
    /// Create an SMS client with a connection config.
    pub fn new(config: config::ClientConfig) -> ClientResult<Self> {
        let http = http::HttpClient::new(config.http)?;
        Ok(Self {
            http: std::sync::Arc::new(http),
            ws_client: std::sync::Arc::new(tokio::sync::RwLock::new(None)),
            ws_config: config.websocket
        })
    }

    /// Borrow the inner HTTP client.
    pub fn http(&self) -> &http::HttpClient {
        &self.http
    }

    /// Set the callback for incoming WebSocket messages.
    /// This must be called before starting the WebSocket connection.
    ///
    /// # Example
    /// ```no_run
    /// use sms_client::{Client, error::ClientResult, config::ClientConfig};
    /// async fn example() -> ClientResult<()> {
    ///     let mut client = Client::new(ClientConfig::with_websocket(
    ///         "http://localhost:3000",
    ///         "ws://localhost:3000/ws"
    ///     ).with_auth("test"))?;
    ///
    ///     client.on_message(|msg| {
    ///         println!("Received message: {:?}", msg);
    ///     }).await?;
    ///
    ///     client.start_blocking_websocket().await
    /// }
    /// ```
    pub async fn on_message<F>(&self, callback: F) -> ClientResult<()>
    where
        F: Fn(ws::types::WebsocketMessage) + Send + Sync + 'static,
    {
        let mut ws_guard = self.create_or_get_ws_client().await?;
        if let Some(ws_client) = ws_guard.as_mut() {
            ws_client.on_message(callback);
        }
        Ok(())
    }

    /// Start the WebSocket connection.
    pub async fn start_background_websocket(&self) -> ClientResult<()> {
        let mut ws_guard = self.create_or_get_ws_client().await?;
        if let Some(ws_client) = ws_guard.as_mut() {
            ws_client.start_background().await?;
        }
        Ok(())
    }

    /// Start the WebSocket connection and block until closed.
    pub async fn start_blocking_websocket(&self) -> ClientResult<()> {
        let mut ws_guard = self.create_or_get_ws_client().await?;
        if let Some(ws_client) = ws_guard.as_mut() {
            ws_client.start_blocking().await?;
        }
        Ok(())
    }

    /// Stop the WebSocket connection.
    pub async fn stop_background_websocket(&self) -> ClientResult<()> {
        let mut ws_guard = self.ws_client.write().await;

        if let Some(ws_client) = ws_guard.as_mut() {
            ws_client.stop_background().await?;
        }

        Ok(())
    }

    /// Check if the WebSocket is currently connected.
    pub async fn is_websocket_connected(&self) -> bool {
        let ws_guard = self.ws_client.read().await;

        if let Some(ws_client) = ws_guard.as_ref() {
            ws_client.is_connected().await
        } else {
            false
        }
    }

    /// Force a WebSocket reconnection.
    pub async fn reconnect_websocket(&self) -> ClientResult<()> {
        let ws_guard = self.ws_client.read().await;

        if let Some(ws_client) = ws_guard.as_ref() {
            ws_client.reconnect().await?;
            Ok(())
        } else {
            Err(ClientError::NoWebsocketClient)
        }
    }

    /// Create or return existing websocket client guard.
    async fn create_or_get_ws_client(&self) -> ClientResult<tokio::sync::RwLockWriteGuard<'_, Option<ws::WebsocketClient>>> {
        let mut ws_guard = self.ws_client.write().await;
        if ws_guard.is_none() {
            let config = match self.ws_config.clone() {
                Some(config) => config,
                None => return Err(ClientError::MissingConfiguration)
            };

            let ws_client = ws::WebsocketClient::new(config);
            *ws_guard = Some(ws_client);
        }

        Ok(ws_guard)
    }
}
impl Drop for Client {
    fn drop(&mut self) {
        // The WebSocket client will handle its own cleanup in its Drop impl
        // This is just here to ensure proper cleanup ordering.
    }
}