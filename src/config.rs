//! SMS-Client connection configuration.

/// HTTP-specific configuration.
#[cfg(feature = "http")]
#[derive(Clone, Debug)]
pub struct HttpConfig {

    /// HTTP base URL. eg: http://192.168.1.2:3000
    pub url: String,

    /// Optional HTTP authorization header token.
    pub authorization: Option<String>,

    /// A default timeout to apply to all requests that do not have
    /// their own timeout (this applies to all if modem_timeout is None,
    /// otherwise only database and sys queries).
    pub base_timeout: std::time::Duration,

    /// An optional timeout to use specifically for modem requests
    /// (requests that must send and receive modem data). This should
    /// be higher than the default timeout as they can take longer.
    pub modem_timeout: Option<std::time::Duration>,
}
#[cfg(feature = "http")]
impl HttpConfig {

    /// The default amount of seconds before an HTTP request should time out.
    /// If there is no modem_timeout, this is applied to all requests.
    pub const HTTP_DEFAULT_BASE_TIMEOUT: u64 = 5;

    /// The default amount of seconds before an HTTP request that interacts directly
    /// with the modem should time out. This should be longer to allow for carrier response.
    pub const HTTP_DEFAULT_MODEM_TIMEOUT: u64 = 20;

    /// Create a new HTTP configuration with default settings.
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            authorization: None,
            base_timeout: std::time::Duration::from_secs(Self::HTTP_DEFAULT_BASE_TIMEOUT),
            modem_timeout: Some(std::time::Duration::from_secs(Self::HTTP_DEFAULT_MODEM_TIMEOUT))
        }
    }

    /// Set the authorization token.
    pub fn with_auth(mut self, token: impl Into<String>) -> Self {
        self.authorization = Some(token.into());
        self
    }

    /// Set the base request timeout.
    pub fn with_base_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.base_timeout = timeout;
        self
    }

    /// Set the modem request timeout.
    pub fn with_modem_timeout(mut self, timeout: Option<std::time::Duration>) -> Self {
        self.modem_timeout = timeout;
        self
    }
}
#[cfg(feature = "http")]
impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:3000".to_string(),
            authorization: None,
            base_timeout: std::time::Duration::from_secs(Self::HTTP_DEFAULT_BASE_TIMEOUT),
            modem_timeout: Some(std::time::Duration::from_secs(Self::HTTP_DEFAULT_MODEM_TIMEOUT))
        }
    }
}

/// WebSocket-specific configuration.
#[cfg(feature = "websocket")]
#[derive(Clone, Debug)]
pub struct WebSocketConfig {

    /// Websocket event channel URL. eg: ws://192.168.1.2:3000/ws
    pub url: String,

    /// Optional Websocket authorization header token.
    pub authorization: Option<String>,

    /// Should the websocket connection automatically reconnect if disconnected.
    pub auto_reconnect: bool,

    /// Interval to use between reconnection attempts.
    pub reconnect_interval: std::time::Duration,

    /// The interval between sending websocket pings.
    pub ping_interval: std::time::Duration,

    /// Timeout duration for missing pings.
    pub ping_timeout: std::time::Duration,

    /// Maximum reconnection attempts (None = unlimited).
    pub max_reconnect_attempts: Option<u32>,

    /// Optional set of events that should be listened to. This is added to
    /// the websocket connection URI, and the server filters out events before
    /// sending them. By default, all events are sent when none are selected.
    pub filtered_events: Option<Vec<String>>
}
#[cfg(feature = "websocket")]
impl WebSocketConfig {

    /// The default interval to use between connection attempts.
    /// Sequential attempts use a backoff up to 60 seconds.
    pub const WS_DEFAULT_RECONNECT_INTERVAL: u64 = 5;

    /// The interval between sending ping messages.
    pub const WS_DEFAULT_PING_INTERVAL: u64 = 10;

    /// The duration between the last ping to count as a timeout.
    pub const WS_DEFAULT_PING_TIMEOUT: u64 = 30;

    /// Create a new WebSocket configuration with default settings.
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            authorization: None,
            auto_reconnect: true,
            reconnect_interval: std::time::Duration::from_secs(Self::WS_DEFAULT_RECONNECT_INTERVAL),
            ping_interval: std::time::Duration::from_secs(Self::WS_DEFAULT_PING_INTERVAL),
            ping_timeout: std::time::Duration::from_secs(Self::WS_DEFAULT_PING_TIMEOUT),
            max_reconnect_attempts: None,
            filtered_events: None
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
    pub fn with_reconnect_interval(mut self, interval: std::time::Duration) -> Self {
        self.reconnect_interval = interval;
        self
    }

    /// Set the ping interval.
    pub fn with_ping_interval(mut self, interval: std::time::Duration) -> Self {
        self.ping_interval = interval;
        self
    }

