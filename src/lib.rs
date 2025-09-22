//! A client library for SMS-API, via HTTP and an optional websocket connection.
//! <https://github.com/morgverd/sms-api>

#![deny(missing_docs)]
#![deny(unsafe_code)]
#![warn(clippy::all, clippy::pedantic)]

use crate::error::*;

pub mod config;
pub mod error;
pub mod types;

#[cfg(feature = "http")]
pub mod http;

#[cfg(feature = "websocket")]
pub mod ws;

/// SMS Client.
#[derive(Clone, Debug)]
pub struct Client {

    #[cfg(feature = "http")]
    http_client: Option<std::sync::Arc<http::HttpClient>>,

    #[cfg(feature = "websocket")]
    ws_client: Option<std::sync::Arc<tokio::sync::Mutex<ws::WebSocketClient>>>
}
impl Client {

    /// Create an SMS client with a connection config.
    pub fn new(config: config::ClientConfig) -> ClientResult<Self> {
        let tls = config.tls;

        #[cfg(feature = "http")]
        let http_client = if let Some(http_config) = config.http {
            Some(std::sync::Arc::new(
                http::HttpClient::new(http_config, &tls)?
            ))
        } else {
            None
        };

        #[cfg(feature = "websocket")]
        let ws_client = config.websocket.map(|ws_config| {
            std::sync::Arc::new(tokio::sync::Mutex::new(
                ws::WebSocketClient::new(ws_config, tls)
            ))
        });

        Ok(Self {
            #[cfg(feature = "http")]
            http_client,

            #[cfg(feature = "websocket")]
            ws_client
        })
    }

    /// Borrow the optional inner HTTP client.
    #[cfg(feature = "http")]
    pub fn http(&self) -> ClientResult<&http::HttpClient> {
        self.http_client
            .as_ref()
            .map(|arc| arc.as_ref())
            .ok_or(ClientError::ConfigError("HttpClient"))
    }

    /// Get a cloned Arc to the optional HTTP client for use in async contexts.
    #[cfg(feature = "http")]
    pub fn http_arc(&self) -> ClientResult<std::sync::Arc<http::HttpClient>> {
        self.http_client
            .clone()
            .ok_or(ClientError::ConfigError("HttpClient"))
    }

    /// Set the callback for incoming WebSocket messages. The callback will include the WebSocket
    /// message and an Arc to the current Client allowing for easy use within the callback!
    /// This must be called before starting the WebSocket connection.
    ///
    /// # Example
    /// ```
    /// use sms_client::http::types::HttpOutgoingSmsMessage;
    /// use sms_client::ws::types::WebsocketMessage;
    /// use sms_client::Client;
    /// use log::info;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client: Client = unimplemented!("See other examples");
    ///
    ///     client.on_message(move |message, client| {
    ///         match message {
    ///             WebsocketMessage::IncomingMessage(sms) => {
    ///                 // Can access client.http() here!
    ///             },
    ///             _ => { }
    ///         }
    ///     }).await?
    /// }
    /// ```
    #[cfg(feature = "websocket")]
    pub async fn on_message<F>(&self, callback: F) -> ClientResult<()>
    where
        F: Fn(ws::types::WebsocketMessage, std::sync::Arc<Self>) + Send + Sync + 'static,
    {
        let ws_client = self.ws_client
            .as_ref()
            .ok_or(ClientError::ConfigError("WebSocketClient"))?;

        let mut ws_guard = ws_client.lock().await;
        let client_arc = std::sync::Arc::new(self.clone());

        ws_guard.on_message(move |msg| {
            callback(msg, std::sync::Arc::clone(&client_arc))
        });

        Ok(())
    }

    /// Set the callback for incoming WebSocket messages (simple version without client copy).
    /// This must be called before starting the WebSocket connection.
    ///
    /// # Example
    /// ```
    /// use sms_client::Client;
    /// use sms_client::ws::types::WebsocketMessage;
    /// use log::info;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client: Client = unimplemented!("See other examples");
    ///
    ///     client.on_message_simple(move |message| {
    ///         match message {
    ///             WebsocketMessage::OutgoingMessage(sms) => info!("Outgoing message: {:?}", sms),
    ///             _ => { }
    ///         }
    ///     }).await?
    /// }
    /// ```
    #[cfg(feature = "websocket")]
    pub async fn on_message_simple<F>(&self, callback: F) -> ClientResult<()>
    where
        F: Fn(ws::types::WebsocketMessage) + Send + Sync + 'static,
    {
        let ws_client = self.ws_client
            .as_ref()
            .ok_or(ClientError::ConfigError("WebSocketClient"))?;

        let mut ws_guard = ws_client.lock().await;
        ws_guard.on_message(callback);

        Ok(())
    }

    /// Start the WebSocket connection.
    #[cfg(feature = "websocket")]
    pub async fn start_background_websocket(&self) -> ClientResult<()> {
        let ws_client = self.ws_client
            .as_ref()
            .ok_or(ClientError::ConfigError("WebsocketConfig"))?;

        let mut ws_guard = ws_client.lock().await;
        ws_guard.start_background().await?;

        Ok(())
    }

    /// Start the WebSocket connection and block until closed.
    #[cfg(feature = "websocket")]
    pub async fn start_blocking_websocket(&self) -> ClientResult<()> {
        let ws_client = self.ws_client
            .as_ref()
            .ok_or(ClientError::ConfigError("WebsocketConfig"))?;

        let mut ws_guard = ws_client.lock().await;
        ws_guard.start_blocking().await?;

        Ok(())
    }

    /// Stop the WebSocket connection.
    #[cfg(feature = "websocket")]
    pub async fn stop_background_websocket(&self) -> ClientResult<()> {
        let ws_client = self.ws_client
            .as_ref()
            .ok_or(ClientError::ConfigError("WebsocketConfig"))?;

        let mut ws_guard = ws_client.lock().await;
        ws_guard.stop_background().await?;

        Ok(())
    }

    /// Check if the WebSocket is currently connected.
    #[cfg(feature = "websocket")]
    pub async fn is_websocket_connected(&self) -> bool {
        let Some(ws_client) = &self.ws_client else {
            return false;
        };

        let ws_guard = ws_client.lock().await;
        ws_guard.is_connected().await
    }

    /// Force a WebSocket reconnection.
    #[cfg(feature = "websocket")]
    pub async fn reconnect_websocket(&self) -> ClientResult<()> {
        let ws_client = self.ws_client
            .as_ref()
            .ok_or(ClientError::NoWebsocketClient)?;

        let ws_guard = ws_client.lock().await;
        ws_guard.reconnect().await.map_err(ClientError::from)
    }
}
impl Drop for Client {
    fn drop(&mut self) {
        // The WebSocket client will handle its own cleanup in its Drop impl
        // This is just here to ensure proper cleanup ordering.
    }
}