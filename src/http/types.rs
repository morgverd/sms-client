//! HTTP interface related request/response types.

use serde::{Serialize, Deserialize};

/// HTTP pagination options allow for lazy reading of large sets of data,
/// for example if thousands of messages have been sent and received from
/// a phone number it would be impractical to request all of them at the
/// same time, instead it can be read in shorter pages using limit+offset.
/// This is applied at the server level when requesting data from database.
#[derive(Serialize, Default, Debug)]
pub struct HttpPaginationOptions {
    /// The maximum amount of return values.
    pub limit: Option<u64>,

    /// The offset in index to start getting values from.
    /// Eg, if the limit was 5, and you want to view page 2,
    /// the offset would be 5, then 10, 15, ...
    pub offset: Option<u64>,

    /// Should return values be reversed? This is useful for getting the
    /// first results from a large set without having to know it's size.
    pub reverse: Option<bool>
}
impl HttpPaginationOptions {

    /// Add pagination options to a json Value.
    pub fn add_to_body(&self, body: &mut serde_json::Value) {
        if let Some(limit) = self.limit {
            body["limit"] = serde_json::json!(limit);
        }
        if let Some(offset) = self.offset {
            body["offset"] = serde_json::json!(offset);
        }
        if let Some(reverse) = self.reverse {
            body["reverse"] = serde_json::json!(reverse);
        }
    }
}

/// The outgoing SMS message to be sent to a target number.
#[derive(Serialize, Debug, Default)]
pub struct HttpOutgoingSmsMessage {
    /// The target phone number, this should be in international format.
    pub to: String,

    /// The full message content. This will be split into multiple messages
    /// by the server if required. This also supports Unicode emojis etc.
    pub content: String,

    /// The relative validity period to use for message sending. This determines
    /// how long the message should remain waiting while undelivered.
    /// By default, this is determined by the server (24 hours).
    pub validity_period: Option<u8>,

    /// Should the SMS message be sent as a Silent class? This makes a popup
    /// show on the users device with the message content if they're logged in.
    pub flash: bool
}
impl HttpOutgoingSmsMessage {

    /// Create a new outgoing message with a default validity period and no flash.
    /// The default validity period is applied by SMS-API, so usually 24 hours.
    pub fn simple_message(
        to: impl Into<String>,
        content: impl Into<String>
    ) -> Self {
        Self {
            to: to.into(),
            content: content.into(),
            ..Default::default()
        }
    }

    /// Set the message flash state. This will show a popup if the recipient is
    /// logged-in to their phone, otherwise as a normal text message.
    pub fn with_flash(mut self, flash: bool) -> Self {
        self.flash = flash;
        self
    }

    /// Set a relative validity period value.
    pub fn with_validity_period(mut self, period: u8) -> Self {
        self.validity_period = Some(period);
        self
    }
}

/// Represents a stored SMS message from the database.
#[derive(Deserialize, Debug)]
pub struct HttpSmsStoredMessage {
    /// Unique identifier for the message.
    pub message_id: i64,

    /// The phone number associated with this message.
    pub phone_number: String,

    /// The actual text content of the message.
    pub message_content: String,

    /// Optional reference number for message tracking.
    /// This is assigned by the modem and is only present for outgoing messages.
    pub message_reference: Option<u8>,

    /// Whether this message was sent (true) or received (false).
    pub is_outgoing: bool,

    /// Current status of the message (e.g., "sent", "delivered", "failed").
    pub status: String,

    /// Unix timestamp when the message was created.
    pub created_at: u32,

    /// Optional Unix timestamp when the message was completed/delivered.
    pub completed_at: Option<u32>
}

/// Delivery report for an already sent SMS message.
#[derive(Deserialize, Debug)]
pub struct HttpSmsDeliveryReport {
    /// Unique identifier for this delivery report.
    pub report_id: i64,

    /// Delivery status code from the network.
    pub status: u8,

    /// Whether this is the final delivery report for the message.
    pub is_final: bool,

    /// Unix timestamp when this report was created.
    pub created_at: u32
}

/// Response returned after sending an SMS message.
#[derive(Deserialize, Debug)]
pub struct HttpSmsSendResponse {
    /// The unique ID assigned to the already sent message.
    pub message_id: i64,

    /// Reference ID for tracking the message.
    pub reference_id: u8
}

/// Network registration status of the modem.
#[derive(Deserialize, Debug)]
pub struct HttpModemNetworkStatusResponse {
    /// Registration status code (0=not registered, 1=registered home, 5=registered roaming).
    pub registration: u8,

    /// Network technology in use (e.g., 2G, 3G, 4G).
    pub technology: u8
}

/// Signal strength information from the modem.
#[derive(Deserialize, Debug)]
pub struct HttpModemSignalStrengthResponse {
    /// Received Signal Strength Indicator (0-31, 99=unknown).
    pub rssi: u8,

    /// Bit Error Rate (0-7, 99=unknown).
    pub ber: u8
}

/// Network operator information from the modem.
#[derive(Deserialize, Debug)]
pub struct HttpModemNetworkOperatorResponse {
    /// Operator selection status (0=automatic, 1=manual).
    pub status: u8,

    /// Format of the operator name (0=long alphanumeric, 1=short alphanumeric, 2=numeric).
    pub format: u8,

    /// Name or code of the network operator.
    pub operator: String
}

/// Battery status information from the modem.
#[derive(Deserialize, Debug)]
pub struct HttpModemBatteryLevelResponse {
    /// Battery status (0=not charging, 1=charging, 2=no battery).
    pub status: u8,

    /// Battery charge level percentage (0-100).
    pub charge: u8,

    /// Battery voltage in volts.
    pub voltage: f32
}