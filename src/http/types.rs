use serde::{Serialize, Deserialize};

#[derive(Serialize, Default, Debug)]
pub struct HttpPaginationOptions {
    pub limit: Option<u64>,
    pub offset: Option<u64>,
    pub reverse: Option<bool>
}
impl HttpPaginationOptions {
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

#[derive(Deserialize, Debug)]
pub struct HttpSmsStoredMessage {
    pub message_id: i64,
    pub phone_number: String,
    pub message_content: String,
    pub message_reference: Option<u8>,
    pub is_outgoing: bool,
    pub status: String,
    pub created_at: u32,
    pub completed_at: Option<u32>
}

#[derive(Deserialize, Debug)]
pub struct HttpSmsDeliveryReport {
    pub report_id: i64,
    pub status: u8,
    pub is_final: bool,
    pub created_at: u32
}

#[derive(Deserialize, Debug)]
pub struct HttpSmsSendResponse {
    pub message_id: i64,
    pub reference_id: u8
}

#[derive(Deserialize, Debug)]
pub struct HttpModemNetworkStatusResponse {
    pub registration: u8,
    pub technology: u8
}

#[derive(Deserialize, Debug)]
pub struct HttpModemSignalStrengthResponse {
    pub rssi: u8,
    pub ber: u8
}

#[derive(Deserialize, Debug)]
pub struct HttpModemNetworkOperatorResponse {
    pub status: u8,
    pub format: u8,
    pub operator: String
}

#[derive(Deserialize, Debug)]
pub struct HttpModemBatteryLevelResponse {
    pub status: u8,
    pub charge: u8,
    pub voltage: f32
}