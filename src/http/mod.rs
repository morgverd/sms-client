//! SMS-API HTTP client.
//! This can be used to interface with the HTTP API standalone if required.

use crate::http::types::*;
use crate::http::error::*;

pub mod types;
pub mod error;
pub mod paginator;

/// Take a response from the client, verify that the status code is 200,
/// then read JSON body and ensure success is true and finally return response value.
async fn read_http_response<T>(response: reqwest::Response) -> HttpResult<T>
where
    T: serde::de::DeserializeOwned
{
    // Verify response status code.
    let status = response.status();
    if !status.is_success() {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error!".to_string());
        return Err(HttpError::HttpStatus {
            status: status.as_u16(),
            message: error_text
        })
    }

    // Verify JSON success status.
    let json: serde_json::Value = response.json().await?;
    let success = json.get("success")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if !success {
        let message = json.get("error_message")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown API error!")
            .to_string();
        return Err(HttpError::ApiError { message })
    }

    // Read response field and make into expected value.
    let response_value = json.get("response")
        .ok_or(HttpError::MissingResponseField)?;

    serde_json::from_value(response_value.clone())
        .map_err(HttpError::JsonError)
}

/// Read a modem-specific response that contains a "type" field and "data" field.
/// Verifies the type matches the expected type before returning the data.
async fn read_modem_response<T>(expected: &str, response: reqwest::Response) -> HttpResult<T>
where
    T: serde::de::DeserializeOwned
{

    // Verify expected response type.
    let json_response: serde_json::Value = read_http_response(response).await?;
    let actual = json_response.get("type")
        .and_then(|v| v.as_str())
        .ok_or(HttpError::MissingTypeField)?;

    if actual != expected {
        return Err(HttpError::ResponseTypeMismatch {
            expected: expected.to_string(),
            actual: actual.to_string()
        });
    }

    // Extract and return the data field.
    let data = json_response.get("data")
        .ok_or(HttpError::MissingDataField)?;

    serde_json::from_value(data.clone())
        .map_err(HttpError::JsonError)
}

/// SMS-API HTTP interface client.
pub struct HttpClient {
    base_url: reqwest::Url,
    authorization: Option<String>,
    modem_timeout: Option<std::time::Duration>,
    client: reqwest::Client
}
impl HttpClient {

    /// Create a new HTTP client that uses the base_url.
    pub fn new(config: crate::config::HttpConfig) -> HttpResult<Self> {
        let client = reqwest::Client::builder()
            .timeout(config.base_timeout)
            .build()?;

        Ok(Self {
            base_url: reqwest::Url::parse(config.url.as_str())?,
            authorization: config.authorization.map(|a| a.into()),
            modem_timeout: config.modem_timeout,
            client
        })
    }

    /// Get messages sent to and from a given phone number.
    /// Pagination options are supported.
    pub async fn get_messages(&self, phone_number: impl Into<String>, pagination: Option<HttpPaginationOptions>) -> HttpResult<Vec<crate::types::SmsStoredMessage>> {
        let mut body = serde_json::json!({
            "phone_number": phone_number.into()
        });
        if let Some(pagination) = pagination {
            pagination.add_to_body(&mut body);
        }

        let url = self.base_url.join("/db/sms")?;
        let response = self.setup_request(false, self.client.post(url))
            .json(&body)
            .send()
            .await?;

        read_http_response(response).await
    }

    /// Get the latest phone numbers that have been in contact with the SMS-API.
    /// This includes both senders and receivers. Pagination options are supported.
    pub async fn get_latest_numbers(&self, pagination: Option<HttpPaginationOptions>) -> HttpResult<Vec<String>> {
        let url = self.base_url.join("/db/latest-numbers")?;
        let mut request = self.setup_request(false, self.client.post(url));

        // Only add a JSON body if there are pagination options.
        if let Some(pagination) = pagination {
            request = request.json(&pagination);
        }

        let response = request.send().await?;
        read_http_response(response).await
    }

