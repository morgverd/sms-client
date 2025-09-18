//! Generic types that apply to both HTTP and Websocket interfaces.

use serde::{Serialize, Deserialize};

/// Represents a stored SMS message from the database.
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
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
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct SmsPartialDeliveryReport {

    /// The target phone number that received the message (and has now sent back a delivery report).
    phone_number: String,

    /// The modem assigned message reference, this is basically useless outside short-term tracking
    /// the message_id is unique should always be used instead for identification.
    reference_id: u8,

    /// The SMS TP-Status: <https://www.etsi.org/deliver/etsi_ts/123000_123099/123040/16.00.00_60/ts_123040v160000p.pdf#page=71>
    status: u8
}

/// <https://www.etsi.org/deliver/etsi_ts/123000_123099/123040/16.00.00_60/ts_123040v160000p.pdf#page=71>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SmsDeliveryReportStatus {
    // Short message transaction completed (0x00-0x1F)
    /// Short message received by the SME successfully
    ReceivedBySme = 0x00,
    /// Short message forwarded by the SC to the SME but delivery confirmation unavailable
    ForwardedButUnconfirmed = 0x01,
    /// Short message replaced by the SC
    ReplacedBySc = 0x02,
    // 0x03-0x0F Reserved
    // 0x10-0x1F SC specific values

    // Temporary error, SC still trying (0x20-0x3F)
    /// Network congestion preventing delivery, SC will retry
    Congestion = 0x20,
    /// SME is busy, SC will retry delivery
    SmeBusy = 0x21,
    /// No response from SME, SC will retry delivery
    NoResponseFromSme = 0x22,
    /// Service rejected by network, SC will retry delivery
    ServiceRejected = 0x23,
    /// Quality of service not available, SC will retry delivery
    QualityOfServiceNotAvailable = 0x24,
    /// Error in SME, SC will retry delivery
    ErrorInSme = 0x25,
    // 0x26-0x2F Reserved
    // 0x30-0x3F SC specific values

    // Permanent error, SC not making more attempts (0x40-0x5F)
    /// Remote procedure error - permanent failure
    RemoteProcedureError = 0x40,
    /// Incompatible destination - permanent failure
    IncompatibleDestination = 0x41,
    /// Connection rejected by SME - permanent failure
    ConnectionRejectedBySme = 0x42,
    /// Destination not obtainable - permanent failure
    NotObtainable = 0x43,
    /// Quality of service not available - permanent failure
    QualityOfServiceNotAvailablePermanent = 0x44,
    /// No interworking available - permanent failure
    NoInterworkingAvailable = 0x45,
    /// SM validity period expired - permanent failure
    SmValidityPeriodExpired = 0x46,
    /// SM deleted by originating SME - permanent failure
    SmDeletedByOriginatingSme = 0x47,
    /// SM deleted by SC administration - permanent failure
    SmDeletedByScAdministration = 0x48,
    /// SM does not exist in SC - permanent failure
    SmDoesNotExist = 0x49,
    // 0x4A-0x4F Reserved
    // 0x50-0x5F SC specific values

    // Temporary error, SC not making more attempts (0x60-0x7F)
    /// Network congestion, SC has stopped retry attempts
    CongestionNoRetry = 0x60,
    /// SME busy, SC has stopped retry attempts
    SmeBusyNoRetry = 0x61,
    /// No response from SME, SC has stopped retry attempts
    NoResponseFromSmeNoRetry = 0x62,
    /// Service rejected, SC has stopped retry attempts
    ServiceRejectedNoRetry = 0x63,
    /// Quality of service not available, SC has stopped retry attempts
    QualityOfServiceNotAvailableNoRetry = 0x64,
    /// Error in SME, SC has stopped retry attempts
    ErrorInSmeNoRetry = 0x65,
    // 0x66-0x69 Reserved
    // 0x6A-0x6F Reserved
    // 0x70-0x7F SC specific values

    /// Unknown or reserved status code - treated as service rejected per spec
    Unknown(u8),
}
impl From<u8> for SmsDeliveryReportStatus {
    fn from(value: u8) -> Self {
        use SmsDeliveryReportStatus::*;

        match value {
            // Transaction completed successfully
            0x00 => ReceivedBySme,
            0x01 => ForwardedButUnconfirmed,
            0x02 => ReplacedBySc,

            // Temporary errors, SC still trying
            0x20 => Congestion,
            0x21 => SmeBusy,
            0x22 => NoResponseFromSme,
            0x23 => ServiceRejected,
            0x24 => QualityOfServiceNotAvailable,
            0x25 => ErrorInSme,

            // Permanent errors
            0x40 => RemoteProcedureError,
            0x41 => IncompatibleDestination,
            0x42 => ConnectionRejectedBySme,
            0x43 => NotObtainable,
            0x44 => QualityOfServiceNotAvailablePermanent,
            0x45 => NoInterworkingAvailable,
            0x46 => SmValidityPeriodExpired,
            0x47 => SmDeletedByOriginatingSme,
            0x48 => SmDeletedByScAdministration,
            0x49 => SmDoesNotExist,

            // Temporary errors, SC not retrying
            0x60 => CongestionNoRetry,
            0x61 => SmeBusyNoRetry,
            0x62 => NoResponseFromSmeNoRetry,
            0x63 => ServiceRejectedNoRetry,
            0x64 => QualityOfServiceNotAvailableNoRetry,
            0x65 => ErrorInSmeNoRetry,

            // All other values (reserved, SC-specific, or unknown)
            _ => Unknown(value)
        }
    }
}
impl SmsDeliveryReportStatus {
    /// Returns true if the SMS was successfully delivered to the SME
    pub fn is_successful(&self) -> bool {
        matches!(self,
            Self::ReceivedBySme |
            Self::ForwardedButUnconfirmed |
            Self::ReplacedBySc
        )
    }