    /// Set the ping timeout.
    pub fn with_ping_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.ping_timeout = timeout;
        self
    }

    /// Set maximum reconnection attempts (None = unlimited).
    pub fn with_max_reconnect_attempts(mut self, max_attempts: Option<u32>) -> Self {
        self.max_reconnect_attempts = max_attempts;
        self
    }

    /// Set filtered listen events, this is included in the connection query-string.
    /// The provided Vec should contain every event name that should be sent by the server.
    /// If None, filtering is disabled so all events are sent.
    pub fn with_filtered_events(mut self, events: Option<Vec<impl Into<String>>>) -> Self {
        self.filtered_events = events.map(|events| events.into_iter().map(Into::into).collect());
        self
    }
}
#[cfg(feature = "websocket")]
impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            url: "ws://localhost:3000/ws".to_string(),
            authorization: None,
            auto_reconnect: true,
            reconnect_interval: std::time::Duration::from_secs(Self::WS_DEFAULT_RECONNECT_INTERVAL),
            ping_interval: std::time::Duration::from_secs(Self::WS_DEFAULT_PING_INTERVAL),
            ping_timeout: std::time::Duration::from_secs(Self::WS_DEFAULT_PING_TIMEOUT),
            max_reconnect_attempts: None,
            filtered_events: None
        }
    }
}

/// WebSocket and HTTP TLS configuration.
#[derive(Clone, Debug)]
pub struct TLSConfig {

    /// TLS certificate filepath.
    pub certificate: std::path::PathBuf
}
impl TLSConfig {

    /// Set a certificate filepath to use for TLS connections.
    pub fn new(certificate: impl Into<std::path::PathBuf>) -> crate::error::ClientResult<Self> {
        Ok(Self {
            certificate: Self::verify_path(certificate.into())?
        })
    }

    /// Verify certificate filepath, that it's a valid filepath and it has an appropriate extension.
    fn verify_path(path: std::path::PathBuf) -> crate::error::ClientResult<std::path::PathBuf> {
        if !path.exists() {
            return Err(crate::error::ClientError::ConfigError("Certificate filepath does not exist"));
        }
        if !path.is_file() {
            return Err(crate::error::ClientError::ConfigError("Certificate filepath is not a file"));
        }
        let canonical_path = path.canonicalize()
            .map_err(|_| { crate::error::ClientError::ConfigError("Invalid certificate path") })?;

        // Check file extension.
        match path.extension().and_then(|s| s.to_str()) {
            Some("pem") | Some("crt") | Some("der") => Ok(canonical_path),
            _ => Err(crate::error::ClientError::ConfigError("Invalid certificate file extension")),
        }
    }
}

/// Complete client configuration.
#[derive(Clone, Debug)]
pub struct ClientConfig {

    /// TLS configuration, used for both HTTP and WebSocket connections.
    pub tls: Option<TLSConfig>,

    /// HTTP configuration.
    #[cfg(feature = "http")]
    pub http: Option<HttpConfig>,

    /// Optional WebSocket configuration.
    #[cfg(feature = "websocket")]
    pub websocket: Option<WebSocketConfig>
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
    #[cfg(feature = "http")]
    pub fn http_only(url: impl Into<String>) -> Self {
        Self {
            tls: None,
            http: Some(HttpConfig::new(url)),

            #[cfg(feature = "websocket")]
            websocket: None
        }
    }

    /// Create a new configuration with only WebSocket support.
    ///
    /// # Example
    /// ```
    /// use sms_client::config::ClientConfig;
    ///
    /// let config = ClientConfig::websocket_only("ws://192.168.1.2:3000/ws");
    /// ```
    #[cfg(feature = "websocket")]
    pub fn websocket_only(ws_url: impl Into<String>) -> Self {
        Self {
            tls: None,

            #[cfg(feature = "http")]
            http: None,

            websocket: Some(WebSocketConfig::new(ws_url))
        }
    }

    /// Create a new configuration with both HTTP and WebSocket support.
    ///
    /// # Example
    /// ```
    /// use sms_client::config::ClientConfig;
    ///
    /// let config = ClientConfig::both(
    ///     "http://192.168.1.2:3000",
    ///     "ws://192.168.1.2:3000/ws"
    /// );
    /// ```
    #[cfg(feature = "http")]
    #[cfg(feature = "websocket")]
    pub fn both(http_url: impl Into<String>, ws_url: impl Into<String>) -> Self {
        Self {
            tls: None,
            http: Some(HttpConfig::new(http_url)),
            websocket: Some(WebSocketConfig::new(ws_url))
        }
    }

