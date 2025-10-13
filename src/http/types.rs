//! HTTP interface related request/response types.

use serde::{Deserialize, Serialize};

/// HTTP pagination options allow for lazy reading of large sets of data,
/// for example if thousands of messages have been sent and received from
/// a phone number it would be impractical to request all of them at the
/// same time, instead it can be read in shorter pages using limit+offset.
/// This is applied at the server level when requesting data from database.
#[derive(Serialize, PartialEq, Default, Debug, Clone, Copy)]
pub struct HttpPaginationOptions {
    /// The maximum amount of return values.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u64>,

    /// The offset in index to start getting values from.
    /// Eg, if the limit was 5, and you want to view page 2,
    /// the offset would be 5, then 10, 15, ...
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u64>,

    /// Should return values be reversed? This is useful for getting the
    /// first results from a large set without having to know it's size.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reverse: Option<bool>,
}
impl HttpPaginationOptions {
    /// Set the limit/page size.
    #[must_use]
    pub fn with_limit(mut self, limit: u64) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set request position offset.
    #[must_use]
    pub fn with_offset(mut self, offset: u64) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Set the reverse state for options.
    #[must_use]
    pub fn with_reverse(mut self, reverse: bool) -> Self {
        self.reverse = Some(reverse);
        self
    }

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
#[derive(Serialize, PartialEq, Default, Debug, Clone)]
pub struct HttpOutgoingSmsMessage {
    /// The target phone number, this should be in international format.
    pub to: String,

    /// The full message content. This will be split into multiple messages
    /// by the server if required. This also supports Unicode emojis etc.
    pub content: String,

    /// The relative validity period to use for message sending. This determines
    /// how long the message should remain waiting while undelivered.
    /// By default, this is determined by the server (24 hours).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validity_period: Option<u8>,

    /// Should the SMS message be sent as a Silent class? This makes a popup
    /// show on the users device with the message content if they're logged in.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flash: Option<bool>,

    /// A timeout that should be applied to the entire request.
    /// If one is not set, the default timeout is used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u32>,
}
impl HttpOutgoingSmsMessage {
    /// Create a new outgoing message with a default validity period and no flash.
    /// The default validity period is applied by SMS-API, so usually 24 hours.
    pub fn simple_message(to: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            to: to.into(),
            content: content.into(),
            ..Default::default()
        }
    }

    /// Set the message flash state. This will show a popup if the recipient is
    /// logged-in to their phone, otherwise as a normal text message.
    #[must_use]
    pub fn with_flash(mut self, flash: bool) -> Self {
        self.flash = Some(flash);
        self
    }

    /// Set a relative validity period value.
    #[must_use]
    pub fn with_validity_period(mut self, period: u8) -> Self {
        self.validity_period = Some(period);
        self
    }

    /// Set a request timeout value.
    #[must_use]
    pub fn with_timeout(mut self, timeout: u32) -> Self {
        self.timeout = Some(timeout);
        self
    }
}

/// Response returned after sending an SMS message.
#[derive(Deserialize, PartialEq, Debug, Clone, Copy)]
pub struct HttpSmsSendResponse {
    /// The unique ID assigned to the already sent message.
    pub message_id: i64,

    /// Reference ID for tracking the message.
    pub reference_id: u8,
}

/// Delivery report for an already sent SMS message.
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Copy)]
pub struct HttpSmsDeliveryReport {
    /// Unique identifier for this delivery report.
    pub report_id: Option<i64>,

    /// Delivery status code from the network.
    pub status: u8,

    /// Whether this is the final delivery report for the message.
    pub is_final: bool,

    /// Unix timestamp when this report was created.
    pub created_at: Option<u32>,
}

/// Network registration status of the modem.
#[derive(Deserialize, PartialEq, Debug, Clone, Copy)]
pub struct HttpModemNetworkStatusResponse {
    /// Registration status code (0=not registered, 1=registered home, 5=registered roaming).
    pub registration: u8,

    /// Network technology in use (e.g., 2G, 3G, 4G).
    pub technology: u8,
}

/// Signal strength information from the modem.
#[derive(Deserialize, PartialEq, Debug, Clone, Copy)]
pub struct HttpModemSignalStrengthResponse {
    /// Received Signal Strength Indicator (0-31, 99=unknown).
    pub rssi: u8,

    /// Bit Error Rate (0-7, 99=unknown).
    pub ber: u8,
}

