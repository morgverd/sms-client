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

/// SMS Client with HTTP and optional WebSocket support.
#[derive(Clone, Debug)]
pub struct Client {

    #[cfg(feature = "http")]
    http: Option<std::sync::Arc<http::HttpClient>>,

    #[cfg(feature = "websocket")]
    ws_client: std::sync::Arc<tokio::sync::RwLock<Option<ws::WebSocketClient>>>,

    #[cfg(feature = "websocket")]
    ws_config: (Option<config::WebSocketConfig>, Option<config::TLSConfig>)
}
impl Client {

    /// Create an SMS client with a connection config.
    pub fn new(config: config::ClientConfig) -> ClientResult<Self> {
        let tls = config.tls;

        #[cfg(feature = "websocket-tls-rustls")]
        let _ = rustls::crypto::CryptoProvider::install_default(
            rustls::crypto::aws_lc_rs::default_provider()
        );

        Ok(Self {

            #[cfg(feature = "http")]
            http: config.http.map(|config| http::HttpClient::new(config, &tls).map(std::sync::Arc::new)).transpose()?,

            #[cfg(feature = "websocket")]
            ws_client: std::sync::Arc::new(tokio::sync::RwLock::new(None)),

            #[cfg(feature = "websocket")]
            ws_config: (config.websocket, tls)
        })
    }

    /// Borrow the optional inner HTTP client.
    #[cfg(feature = "http")]
    pub fn http(&self) -> ClientResult<&http::HttpClient> {
        match &self.http {
            Some(http) => Ok(http),
            None => Err(ClientError::ConfigError("HttpClient"))
        }
    }

    /// Get a cloned Arc to the optional HTTP client for use in async contexts.
    #[cfg(feature = "http")]
    pub fn http_arc(&self) -> ClientResult<std::sync::Arc<http::HttpClient>> {
        match &self.http {
            Some(http) => Ok(http.clone()),
            None => Err(ClientError::ConfigError("HttpClient"))
        }
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
        let mut ws_guard = self.create_or_get_ws_client().await?;
        if let Some(ws_client) = ws_guard.as_mut() {
            let client = std::sync::Arc::new(self.clone());
            ws_client.on_message(move |msg| {
                callback(msg, std::sync::Arc::clone(&client));
            });
        }
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
        let mut ws_guard = self.create_or_get_ws_client().await?;
        if let Some(ws_client) = ws_guard.as_mut() {
            ws_client.on_message(callback);
        }
        Ok(())
    }

    /// Start the WebSocket connection.
    #[cfg(feature = "websocket")]
    pub async fn start_background_websocket(&self) -> ClientResult<()> {
        let mut ws_guard = self.create_or_get_ws_client().await?;
        if let Some(ws_client) = ws_guard.as_mut() {
            ws_client.start_background().await?;
        }
        Ok(())
    }

    /// Start the WebSocket connection and block until closed.
    #[cfg(feature = "websocket")]
    pub async fn start_blocking_websocket(&self) -> ClientResult<()> {
        let mut ws_guard = self.create_or_get_ws_client().await?;
        if let Some(ws_client) = ws_guard.as_mut() {
            ws_client.start_blocking().await?;
        }
        Ok(())
    }

    /// Stop the WebSocket connection.
    #[cfg(feature = "websocket")]
    pub async fn stop_background_websocket(&self) -> ClientResult<()> {
        let mut ws_guard = self.ws_client.write().await;

        if let Some(ws_client) = ws_guard.as_mut() {
            ws_client.stop_background().await?;
        }

        Ok(())
    }

    /// Check if the WebSocket is currently connected.
    #[cfg(feature = "websocket")]
    pub async fn is_websocket_connected(&self) -> bool {
        let ws_guard = self.ws_client.read().await;

        if let Some(ws_client) = ws_guard.as_ref() {
            ws_client.is_connected().await
        } else {
            false
        }
    }

    /// Force a WebSocket reconnection.
    #[cfg(feature = "websocket")]
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
    #[cfg(feature = "websocket")]
    async fn create_or_get_ws_client(&self) -> ClientResult<tokio::sync::RwLockWriteGuard<'_, Option<ws::WebSocketClient>>> {
        let mut ws_guard = self.ws_client.write().await;
        if ws_guard.is_none() {
            let (ws_config, tls_config) = match self.ws_config.clone() {
                (Some(config), tls) => (config, tls),
                _ => return Err(ClientError::ConfigError("WebsocketConfig"))
            };

            let ws_client = ws::WebSocketClient::new(ws_config, tls_config);
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