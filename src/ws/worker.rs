//! WebSocket worker loop and message handling.

use futures_util::{SinkExt, StreamExt};
use crate::ws::connection::ConnectionParams;
use crate::ws::error::*;
use crate::ws::types::*;

/// Control messages for the worker loop
pub enum ControlMessage {
    Stop,
    Reconnect,
}

/// Action to take after processing a message
enum MessageAction {
    Continue,
    Reconnect,
}

/// Worker loop handler
pub struct WorkerLoop {
    config: crate::config::WebSocketConfig,
    tls_config: Option<crate::config::TLSConfig>,
    callback: Option<MessageCallback>,
    is_connected: std::sync::Arc<tokio::sync::RwLock<bool>>
}
impl WorkerLoop {
    /// Create a new worker loop
    pub fn new(
        config: crate::config::WebSocketConfig,
        tls_config: Option<crate::config::TLSConfig>,
        callback: Option<MessageCallback>,
        is_connected: std::sync::Arc<tokio::sync::RwLock<bool>>
    ) -> Self {
        Self {
            config,
            tls_config,
            callback,
            is_connected,
        }
    }

    /// Run the worker loop
    pub async fn run(
        self,
        mut control_rx: tokio::sync::mpsc::UnboundedReceiver<ControlMessage>,
    ) -> WebsocketResult<()> {
        let mut reconnect_count = 0u32;

        // Create connection parameters
        let connection_params = ConnectionParams::from_config(&self.config, &self.tls_config)?;

        loop {
            // Try to establish connection and handle messages
            match self.handle_connection(&connection_params, &mut control_rx).await {
                Ok(should_reconnect) => {
                    // Emit disconnection event
                    let will_reconnect = should_reconnect && self.config.auto_reconnect;
                    self.emit_connection_update(false, will_reconnect);

                    if !will_reconnect {
                        break;
                    }
                    reconnect_count += 1;
                }
                Err(e) => {
                    if matches!(e, WebsocketError::Unauthorized) {
                        return Err(e);
                    }

                    // Emit disconnection event
                    let will_reconnect = self.config.auto_reconnect;
                    self.emit_connection_update(false, will_reconnect);

                    log::error!("WebSocket error: {:#?}", e);
                    if !will_reconnect {
                        break;
                    }
                    reconnect_count += 1;
                }
            }

            *self.is_connected.write().await = false;

            // Backoff delay (capped at 60 seconds)
            let delay = std::cmp::min(
                self.config.reconnect_interval * reconnect_count,
                std::time::Duration::from_secs(60),
            );

            // Wait before reconnecting, but check for stop signal
            log::debug!("Reconnecting in {:?}...", delay);
            tokio::select! {
                _ = tokio::time::sleep(delay) => {},
                Some(ControlMessage::Stop) = control_rx.recv() => {
                    log::debug!("WebSocket worker stopped during reconnect delay.");
                    break;
                }
            }
        }

        *self.is_connected.write().await = false;
        log::debug!("WebSocket worker terminated");
        Ok(())
    }

    /// Handle an active WebSocket connection
    async fn handle_connection(
        &self,
        connection_params: &ConnectionParams,
        control_rx: &mut tokio::sync::mpsc::UnboundedReceiver<ControlMessage>,
    ) -> WebsocketResult<bool> {
        // Establish connection
        let ws_stream = connection_params.connect().await?;

        *self.is_connected.write().await = true;
        self.emit_connection_update(true, false);

        // Set up ping interval
        let mut ping_interval = tokio::time::interval(self.config.ping_interval);
        ping_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        let mut last_pong_time = tokio::time::Instant::now();
        let mut waiting_for_pong = false;

        let (mut write, mut read) = ws_stream.split();
        loop {
            tokio::select! {
                Some(msg) = read.next() => {
                    match self.handle_message(msg, &mut write, &mut last_pong_time, &mut waiting_for_pong).await? {
                        MessageAction::Continue => continue,
                        MessageAction::Reconnect => return Ok(true),
                    }
                }

                _ = ping_interval.tick() => {
                    if self.should_send_ping(waiting_for_pong, last_pong_time).await? {
                        if write.send(tungstenite::Message::Ping(Vec::new().into())).await.is_err() {
                            log::trace!("Failed to send ping");
                            return Ok(true);
                        }
                        waiting_for_pong = true;
                    } else {
                        return Ok(true); // Ping timeout, reconnect
                    }
                }

                Some(msg) = control_rx.recv() => {
                    return self.handle_control_message(msg, &mut write).await;
                }
            }
        }
    }