/// Network operator information from the modem.
#[derive(Deserialize, PartialEq, Debug, Clone)]
pub struct HttpModemNetworkOperatorResponse {
    /// Operator selection status (0=automatic, 1=manual).
    pub status: u8,

    /// Format of the operator name (0=long alphanumeric, 1=short alphanumeric, 2=numeric).
    pub format: u8,

    /// Name or code of the network operator.
    pub operator: String,
}

/// Battery status information from the modem.
#[derive(Deserialize, PartialEq, Debug, Clone, Copy)]
pub struct HttpModemBatteryLevelResponse {
    /// Battery status (0=not charging, 1=charging, 2=no battery).
    pub status: u8,

    /// Battery charge level percentage (0-100).
    pub charge: u8,

    /// Battery voltage in volts.
    pub voltage: f32,
}

/// Combine an outgoing message and send response into a dummy `SmsStoredMessage`.
impl From<(HttpOutgoingSmsMessage, HttpSmsSendResponse)> for crate::types::SmsStoredMessage {
    fn from(
        value: (HttpOutgoingSmsMessage, HttpSmsSendResponse),
    ) -> crate::types::SmsStoredMessage {
        crate::types::SmsStoredMessage {
            message_id: value.1.message_id,
            phone_number: value.0.to,
            message_content: value.0.content,
            message_reference: Some(value.1.reference_id),
            is_outgoing: true,
            status: "Unknown".to_string(),
            created_at: None,
            completed_at: None,
        }
    }
}

/// The raw `DeviceInfoResponse` with raw values.
#[derive(Deserialize, PartialEq, Debug, Clone)]
pub struct HttpSmsDeviceInfoResponse {
    /// SMS API version string, including features.
    pub version: String,

    /// The phone number associated with the SMS device
    pub phone_number: Option<String>,

    /// The name of the cellular service provider
    pub service_provider: Option<String>,

    /// Network operator information as (code1, code2, `operator_name`)
    pub network_operator: Option<(u8, u8, String)>,

    /// Current network connection status as (`status_code`, `strength_indicator`)
    pub network_status: Option<(u8, u8)>,

    /// Battery information as (`level_percentage`, `charging_status`, voltage)
    pub battery: Option<(u8, u8, f32)>,

    /// Signal strength information as (`strength_level`, `quality_indicator`)
    pub signal: Option<(u8, u8)>,
}

/// Formatted device info response, with each value packed into a proper optional response.
#[derive(Deserialize, PartialEq, Debug, Clone)]
pub struct HttpSmsDeviceInfoData {
    /// SMS API version string, including features.
    pub version: String,

    /// The phone number associated with the SMS device
    pub phone_number: Option<String>,

    /// The name of the cellular service provider
    pub service_provider: Option<String>,

    /// Detailed network operator information and capabilities
    pub network_operator: Option<HttpModemNetworkOperatorResponse>,

    /// Current network connection status and diagnostics
    pub network_status: Option<HttpModemNetworkStatusResponse>,

    /// Battery level, charging state, and power metrics
    pub battery: Option<HttpModemBatteryLevelResponse>,

    /// Signal strength measurements and quality indicators
    pub signal: Option<HttpModemSignalStrengthResponse>,
}
impl From<HttpSmsDeviceInfoResponse> for HttpSmsDeviceInfoData {
    fn from(value: HttpSmsDeviceInfoResponse) -> HttpSmsDeviceInfoData {
        HttpSmsDeviceInfoData {
            version: value.version,
            phone_number: value.phone_number,
            service_provider: value.service_provider,
            network_operator: value
                .network_operator
                .map(|v| HttpModemNetworkOperatorResponse {
                    status: v.0,
                    format: v.1,
                    operator: v.2,
                }),
            network_status: value
                .network_status
                .map(|v| HttpModemNetworkStatusResponse {
                    registration: v.0,
                    technology: v.1,
                }),
            battery: value.battery.map(|v| HttpModemBatteryLevelResponse {
                status: v.0,
                charge: v.1,
                voltage: v.2,
            }),
            signal: value.signal.map(|v| HttpModemSignalStrengthResponse {
                rssi: v.0,
                ber: v.1,
            }),
        }
    }
}

/// Used in latest-numbers return value, as a number and friendly name.
pub type LatestNumberFriendlyNamePair = (String, Option<String>);
