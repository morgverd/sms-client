//! WebSocket connection establishment and management.

use crate::ws::error::*;
use tungstenite::client::IntoClientRequest;

/// Connection parameters for WebSocket
pub struct ConnectionParams {
    pub url: String,
    pub authorization: Option<http::HeaderValue>,
    pub connector: Option<tokio_tungstenite::Connector>,
}
impl ConnectionParams {
    /// Create connection parameters from configuration
    pub fn from_config(
        config: &crate::config::WebSocketConfig,
        tls_config: Option<&crate::config::TLSConfig>,
    ) -> WebsocketResult<Self> {
        // Parse URL and add event filters if present
        let mut url = config
            .url
            .parse::<url::Url>()
            .map_err(|e| WebsocketError::UrlError(e.into()))?;

        if let Some(filter) = &config.filtered_events {
            url.query_pairs_mut()
                .append_pair("events", filter.join(",").as_str());
        }

        // Parse authorization header if present
        let authorization = config
            .authorization
            .as_ref()
            .map(|auth| auth.parse::<http::HeaderValue>())
            .transpose()?;

        // Create TLS connector if configured
        let connector = crate::ws::tls::create_connector(tls_config)?;
        Ok(Self {
            url: url.to_string(),
            authorization,
            connector,
        })
    }

    /// Establish WebSocket connection
    pub async fn connect(
        &self,
    ) -> WebsocketResult<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
    > {
        log::debug!("Connecting to WebSocket: {}", self.url);

        // Create request with optional authorization
        let mut request = self.url.as_str().into_client_request()?;
        if let Some(token) = &self.authorization {
            request.headers_mut().append("authorization", token.clone());
        }

        // Connect with optional TLS support
        let ws_stream = match &self.connector {
            #[cfg(any(feature = "websocket-tls-rustls", feature = "websocket-tls-native"))]
            Some(connector) => {
                match tokio_tungstenite::connect_async_tls_with_config(
                    request,
                    None,
                    false,
                    Some(connector.clone()),
                )
                .await
                {
                    Ok((stream, _)) => stream,
                    Err(e) => return Self::handle_connection_error(e),
                }
            }

            #[cfg(not(any(feature = "websocket-tls-rustls", feature = "websocket-tls-native")))]
            Some(_) => {
                return Err(WebsocketError::TLSError(
                    "TLS connector provided but no TLS features enabled".to_string(),
                ));
            }

            None => match tokio_tungstenite::connect_async(request).await {
                Ok((stream, _)) => stream,
                Err(e) => return Self::handle_connection_error(e),
            },
        };

        log::debug!("WebSocket connected successfully");
        Ok(ws_stream)
    }

    /// Handle connection errors, checking for authorization failures
    fn handle_connection_error(
        error: tungstenite::Error,
    ) -> WebsocketResult<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
    > {
        if let tungstenite::Error::Http(response) = &error {
            if response.status() == http::StatusCode::UNAUTHORIZED {
                return Err(WebsocketError::Unauthorized);
            }
        }
        Err(error.into())
    }
}