    /// Create a configuration from individual HTTP and WebSocket configs.
    ///
    /// # Example
    /// ```
    /// use std::time::Duration;
    /// use sms_client::config::{ClientConfig, HttpConfig, WebSocketConfig};
    ///
    /// let http = HttpConfig::new("http://192.168.1.2:3000")
    ///     .with_auth("token123")
    ///     .with_base_timeout(Duration::from_secs(30));
    ///
    /// let ws = WebSocketConfig::new("ws://192.168.1.2:3000/ws")
    ///     .with_auth("token123")
    ///     .with_auto_reconnect(true)
    ///     .with_max_reconnect_attempts(Some(10));
    ///
    /// let config = ClientConfig::from_parts(Some(http), Some(ws));
    /// ```
    #[cfg(feature = "http")]
    #[cfg(feature = "websocket")]
    pub fn from_parts(http: Option<HttpConfig>, websocket: Option<WebSocketConfig>) -> Self {
        Self { tls: None, http, websocket }
    }

    /// Add TLS configuration.
    pub fn add_tls(mut self, tls: TLSConfig) -> Self {
        self.tls = Some(tls);
        self
    }

    /// Set authorization for both HTTP and WebSocket.
    /// This only sets the authorization value for components that already exist.
    ///
    /// # Example
    /// ```rust
    /// use sms_client::config::ClientConfig;
    ///
    /// let config = ClientConfig::both(
    ///     "http://192.168.1.2:3000",
    ///     "ws://192.168.1.2:3000/ws"
    /// ).with_auth("my-token");
    /// ```
    pub fn with_auth(mut self, token: impl Into<String>) -> Self {
        let token = token.into();

        #[cfg(feature = "http")]
        if let Some(http) = &mut self.http {
            http.authorization = Some(token.clone());
        }

        #[cfg(feature = "websocket")]
        if let Some(ws) = &mut self.websocket {
            ws.authorization = Some(token);
        }
        self
    }

    /// Modify/Set a TLSConfig with certificate filepath.
    ///
    /// # Example
    /// ```rust
    /// use sms_client::config::ClientConfig;
    ///
    /// let config = ClientConfig::http_only("https://192.168.1.2:3000")
    ///     .with_certificate("./certificate.crt")?;
    pub fn with_certificate(mut self, certificate: impl Into<std::path::PathBuf>) -> crate::error::ClientResult<Self> {
        if let Some(tls) = &mut self.tls {
            tls.certificate = TLSConfig::verify_path(certificate.into())?;
        } else {
            self.tls = Some(TLSConfig::new(certificate)?);
        }
        Ok(self)
    }

    /// Configure the HTTP component if present.
    ///
    /// # Example
    /// ```
    /// use sms_client::config::ClientConfig;
    /// use std::time::Duration;
    ///
    /// let config = ClientConfig::http_only("http://192.168.1.2:3000")
    ///     .configure_http(|http| {
    ///         http.with_base_timeout(Duration::from_secs(30))
    ///             .with_auth("token")
    ///     });
    /// ```
    #[cfg(feature = "http")]
    pub fn configure_http<F>(mut self, f: F) -> Self
    where
        F: FnOnce(HttpConfig) -> HttpConfig,
    {
        if let Some(http) = self.http {
            self.http = Some(f(http));
        }
        self
    }

    /// Configure the WebSocket component if present.
    ///
    /// # Example
    /// ```
    /// use sms_client::config::ClientConfig;
    /// use std::time::Duration;
    ///
    /// let config = ClientConfig::both(
    ///     "http://192.168.1.2:3000",
    ///     "ws://192.168.1.2:3000/ws"
    /// ).configure_websocket(|ws| {
    ///     ws.with_ping_interval(Duration::from_secs(60))
    ///       .with_max_reconnect_attempts(Some(5))
    /// });
    /// ```
    #[cfg(feature = "websocket")]
    pub fn configure_websocket<F>(mut self, f: F) -> Self
    where
        F: FnOnce(WebSocketConfig) -> WebSocketConfig,
    {
        if let Some(ws) = self.websocket {
            self.websocket = Some(f(ws));
        }
        self
    }

    /// Add WebSocket configuration.
    ///
    /// # Example
    /// ```
    /// use sms_client::config::{ClientConfig, WebSocketConfig};
    ///
    /// let config = ClientConfig::http_only("http://192.168.1.2:3000")
    ///     .add_websocket(WebSocketConfig::new("ws://192.168.1.2:3000/ws"));
    /// ```
    #[cfg(feature = "websocket")]
    pub fn add_websocket(mut self, websocket: WebSocketConfig) -> Self {
        self.websocket = Some(websocket);
        self
    }
}
impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            tls: None,

            #[cfg(feature = "http")]
            http: Some(HttpConfig::default()),

            #[cfg(feature = "websocket")]
            websocket: Some(WebSocketConfig::default())
        }
    }
}

#[cfg(feature = "http")]
impl From<HttpConfig> for ClientConfig {
    fn from(http: HttpConfig) -> Self {
        ClientConfig { tls: None, http: Some(http), ..Default::default() }
    }
}

#[cfg(feature = "websocket")]
impl From<WebSocketConfig> for ClientConfig {
    fn from(ws: WebSocketConfig) -> Self {
        ClientConfig { tls: None, websocket: Some(ws), ..Default::default() }
    }
}