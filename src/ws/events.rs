//! WebSocket event kinds (server or client generated).

/// An event passed to on_message (or variant). This represents an event
/// that could either be from the SMS server or the client (reconnection).
#[derive(Debug, Clone, PartialEq)]
pub enum WebsocketEvent {
    /// An event received from the server.
    Server(sms_types::events::Event),

    /// A reconnection event created by client worker.
    Reconnection(WebsocketReconnectionKind),
}

/// A client generated event for Websocket connection state updates.
#[derive(Debug, Clone, PartialEq)]
pub enum WebsocketReconnectionKind {
    /// The websocket is now connected.
    Connected,

    /// The websocket is disconnected, the contained value is if
    /// automatic reconnection will be attempted.
    Disconnected(bool),
}
