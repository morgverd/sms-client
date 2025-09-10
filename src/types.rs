//! Generic types that apply to both HTTP and Websocket interfaces.

use serde::{Serialize, Deserialize};

/// Represents a stored SMS message from the database.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SmsStoredMessage {

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
    pub created_at: Option<u32>,

    /// Optional Unix timestamp when the message was completed/delivered.
    pub completed_at: Option<u32>
}

/// A partial message delivery report, as it comes from the modem.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SmsPartialDeliveryReport {

    /// The target phone number that received the message (and has now sent back a delivery report).
    phone_number: String,

    /// The modem assigned message reference, this is basically useless outside short-term tracking
    /// the message_id is unique should always be used instead for identification.
    reference_id: u8,

    /// The SMS TP-Status: https://www.etsi.org/deliver/etsi_ts/123000_123099/123040/16.00.00_60/ts_123040v160000p.pdf#page=71
    status: u8
}

/// Represents the current status of the modem.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ModemStatusUpdateState {

    /// Modem is starting up.
    Startup,

    /// Modem is online and operational.
    Online,

    /// Modem is shutting down.
    ShuttingDown,

    /// Modem is offline and not operational.
    Offline
}

/// GNSS (Global Navigation Satellite System) fix status.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum GnssFixStatus {

    /// GNSS fix status is unknown.
    Unknown,

    /// No GNSS fix.
    NotFix,

    /// 2D GNSS fix (latitude and longitude only).
    Fix2D,

    /// 3D GNSS fix (latitude, longitude, and altitude).
    Fix3D
}

/// Represents a GNSS position report with optional fields for satellite info.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GnssPositionReport {

    /// Indicates whether the GNSS receiver is currently running.
    pub run_status: bool,

    /// Whether a valid fix has been obtained.
    pub fix_status: bool,

    /// UTC time of the position report in ISO 8601 format.
    pub utc_time: String,

    /// Latitude in decimal degrees.
    pub latitude: Option<f64>,

    /// Longitude in decimal degrees.
    pub longitude: Option<f64>,

    /// Mean sea level altitude in meters.
    pub msl_altitude: Option<f64>,

    /// Ground speed in meters per second.
    pub ground_speed: Option<f32>,

    /// Ground course in degrees.
    pub ground_course: Option<f32>,

    /// Fix mode indicating 2D/3D fix or unknown.
    pub fix_mode: GnssFixStatus,

    /// Horizontal Dilution of Precision.
    pub hdop: Option<f32>,

    /// Position Dilution of Precision.
    pub pdop: Option<f32>,

    /// Vertical Dilution of Precision.
    pub vdop: Option<f32>,

    /// Number of GPS satellites in view.
    pub gps_in_view: Option<u8>,

    /// Number of GNSS satellites used in the fix.
    pub gnss_used: Option<u8>,

    /// Number of GLONASS satellites in view.
    pub glonass_in_view: Option<u8>
}