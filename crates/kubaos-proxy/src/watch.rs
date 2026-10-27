//! What the session service does when the machine wakes or the network changes: logind's resume,
//! and NetworkManager's active connections, which change when the tunnel drops while the machine
//! is awake.
//!
//! # Owns
//!
//! - [`Event`], [`Response`] and [`respond`], the choice between them.
//!
//! # Not here
//!
//! - Following the bus and reconciling: `main.rs` and [`crate::apply::reconcile`].

use std::time::Duration;

/// How long after a failed reconnect a change of network doesn't try again: NetworkManager's own
/// events from the attempt would otherwise start the next one, without end.
pub const RETRY_AFTER: Duration = Duration::from_secs(30);

/// What happened.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    /// The machine woke from sleep.
    Resumed,
    /// NetworkManager's active connections changed: a network came or went, or the tunnel.
    NetworkChanged,
}

/// What the service does about it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Response {
    /// Tell the shell what is up now.
    Publish,
    /// Make the machine what the user wants, the tunnel back up among it, then tell the shell.
    Reconcile,
}

/// The response to `event`, while the user wants the Proxy on or not, `since_failure` after the
/// last reconnect that failed.
pub fn respond(event: Event, wanted: bool, since_failure: Option<Duration>) -> Response {
    if !wanted {
        return Response::Publish;
    }
    match (event, since_failure) {
        (Event::NetworkChanged, Some(since)) if since < RETRY_AFTER => Response::Publish,
        _ => Response::Reconcile,
    }
}

#[cfg(test)]
mod tests;
