//! Websocket interface related message types.

use serde::{Deserialize, Serialize};

/// WebSocket message types that can be received from the server.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "data")]
pub enum WebsocketMessage {

    /// New SMS message received.
    #[serde(rename = "incoming")]
    IncomingMessage(crate::types::SmsStoredMessage),

    /// SMS message being sent from API or other connected client.
    #[serde(rename = "outgoing")]
    OutgoingMessage(crate::types::SmsStoredMessage),

    /// Delivery report update.
    #[serde(rename = "delivery")]
    DeliveryReport {
        /// The target message_id this delivery report applies to.
        /// This is determined from the message_reference and sender.
        message_id: i64,

        /// The received delivery report.
        report: crate::types::SmsPartialDeliveryReport
    },

    /// Modem hat connection status update.
    /// This can be either: Startup, Online, ShuttingDown, Offline
    #[serde(rename = "modem_status_update")]
    ModemStatusUpdate {

        /// Previous state from last update.
        previous: crate::types::ModemStatusUpdateState,

        /// Current state after update.
        current: crate::types::ModemStatusUpdateState
    },

    /// An unsolicited position report from GNSS.
    #[serde(rename = "gnss_position_report")]
    GnssPositionReport(crate::types::GnssPositionReport),

    /// WebSocket connection status update (client-side only).
    /// This message is generated locally when there is a connection or disconnection.
    WebsocketConnectionUpdate {

        /// Connection status: true = connected, false = disconnected
        connected: bool,

        /// If connection is false, will the client attempt to automatically reconnect?
        reconnect: bool
    }
}

/// A callback to be run when the websocket receives a message.
pub type MessageCallback = std::sync::Arc<dyn Fn(WebsocketMessage) + Send + Sync>;