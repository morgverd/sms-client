//! SMS-Client connection configuration.

use std::time::Duration;

/// HTTP-specific configuration.
#[derive(Clone, Debug)]
pub struct HttpConfig {
    /// HTTP base URL. eg: http://192.168.1.2:3000
    pub url: String,

    /// Optional HTTP authorization header token.
    pub authorization: Option<String>,

    /// Request timeout duration.
    pub timeout: Duration,
}
impl HttpConfig {
    /// Create a new HTTP configuration with default settings.
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            authorization: None,
            timeout: Duration::from_secs(10),
        }
    }

    /// Set the authorization token.
    pub fn with_auth(mut self, token: impl Into<String>) -> Self {
        self.authorization = Some(token.into());
        self
    }

    /// Set the request timeout.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}
impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:3000".to_string(),
            authorization: None,
            timeout: Duration::from_secs(10),
        }
    }
}

/// WebSocket-specific configuration.
#[derive(Clone, Debug)]
pub struct WebsocketConfig {
    /// Websocket event channel URL. eg: ws://192.168.1.2:3000/ws
    pub url: String,

    /// Optional Websocket authorization header token.
    pub authorization: Option<String>,

    /// Should the websocket connection automatically reconnect if disconnected.
    pub auto_reconnect: bool,

    /// Interval to use between reconnection attempts.
    pub reconnect_interval: Duration,

    /// The interval between sending websocket pings.
    pub ping_interval: Duration,

    /// Timeout duration for missing pings.
    pub ping_timeout: Duration,

    /// Maximum reconnection attempts (None = unlimited).
    pub max_reconnect_attempts: Option<u32>,
}
impl WebsocketConfig {
    /// Create a new WebSocket configuration with default settings.
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            authorization: None,
            auto_reconnect: true,
            reconnect_interval: Duration::from_secs(5),
            ping_interval: Duration::from_secs(30),
            ping_timeout: Duration::from_secs(10),
            max_reconnect_attempts: None,
        }
    }

    /// Set the authorization token.
    pub fn with_auth(mut self, token: impl Into<String>) -> Self {
        self.authorization = Some(token.into());
        self
    }

    /// Enable or disable auto-reconnection.
    pub fn with_auto_reconnect(mut self, enabled: bool) -> Self {
        self.auto_reconnect = enabled;
        self
    }

    /// Set the reconnection interval.
    pub fn with_reconnect_interval(mut self, interval: Duration) -> Self {
        self.reconnect_interval = interval;
        self
    }

    /// Set the ping interval.
    pub fn with_ping_interval(mut self, interval: Duration) -> Self {
        self.ping_interval = interval;
        self
    }

    /// Set the ping timeout.
    pub fn with_ping_timeout(mut self, timeout: Duration) -> Self {
        self.ping_timeout = timeout;
        self
    }

    /// Set maximum reconnection attempts (None = unlimited).
    pub fn with_max_reconnect_attempts(mut self, max_attempts: Option<u32>) -> Self {
        self.max_reconnect_attempts = max_attempts;
        self
    }
}
impl Default for WebsocketConfig {
    fn default() -> Self {
        Self {
            url: "ws://localhost:3000/ws".to_string(),
            authorization: None,
            auto_reconnect: true,
            reconnect_interval: Duration::from_secs(5),
            ping_interval: Duration::from_secs(30),
            ping_timeout: Duration::from_secs(10),
            max_reconnect_attempts: None,
        }
    }
}

/// Complete client configuration.
#[derive(Clone, Debug)]
pub struct ClientConfig {
    /// HTTP configuration.
    pub http: HttpConfig,

    /// Optional WebSocket configuration.
    pub websocket: Option<WebsocketConfig>,
}
impl ClientConfig {
    /// Create a new configuration with only HTTP support.
    ///
    /// # Example
    /// ```
    /// use sms_client::config::ClientConfig;
    ///
    /// let config = ClientConfig::http_only("http://192.168.1.2:3000");
    /// ```
    pub fn http_only(url: impl Into<String>) -> Self {
        Self {
            http: HttpConfig::new(url),
            websocket: None,
        }
    }