    /// Returns true if this is a temporary error where SC is still trying
    pub fn is_temporary_retrying(&self) -> bool {
        use SmsDeliveryReportStatus::*;

        matches!(self,
            Congestion | SmeBusy | NoResponseFromSme | ServiceRejected |
            QualityOfServiceNotAvailable | ErrorInSme
        ) || matches!(self, Unknown(val) if *val >= 0x20 && *val <= 0x3F)
    }

    /// Returns true if this is a permanent error (no more delivery attempts)
    pub fn is_permanent_error(&self) -> bool {
        use SmsDeliveryReportStatus::*;

        matches!(self,
            RemoteProcedureError | IncompatibleDestination | ConnectionRejectedBySme |
            NotObtainable | QualityOfServiceNotAvailablePermanent | NoInterworkingAvailable |
            SmValidityPeriodExpired | SmDeletedByOriginatingSme | SmDeletedByScAdministration |
            SmDoesNotExist
        ) || matches!(self, Unknown(val) if *val >= 0x40 && *val <= 0x5F)
    }

    /// Returns true if this is a temporary error where SC has stopped trying
    pub fn is_temporary_no_retry(&self) -> bool {
        use SmsDeliveryReportStatus::*;

        matches!(self,
            CongestionNoRetry | SmeBusyNoRetry | NoResponseFromSmeNoRetry |
            ServiceRejectedNoRetry | QualityOfServiceNotAvailableNoRetry | ErrorInSmeNoRetry
        ) || matches!(self, Unknown(val) if *val >= 0x60 && *val <= 0x7F)
    }

    /// Converts the status to a simplified status group for easier categorization
    pub fn to_status_group(&self) -> SmsDeliveryReportStatusGroup {
        if self.is_successful() {
            SmsDeliveryReportStatusGroup::Received
        } else if self.is_temporary_retrying() {
            SmsDeliveryReportStatusGroup::Sent
        } else if self.is_permanent_error() || self.is_temporary_no_retry() {
            // Both permanent errors and temporary errors with no retry are treated as failures
            if self.is_permanent_error() {
                SmsDeliveryReportStatusGroup::PermanentFailure
            } else {
                SmsDeliveryReportStatusGroup::TemporaryFailure
            }
        } else {

            // For unknown status codes, classify based on their range.
            match self {
                Self::Unknown(val) if *val >= 0x20 && *val <= 0x3F => SmsDeliveryReportStatusGroup::Sent,
                Self::Unknown(val) if *val >= 0x40 && *val <= 0x5F => SmsDeliveryReportStatusGroup::PermanentFailure,
                Self::Unknown(val) if *val >= 0x60 && *val <= 0x7F => SmsDeliveryReportStatusGroup::TemporaryFailure,
                _ => SmsDeliveryReportStatusGroup::PermanentFailure // Default for truly unknown codes
            }
        }
    }
}

/// Generalised group for message delivery status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SmsDeliveryReportStatusGroup {

    /// Message was sent but delivery is still pending (temporary errors with retry)
    Sent,
    /// Message was successfully received by the destination.
    Received,
    /// Temporary delivery failure where SC has stopped retrying.
    TemporaryFailure,
    /// Permanent delivery failure - message will not be delivered.
    PermanentFailure
}
impl From<SmsDeliveryReportStatus> for SmsDeliveryReportStatusGroup {
    fn from(status: SmsDeliveryReportStatus) -> Self {
        status.to_status_group()
    }
}

/// Represents the current status of the modem.
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
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
impl std::fmt::Display for ModemStatusUpdateState {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ModemStatusUpdateState::Startup => write!(f, "Startup"),
            ModemStatusUpdateState::Online => write!(f, "Online"),
            ModemStatusUpdateState::ShuttingDown => write!(f, "ShuttingDown"),
            ModemStatusUpdateState::Offline => write!(f, "Offline")
        }
    }
}

/// GNSS (Global Navigation Satellite System) fix status.
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
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
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
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