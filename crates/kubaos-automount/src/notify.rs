//! Desktop notifications, through `org.freedesktop.Notifications` on the session bus, which the
//! shell serves with buttons for their actions.
//!
//! The interface: <https://specifications.freedesktop.org/notification-spec/latest/>
//!
//! # Owns
//!
//! - [`Note`], what a notification says and offers, and [`Notifier`], which sends it.
//! - Reading `ActionInvoked` and `NotificationClosed` off the bus: [`Signal`].
//!
//! # Not here
//!
//! - What each action does: [`crate::service`].

use std::collections::HashMap;

use zbus::Connection;
use zbus::MatchRule;
use zbus::MessageStream;
use zbus::message::Type;
use zbus::zvariant::Value;

const NAME: &str = "org.freedesktop.Notifications";
const PATH: &str = "/org/freedesktop/Notifications";

/// The app the notifications are from, as the shell titles them.
const APP: &str = "Files";
/// How long a notification stays, in milliseconds; -1 lets the shell decide.
const SHELL_DECIDES: i32 = -1;

/// What a notification says and the buttons it has.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Note {
    pub summary: String,
    pub body: String,
    pub icon: &'static str,
    /// Each button's key and label. The key `default` is a click on the notification itself.
    pub actions: Vec<(&'static str, String)>,
}

impl Note {
    #[must_use]
    pub fn new(icon: &'static str, summary: impl Into<String>, body: impl Into<String>) -> Self {
        Note {
            summary: summary.into(),
            body: body.into(),
            icon,
            actions: Vec::new(),
        }
    }

    /// The note with a button `label` that sends `key`.
    #[must_use]
    pub fn action(mut self, key: &'static str, label: impl Into<String>) -> Self {
        self.actions.push((key, label.into()));
        self
    }
}

/// Sends notifications on a session bus connection.
#[derive(Clone, Debug)]
pub struct Notifier {
    conn: Connection,
}

/// What the notification server said about a notification.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Signal {
    /// The user pressed the button with this key.
    Action(u32, String),
    Closed(u32),
}

impl Notifier {
    #[must_use]
    pub fn new(conn: Connection) -> Self {
        Notifier { conn }
    }

    /// Shows `note`, and returns its id, which its signals carry.
    ///
    /// # Errors
    ///
    /// When no notification server answers.
    pub async fn send(&self, note: &Note) -> zbus::Result<u32> {
        let actions: Vec<&str> = note
            .actions
            .iter()
            .flat_map(|(key, label)| [*key, label.as_str()])
            .collect();
        let hints: HashMap<&str, Value<'_>> = HashMap::new();
        let reply = self
            .conn
            .call_method(
                Some(NAME),
                PATH,
                Some(NAME),
                "Notify",
                &(APP, 0_u32, note.icon, &note.summary, &note.body, actions, hints, SHELL_DECIDES),
            )
            .await?;
        reply.body().deserialize::<u32>()
    }

    /// The notification server's signals, as they come.
    ///
    /// # Errors
    ///
    /// When the bus won't send them.
    pub async fn signals(&self) -> zbus::Result<MessageStream> {
        let rule = MatchRule::builder()
            .msg_type(Type::Signal)
            .interface(NAME)?
            .path(PATH)?
            .build();
        MessageStream::for_match_rule(rule, &self.conn, None).await
    }
}

/// `message` as a [`Signal`], if it is one of the two this reads.
pub fn signal(message: &zbus::Message) -> Option<Signal> {
    let header = message.header();
    match header.member()?.as_str() {
        "ActionInvoked" => {
            let (id, key): (u32, String) = message.body().deserialize().ok()?;
            Some(Signal::Action(id, key))
        }
        "NotificationClosed" => {
            let (id, _reason): (u32, u32) = message.body().deserialize().ok()?;
            Some(Signal::Closed(id))
        }
        _ => None,
    }
}