    /// Create a new configuration with both HTTP and WebSocket support.
    ///
    /// # Example
    /// ```
    /// use sms_client::config::ClientConfig;
    ///
    /// let config = ClientConfig::with_websocket(
    ///     "http://192.168.1.2:3000",
    ///     "ws://192.168.1.2:3000/ws"
    /// );
    /// ```
    pub fn with_websocket(http_url: impl Into<String>, ws_url: impl Into<String>) -> Self {
        Self {
            http: HttpConfig::new(http_url),
            websocket: Some(WebsocketConfig::new(ws_url)),
        }
    }

    /// Create a configuration from individual HTTP and WebSocket configs.
    ///
    /// # Example
    /// ```
    /// use std::time::Duration;
    /// use sms_client::config::{ClientConfig, HttpConfig, WebsocketConfig};
    ///
    /// let http = HttpConfig::new("http://192.168.1.2:3000")
    ///     .with_auth("token123")
    ///     .with_timeout(Duration::from_secs(30));
    ///
    /// let ws = WebsocketConfig::new("ws://192.168.1.2:3000/ws")
    ///     .with_auth("token123")
    ///     .with_auto_reconnect(true)
    ///     .with_max_reconnect_attempts(Some(10));
    ///
    /// let config = ClientConfig::from_parts(http, Some(ws));
    /// ```
    pub fn from_parts(http: HttpConfig, websocket: Option<WebsocketConfig>) -> Self {
        Self { http, websocket }
    }

    /// Set authorization for both HTTP and WebSocket.
    ///
    /// # Example
    /// ```
    /// use sms_client::config::ClientConfig;
    ///
    /// let config = ClientConfig::with_websocket(
    ///     "http://192.168.1.2:3000",
    ///     "ws://192.168.1.2:3000/ws"
    /// ).with_auth("my-token");
    /// ```
    pub fn with_auth(mut self, token: impl Into<String>) -> Self {
        let token = token.into();
        self.http.authorization = Some(token.clone());
        if let Some(ws) = &mut self.websocket {
            ws.authorization = Some(token);
        }
        self
    }

    /// Configure the HTTP component.
    ///
    /// # Example
    /// ```
    /// use sms_client::config::ClientConfig;
    /// use std::time::Duration;
    ///
    /// let config = ClientConfig::http_only("http://192.168.1.2:3000")
    ///     .configure_http(|http| {
    ///         http.with_timeout(Duration::from_secs(30))
    ///             .with_auth("token")
    ///     });
    /// ```
    pub fn configure_http<F>(mut self, f: F) -> Self
    where
        F: FnOnce(HttpConfig) -> HttpConfig,
    {
        self.http = f(self.http);
        self
    }

    /// Configure the WebSocket component if present.
    ///
    /// # Example
    /// ```
    /// use sms_client::config::ClientConfig;
    /// use std::time::Duration;
    ///
    /// let config = ClientConfig::with_websocket(
    ///     "http://192.168.1.2:3000",
    ///     "ws://192.168.1.2:3000/ws"
    /// ).configure_websocket(|ws| {
    ///     ws.with_ping_interval(Duration::from_secs(60))
    ///       .with_max_reconnect_attempts(Some(5))
    /// });
    /// ```
    pub fn configure_websocket<F>(mut self, f: F) -> Self
    where
        F: FnOnce(WebsocketConfig) -> WebsocketConfig,
    {
        if let Some(ws) = self.websocket {
            self.websocket = Some(f(ws));
        }
        self
    }

    /// Add WebSocket support to an HTTP-only configuration.
    ///
    /// # Example
    /// ```
    /// use sms_client::config::{ClientConfig, WebsocketConfig};
    ///
    /// let config = ClientConfig::http_only("http://192.168.1.2:3000")
    ///     .add_websocket(WebsocketConfig::new("ws://192.168.1.2:3000/ws"));
    /// ```
    pub fn add_websocket(mut self, websocket: WebsocketConfig) -> Self {
        self.websocket = Some(websocket);
        self
    }

