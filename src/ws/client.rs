//! Main WebSocket client implementation.

use crate::ws::error::*;
use crate::ws::worker::{ControlMessage, WorkerLoop};
use sms_types::websocket::*;

/// WebSocket client for real-time message reception.
pub struct WebSocketClient {
    config: crate::config::WebSocketConfig,
    tls_config: Option<crate::config::TLSConfig>,
    callback: Option<crate::ws::MessageCallback>,
    control_tx: Option<tokio::sync::mpsc::UnboundedSender<ControlMessage>>,
    worker_handle: Option<tokio::task::JoinHandle<WebsocketResult<()>>>,
    is_connected: std::sync::Arc<tokio::sync::RwLock<bool>>,
}
impl WebSocketClient {
    /// Create a new WebSocket client.
    pub fn new(
        config: crate::config::WebSocketConfig,
        tls_config: Option<crate::config::TLSConfig>,
    ) -> Self {
        Self {
            config,
            tls_config,
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

        let worker_loop = WorkerLoop::new(
            self.config.clone(),
            self.tls_config.clone(),
            self.callback.clone(),
            std::sync::Arc::clone(&self.is_connected),
        );

        let worker_handle = tokio::spawn(async move { worker_loop.run(control_rx).await });

        self.worker_handle = Some(worker_handle);
        Ok(())
    }

    /// Start the WebSocket connection and block until it closes.
    pub async fn start_blocking(&mut self) -> WebsocketResult<()> {
        let (control_tx, control_rx) = tokio::sync::mpsc::unbounded_channel();
        self.control_tx = Some(control_tx);

        let worker_loop = WorkerLoop::new(
            self.config.clone(),
            self.tls_config.clone(),
            self.callback.clone(),
            std::sync::Arc::clone(&self.is_connected),
        );

        // Run directly in this task (no spawn)
        worker_loop.run(control_rx).await
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
}
impl Drop for WebSocketClient {
    fn drop(&mut self) {
        // Send stop signal to worker if still running.
        if let Some(tx) = &self.control_tx {
            let _ = tx.send(ControlMessage::Stop);
        }
    }
}
impl std::fmt::Debug for WebSocketClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebsocketClient")
            .field("url", &self.config.url)
            .field("is_connected", &self.is_connected)
            .field("has_tls_config", &self.tls_config.is_some())
            .finish()
    }
}
