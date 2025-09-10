//! WebSocket client for receiving real-time SMS messages.

use futures_util::{StreamExt, SinkExt};
use http::StatusCode;
use tungstenite::client::IntoClientRequest;
use crate::ws::error::*;
use crate::ws::types::*;

pub mod error;
pub mod types;

enum ControlMessage {
    Stop,
    Reconnect,
}

/// WebSocket client for real-time message reception.
pub struct WebsocketClient {
    config: crate::config::WebsocketConfig,
    callback: Option<MessageCallback>,
    control_tx: Option<tokio::sync::mpsc::UnboundedSender<ControlMessage>>,
    worker_handle: Option<tokio::task::JoinHandle<WebsocketResult<()>>>,
    is_connected: std::sync::Arc<tokio::sync::RwLock<bool>>
}
impl WebsocketClient {
    /// Create a new WebSocket client.
    pub fn new(config: crate::config::WebsocketConfig) -> Self {
        Self {
            config,
            callback: None,
            control_tx: None,
            worker_handle: None,
            is_connected: std::sync::Arc::new(tokio::sync::RwLock::new(false)),
        }
    }

    /// Set the message callback handler.
    pub fn on_message<F>(&mut self, callback: F)
    where
        F: Fn(WebsocketMessage) + Send + Sync + 'static,
    {
        self.callback = Some(std::sync::Arc::new(callback));
    }

    /// Start the WebSocket connection in the background (spawns a worker task).
    pub async fn start_background(&mut self) -> WebsocketResult<()> {
        if self.worker_handle.is_some() {
            return Err(WebsocketError::AlreadyConnected);
        }

        let (control_tx, control_rx) = tokio::sync::mpsc::unbounded_channel();
        self.control_tx = Some(control_tx);

        let config = self.config.clone();
        let callback = self.callback.clone();
        let is_connected = std::sync::Arc::clone(&self.is_connected);

        let worker_handle = tokio::spawn(async move {
            Self::worker_loop(config, callback, control_rx, is_connected).await
        });

        self.worker_handle = Some(worker_handle);
        Ok(())
    }

    /// Start the WebSocket connection and block until it closes.
    pub async fn start_blocking(&mut self) -> WebsocketResult<()> {
        let (control_tx, control_rx) = tokio::sync::mpsc::unbounded_channel();
        self.control_tx = Some(control_tx);

        let config = self.config.clone();
        let callback = self.callback.clone();
        let is_connected = std::sync::Arc::clone(&self.is_connected);

        // Run directly in this task (no spawn)
        Self::worker_loop(config, callback, control_rx, is_connected).await
    }

    /// Stop the WebSocket connection and worker.
    pub async fn stop_background(&mut self) -> WebsocketResult<()> {
        if let Some(tx) = &self.control_tx {
            let _ = tx.send(ControlMessage::Stop);
        }

        if let Some(handle) = self.worker_handle.take() {
            // Wait for worker to finish with timeout
            let _ = tokio::time::timeout(std::time::Duration::from_secs(5), handle).await;
        }

        self.control_tx = None;
        *self.is_connected.write().await = false;

        Ok(())
    }

    /// Check if the WebSocket is currently connected.
    pub async fn is_connected(&self) -> bool {
        *self.is_connected.read().await
    }

    /// Force a reconnection attempt.
    pub async fn reconnect(&self) -> WebsocketResult<()> {
        if let Some(tx) = &self.control_tx {
            tx.send(ControlMessage::Reconnect)
                .map_err(|_| WebsocketError::ChannelError)?;
            Ok(())
        } else {
            Err(WebsocketError::NotConnected)
        }
    }

    /// Main worker loop that handles connection and reconnection.
    async fn worker_loop(
        config: crate::config::WebsocketConfig,
        callback: Option<MessageCallback>,
        mut control_rx: tokio::sync::mpsc::UnboundedReceiver<ControlMessage>,
        is_connected: std::sync::Arc<tokio::sync::RwLock<bool>>,
    ) -> WebsocketResult<()> {
        let mut reconnect_count = 0u32;

        loop {
            // Try to establish connection
            match Self::connect_and_handle(&config, &callback, &mut control_rx, &is_connected).await {
                Ok(should_reconnect) => {
                    if !should_reconnect || !config.auto_reconnect {
                        break;
                    }
                    reconnect_count += 1;
                }
                Err(e) => {
                    // If unauthorized, there is no use in attempting reconnections.
                    log::error!("WebSocket error: {:#?}", e);
                    if matches!(e, WebsocketError::Unauthorized) {
                        return Err(e);
                    }
                    if !config.auto_reconnect {
                        break;
                    }
                    reconnect_count += 1;
                }
            }

            *is_connected.write().await = false;

            // Calculate backoff delay (capped at 60 seconds)
            let delay = std::cmp::min(
                config.reconnect_interval * reconnect_count,
                std::time::Duration::from_secs(60),
            );

            // Wait before reconnecting, but check for stop signal
            log::trace!("Reconnecting in {:?}...", delay);
            tokio::select! {
                _ = tokio::time::sleep(delay) => {},
                Some(ControlMessage::Stop) = control_rx.recv() => {
                    log::trace!("WebSocket worker stopped during reconnect delay.");
                    break;
                }
            }
        }

        *is_connected.write().await = false;
        log::trace!("WebSocket worker terminated");
        Ok(())
    }

