# SMS Client

A remote client library for interfacing with the [SMS-API](https://github.com/morgverd/sms-api),
making it easy to send and receive SMS messages from Rust, **all with your own hardware and no API subscriptions!**

This also includes bindings to the SMS database, useful for retrieving previous messages and delivery states.

## Installation

Simply use `cargo add sms-client`

Here's some other usage examples from inside a project `Cargo.toml`.

```toml
[dependencies]

# Includes ONLY the HttpClient.
sms-client = "1.5.3"

# Includes BOTH the HttpClient and WebSocketClient.
sms-client = { version = "1.5.3", features = ["websocket"] }

# Includes ONLY the WebSocketClient.
sms-client = { version = "1.5.3", default-features = false, features = ["websocket"] }

# Includes BOTH, with Rust-TLS.
sms-client = { version = "1.5.3", features = ["http-tls-rustls", "websocket-tls-rustls"] }

# Includes BOTH, with native TLS.
sms-client = { version = "1.5.3", features = ["http-tls-native", "websocket-tls-native"] }
```

## Compilation Features

> When enabling a TLS feature (eg: `websocket-tls-native`) the base feature (`websocket`) is also enabled.

| Feature Name         | Description                                         | Default |
|----------------------|-----------------------------------------------------|---------|
| http                 | Enables HttpClient to send commands to API.         | Yes     |
| websocket            | Enables WebSocketClient to receive events from API. | No      |
| http-tls-rustls      | Uses Rust-TLS for reqwest HTTP client.              | Yes     |
| http-tls-native      | Uses default TLS for reqwest HTTP client.           | No      |
| websocket-tls-rustls | Uses Rust-TLS for WebSocket client.                 | No      |
| websocket-tls-native | Uses default TLS for WebSocket client.              | No      |

## Example Projects

Here are two example projects that use this crate:
- [Pushover](/examples/pushover) - Send Pushover notifications for Incoming messages.
- [SMS-Terminal](https://github.com/morgverd/sms-terminal) - Send and receive SMS messages via a TUI application.

## Example Code

This is an example that listens for incoming SMS messages, and then replies with the same message content.

```rust
use std::sync::Arc;

use sms_client::types::SmsStoredMessage;
use sms_client::ws::types::WebsocketMessage;
use sms_client::http::types::HttpOutgoingSmsMessage;
use sms_client::config::ClientConfig;
use sms_client::error::ClientResult;
use sms_client::Client;

#[tokio::main]
async fn main() -> ClientResult<()> {
    let config = ClientConfig::both(
        "https://localhost:3000", // HTTP base uri 
        "wss://localhost:3000/ws" // WebSocket base uri
    )

        // Created WebSocket and HTTP config can be modified during build.
        .configure_websocket(|ws| ws.with_auto_reconnect(false))

        // Add TLS configuration with a certificate to use for all connections.
        .add_tls(
            TLSConfig::new("./certificate.crt")?
        )

        // Sets Authorization header for all connections.
        .with_auth("test!");

    // Create main SMS client, and set WebSocket message callback.
    let client = Client::new(config)?;
    client.on_message(move |message, client| {
        
        // Match WebSocket message to check if it's an IncomingMessage.
        match message {
            WebsocketMessage::IncomingMessage(sms) => send_reply(client, sms),
            _ => { }
        }
    }).await?;

    // Start the websocket loop as blocking. This means the app will halt here
    // until the connection is closed. Alternatively, start_background_websocket()
    // can be used to start the loop in another task.
    client.start_blocking_websocket().await
}

fn send_reply(client: Arc<Client>, message: SmsStoredMessage) {

    // The HttpClient is a Result since it may not have been loaded if disabled by config.
    // In this example though, we know the client will be present.
    let Some(http) = client.http().ok() else {
        return None;
    };

    // Create a reply message with the sender as recipient and same message content.
    let reply = HttpOutgoingSmsMessage::simple_message(
        message.phone_number,
        message.message_content
    );
    tokio::spawn(async move {
        // Ignore result, in reality this should certainly be handled.
        let _ = http.send_sms(&reply).await;
    });
}
```