    /// Handle incoming WebSocket messages
    async fn handle_message(
        &self,
        msg: Result<tungstenite::Message, tungstenite::Error>,
        write: &mut futures_util::stream::SplitSink<
            tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
            tungstenite::Message
        >,
        last_pong_time: &mut tokio::time::Instant,
        waiting_for_pong: &mut bool,
    ) -> WebsocketResult<MessageAction> {
        match msg {
            Ok(tungstenite::Message::Text(text)) => {
                self.process_text_message(text.to_string());
                Ok(MessageAction::Continue)
            }
            Ok(tungstenite::Message::Close(frame)) => {
                log::debug!("WebSocket closed by server: {:?}", frame);
                Ok(MessageAction::Reconnect)
            }
            Ok(tungstenite::Message::Ping(data)) => {
                log::trace!("Received WebSocket ping, sending pong");
                if write.send(tungstenite::Message::Pong(data)).await.is_err() {
                    return Err(WebsocketError::SendError);
                }
                Ok(MessageAction::Continue)
            }
            Ok(tungstenite::Message::Pong(_)) => {
                *last_pong_time = tokio::time::Instant::now();
                *waiting_for_pong = false;
                log::trace!("Received native WebSocket pong frame");
                Ok(MessageAction::Continue)
            }
            Err(e) => {
                log::error!("WebSocket receive error: {}", e);
                Ok(MessageAction::Reconnect)
            }
            _ => Ok(MessageAction::Continue),
        }
    }

    /// Process text message
    fn process_text_message(&self, text: String) {
        match serde_json::from_str::<WebsocketMessage>(&text) {
            Ok(ws_msg) => {
                if let Some(cb) = &self.callback {
                    cb(ws_msg);
                }
            }
            Err(e) => {
                log::warn!("Invalid WebSocket message: {:?} -> {:#?}", text, e);
            }
        }
    }

    /// Check if we should send a ping or timeout
    async fn should_send_ping(
        &self,
        waiting_for_pong: bool,
        last_pong_time: tokio::time::Instant,
    ) -> WebsocketResult<bool> {
        if waiting_for_pong {
            let time_since_last_pong = tokio::time::Instant::now() - last_pong_time;
            if time_since_last_pong > self.config.ping_timeout {
                log::trace!("Ping timeout - no pong received for {:?}", time_since_last_pong);
                return Ok(false);
            }
        }

        log::trace!("Sending ping");
        Ok(true)
    }

    /// Handle control messages
    async fn handle_control_message(
        &self,
        msg: ControlMessage,
        write: &mut futures_util::stream::SplitSink<
            tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
            tungstenite::Message
        >,
    ) -> WebsocketResult<bool> {
        match msg {
            ControlMessage::Stop => {
                log::trace!("Received stop signal");
                let _ = write.send(tungstenite::Message::Close(None)).await;
                Ok(false)
            }
            ControlMessage::Reconnect => {
                log::trace!("Received reconnect signal");
                let _ = write.send(tungstenite::Message::Close(None)).await;
                Ok(true)
            }
        }
    }

    /// Emit connection status update
    fn emit_connection_update(&self, connected: bool, reconnect: bool) {
        if let Some(cb) = &self.callback {
            cb(WebsocketMessage::WebsocketConnectionUpdate { connected, reconnect });
        }
    }
}