    /// Connect to WebSocket and handle messages.
    async fn connect_and_handle(
        config: &crate::config::WebsocketConfig,
        callback: &Option<MessageCallback>,
        control_rx: &mut tokio::sync::mpsc::UnboundedReceiver<ControlMessage>,
        is_connected: &std::sync::Arc<tokio::sync::RwLock<bool>>,
    ) -> WebsocketResult<bool> {
        log::debug!("Connecting to WebSocket: {}", config.url);

        // Create request with optional authorization.
        let mut request = config.url.clone().into_client_request()?;
        if let Some(token) = &config.authorization {
            request
                .headers_mut()
                .append("authorization", token.parse()?);
        }

        // Start connection, throwing a special error if unauthorized
        // to prevent reconnection attempts due to token failures.
        let ws_stream = match tokio_tungstenite::connect_async(request).await {
            Ok((ws_stream, _)) => ws_stream,
            Err(e) => {
                match &e {
                    tungstenite::error::Error::Http(response) => {

                        // If it's a connection Http error, we should check that it's not because it's Unauthorized.
                        // If it is, there is no use in attempting reconnections as the token is invalid.
                        if response.status() == StatusCode::UNAUTHORIZED {
                            return Err(WebsocketError::Unauthorized);
                        }
                    },
                    _ => { }
                }
                return Err(WebsocketError::from(e));
            }
        };

        *is_connected.write().await = true;
        log::debug!("WebSocket connected successfully");

        let mut ping_interval = tokio::time::interval(config.ping_interval);
        ping_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        let mut last_pong_time = tokio::time::Instant::now();
        let mut waiting_for_pong = false;

        let (mut write, mut read) = ws_stream.split();
        loop {
            tokio::select! {
                Some(msg) = read.next() => {
                    match msg {
                        Ok(tungstenite::Message::Text(text)) => {
                            match serde_json::from_str::<WebsocketMessage>(&text) {
                                Ok(ws_msg) => {
                                    // Call user callback.
                                    if let Some(cb) = callback {
                                        cb(ws_msg);
                                    }
                                },
                                Err(e) => {
                                    log::warn!("Invalid WebSocket message: {:?} -> {:#?}", text, e);
                                }
                            }
                        },
                        Ok(tungstenite::Message::Close(frame)) => {
                            log::debug!("WebSocket closed by server: {:?}", frame);
                            return Ok(true); // Should reconnect
                        },
                        Ok(tungstenite::Message::Ping(data)) => {
                            // Respond to ping with pong
                            log::trace!("Received WebSocket ping, sending pong");
                            if write.send(tungstenite::Message::Pong(data)).await.is_err() {
                                return Err(WebsocketError::SendError);
                            }
                        },
                        Ok(tungstenite::Message::Pong(_)) => {
                            // Pong frame
                            last_pong_time = tokio::time::Instant::now();
                            waiting_for_pong = false;
                            log::trace!("Received native WebSocket pong frame");
                        },
                        Err(e) => {
                            log::error!("WebSocket receive error: {}", e);
                            return Ok(true); // Should reconnect
                        },
                        _ => { }
                    }
                },

                // Send periodic pings
                _ = ping_interval.tick() => {
                    // Check if we've received a pong recently
                    let time_since_last_pong = tokio::time::Instant::now() - last_pong_time;

                    if waiting_for_pong && time_since_last_pong > config.ping_timeout {
                        log::trace!("Ping timeout - no pong received for {:?}", time_since_last_pong);
                        return Ok(true); // Should reconnect
                    }

                    // Send ping
                    log::trace!("Sending ping");
                    if write.send(tungstenite::Message::Ping(Vec::new().into())).await.is_err() {
                        log::trace!("Failed to send ping");
                        return Ok(true); // Should reconnect
                    }

                    waiting_for_pong = true;
                },

                // Handle control messages
                Some(msg) = control_rx.recv() => {
                    return match msg {
                        ControlMessage::Stop => {
                            log::trace!("Received stop signal");
                            let _ = write.send(tungstenite::Message::Close(None)).await;
                            Ok(false) // Should not reconnect
                        }
                        ControlMessage::Reconnect => {
                            log::trace!("Received reconnect signal");
                            let _ = write.send(tungstenite::Message::Close(None)).await;
                            Ok(true) // Should reconnect
                        }
                    }
                }
            }
        }
    }
}
impl Drop for WebsocketClient {
    fn drop(&mut self) {
        // Send stop signal to worker if still running.
        if let Some(tx) = &self.control_tx {
            let _ = tx.send(ControlMessage::Stop);
        }
    }
}