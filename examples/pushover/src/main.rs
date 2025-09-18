use std::env::var;
use log::{debug, error};
use pushover_rs::{send_pushover_request, MessageBuilder};

use sms_client::Client;
use sms_client::types::SmsStoredMessage;
use sms_client::ws::types::WebsocketMessage;
use sms_client::config::{ClientConfig, WebsocketConfig};
use sms_client::error::ClientError;

#[derive(Clone)]
struct PushoverConfig {
    users: Vec<String>,
    token: String
}

#[derive(Clone)]
struct AppConfig {
    websocket_url: String,
    websocket_auth: Option<String>,
    pushover: PushoverConfig
}
impl AppConfig {
    fn from_env() -> Self {
        AppConfig {
            websocket_url: var("SMS_PUSHOVER_WS_URL").expect("SMS_PUSHOVER_WS_URL not set"),
            websocket_auth: var("SMS_PUSHOVER_WS_AUTH").ok(),
            pushover: PushoverConfig {
                users: var("SMS_PUSHOVER_USERS")
                    .expect("SMS_PUSHOVER_USERS not set")
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect(),

                token: var("SMS_PUSHOVER_TOKEN").expect("SMS_PUSHOVER_TOKEN not set")
            }
        }
    }
}
impl Into<ClientConfig> for AppConfig {
    fn into(self) -> ClientConfig {

        // Filter for only incoming messages.
        let mut config = WebsocketConfig::new(&self.websocket_url)
            .with_filtered_events(Some(vec!["incoming"]));

        // Apply optional authorization.
        if let Some(auth) = &self.websocket_auth {
            config = config.with_auth(auth);
        }

        config.into()
    }
}

async fn send_message(config: PushoverConfig, sms: &SmsStoredMessage) {
    debug!("Got SMS message: {:?}", sms);

    // Create a new message for each target pushover user key.
    for user_key in config.users {
        let builder = MessageBuilder::new(&user_key, &config.token, &sms.message_content)
            .set_title(&format!("SMS from {}", sms.phone_number));

        match send_pushover_request(builder.build()).await {
            Ok(response) if response.status == 1 => { },
            _ => error!("Failed to send pushover request")
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), ClientError> {
    env_logger::init();
    let config = AppConfig::from_env();

    // Listen for incoming messages.
    let client = Client::new(config.clone().into())?;
    client.on_message_simple(move |message| {
        match message {
            WebsocketMessage::IncomingMessage(sms) => {

                // Create a tokio task to send the pushover notifications.
                let pushover_config = config.pushover.clone();
                tokio::spawn(async move {
                    send_message(pushover_config, &sms).await;
                });
            },
            _ => { }
        }
    }).await?;

    // Start the websocket and block since we have nothing else to do.
    client.start_blocking_websocket().await
}