    /// Remove WebSocket support from the configuration.
    pub fn without_websocket(mut self) -> Self {
        self.websocket = None;
        self
    }
}
impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            http: HttpConfig::default(),
            websocket: Some(WebsocketConfig::default()),
        }
    }
}

/// Builder for creating a client with a fluent API.
///
/// # Example
/// ```
/// use sms_client::config::ConfigBuilder;
///
/// let config = ConfigBuilder::new()
///     .http_url("http://192.168.1.2:3000")
///     .websocket_url("ws://192.168.1.2:3000/ws")
///     .auth_token("my-token")
///     .auto_reconnect(true)
///     .build();
/// ```
pub struct ConfigBuilder {
    http_url: Option<String>,
    ws_url: Option<String>,
    auth_token: Option<String>,
    http_timeout: Duration,
    auto_reconnect: bool,
    reconnect_interval: Duration,
    ping_interval: Duration,
    ping_timeout: Duration,
    max_reconnect_attempts: Option<u32>,
}
impl ConfigBuilder {
    /// Create a new builder with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the HTTP URL.
    pub fn http_url(mut self, url: impl Into<String>) -> Self {
        self.http_url = Some(url.into());
        self
    }

    /// Set the WebSocket URL.
    pub fn websocket_url(mut self, url: impl Into<String>) -> Self {
        self.ws_url = Some(url.into());
        self
    }

    /// Set the authorization token for both HTTP and WebSocket.
    pub fn auth_token(mut self, token: impl Into<String>) -> Self {
        self.auth_token = Some(token.into());
        self
    }

    /// Set the HTTP request timeout.
    pub fn http_timeout(mut self, timeout: Duration) -> Self {
        self.http_timeout = timeout;
        self
    }

    /// Enable or disable WebSocket auto-reconnection.
    pub fn auto_reconnect(mut self, enabled: bool) -> Self {
        self.auto_reconnect = enabled;
        self
    }

    /// Set the WebSocket reconnection interval.
    pub fn reconnect_interval(mut self, interval: Duration) -> Self {
        self.reconnect_interval = interval;
        self
    }

    /// Set the WebSocket ping interval.
    pub fn ping_interval(mut self, interval: Duration) -> Self {
        self.ping_interval = interval;
        self
    }

    /// Set the WebSocket ping timeout.
    pub fn ping_timeout(mut self, timeout: Duration) -> Self {
        self.ping_timeout = timeout;
        self
    }

    /// Set maximum WebSocket reconnection attempts.
    pub fn max_reconnect_attempts(mut self, max: Option<u32>) -> Self {
        self.max_reconnect_attempts = max;
        self
    }

    /// Build the final ClientConfig.
    pub fn build(self) -> ClientConfig {
        let http_url = self.http_url.unwrap_or_else(|| "http://127.0.0.1:3000".to_string());

        let mut http = HttpConfig::new(http_url)
            .with_timeout(self.http_timeout);

        if let Some(token) = &self.auth_token {
            http = http.with_auth(token.clone());
        }

        let websocket = self.ws_url.map(|url| {
            let mut ws = WebsocketConfig::new(url)
                .with_auto_reconnect(self.auto_reconnect)
                .with_reconnect_interval(self.reconnect_interval)
                .with_ping_interval(self.ping_interval)
                .with_ping_timeout(self.ping_timeout)
                .with_max_reconnect_attempts(self.max_reconnect_attempts);

            if let Some(token) = &self.auth_token {
                ws = ws.with_auth(token.clone());
            }

            ws
        });

        ClientConfig::from_parts(http, websocket)
    }
}
impl Default for ConfigBuilder {
    fn default() -> Self {
        Self {
            http_url: None,
            ws_url: None,
            auth_token: None,
            http_timeout: Duration::from_secs(10),
            auto_reconnect: true,
            reconnect_interval: Duration::from_secs(5),
            ping_interval: Duration::from_secs(30),
            ping_timeout: Duration::from_secs(10),
            max_reconnect_attempts: None,
        }
    }
}