    /// Get received delivery reports for a given message_id (comes from send_sms etc).
    /// Pagination options are supported.
    pub async fn get_delivery_reports(&self, message_id: i64, pagination: Option<HttpPaginationOptions>) -> HttpResult<Vec<HttpSmsDeliveryReport>> {
        let mut body = serde_json::json!({
            "message_id": message_id
        });
        if let Some(pagination) = pagination {
            pagination.add_to_body(&mut body);
        }

        let url = self.base_url.join("/db/delivery-reports")?;
        let response = self.setup_request(false, self.client.post(url))
            .json(&body)
            .send()
            .await?;

        read_http_response(response).await
    }

    /// Send an SMS message to a target phone_number. The result will contain the
    /// message reference (provided from modem) and message id (used internally).
    pub async fn send_sms(&self, message: HttpOutgoingSmsMessage) -> HttpResult<HttpSmsSendResponse> {
        let url = self.base_url.join("/sms/send")?;
        let response = self.setup_request(true, self.client.post(url))
            .json(&message)
            .send()
            .await?;

        read_http_response(response).await
    }

    /// Get the carrier network status.
    pub async fn get_network_status(&self) -> HttpResult<HttpModemNetworkStatusResponse> {
        self.modem_request("modem-status", "NetworkStatus").await
    }

    /// Get the modem signal strength for the connected tower.
    pub async fn get_signal_strength(&self) -> HttpResult<HttpModemSignalStrengthResponse> {
        self.modem_request("signal-strength", "SignalStrength").await
    }

    /// Get the underlying network operator, this is often the same across
    /// multiple service providers for a given region. Eg: vodafone.
    pub async fn get_network_operator(&self) -> HttpResult<HttpModemNetworkOperatorResponse> {
        self.modem_request("network-operator", "NetworkOperator").await
    }

    /// Get the SIM service provider, this is the brand that manages the contract.
    /// This matters less than the network operator, as they're just resellers. Eg: ASDA Mobile.
    pub async fn get_service_provider(&self) -> HttpResult<String> {
        self.modem_request("service-provider", "ServiceProvider").await
    }

    /// Get the Modem Hat's battery level, which is used for GNSS warm starts.
    pub async fn get_battery_level(&self) -> HttpResult<HttpModemBatteryLevelResponse> {
        self.modem_request("battery-level", "BatteryLevel").await
    }

    /// Get the configured sender SMS number. This should be used primarily for client identification.
    /// This is optional, as the API could have left this un-configured without any value set.
    pub async fn get_phone_number(&self) -> HttpResult<Option<String>> {
        let url = self.base_url.join("/sys/phone-number")?;
        let response = self.setup_request(false, self.client.get(url))
            .send()
            .await?;

        read_http_response(response).await
    }

    /// Get the modem SMS-API version string. This will be a semver format,
    /// often with feature names added as a suffix, eg: "0.0.1+sentry".
    pub async fn get_version(&self) -> HttpResult<String> {
        let url = self.base_url.join("/sys/version")?;
        let response = self.setup_request(false, self.client.get(url))
            .send()
            .await?;

        read_http_response(response).await
    }

    /// Send an SMS modem request, the response contains a named type which is verified.
    async fn modem_request<T>(&self, route: &str, expected: &str) -> HttpResult<T>
    where
        T: serde::de::DeserializeOwned
    {
        let url = self.base_url.join(&format!("/sms/{route}"))?;
        let response = self.setup_request(true, self.client.get(url))
            .send()
            .await?;

        read_modem_response::<T>(expected, response).await
    }

    /// Allow for a different timeout to be used for modem requests,
    /// and apply optional authorization header to request builder.
    fn setup_request(&self, is_modem: bool, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        let builder = if is_modem && let Some(timeout) = &self.modem_timeout {
            builder.timeout(*timeout)
        } else {
            builder
        };
        if let Some(auth) = &self.authorization {
            builder.header("authorization", auth)
        } else {
            builder
        }
    }
}