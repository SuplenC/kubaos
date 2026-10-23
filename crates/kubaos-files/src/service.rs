//! `org.freedesktop.FileManager1` on the session bus, which apps call for "Show in folder"
//! (Chrome's downloads, VS Code, Telegram, …). D-Bus starts it on the first call; it answers until
//! the calls stop for [`DEFAULT_IDLE`], then quits, so nothing stays resident.
//!
//! The interface: <https://www.freedesktop.org/wiki/Specifications/file-manager-interface/>
//!
//! # Owns
//!
//! - Taking the name, answering `ShowFolders`, `ShowItems` and `ShowItemProperties`, and
//!   introspection, then giving the name back.
//! - What a call's URIs open.
//!
//! # Not here
//!
//! - Marshalling and the socket: `kotodama_extension::dbus`.
//! - Opening the window: [`crate::window`].
//! - The activation file: `home/files.nix`.

use std::io;
use std::path::PathBuf;
use std::time::Duration;
use std::time::Instant;

use kotodama_extension::dbus;
use kotodama_extension::dbus::Bus;
use kotodama_extension::dbus::Call;
use kotodama_extension::dbus::Marshal;
use kotodama_extension::dbus::Message;

use crate::entry;
use crate::entry::EntryError;
use crate::entry::Resolver;

/// The bus name, which is also the interface.
pub const NAME: &str = "org.freedesktop.FileManager1";
/// The object the methods are on.
pub const OBJECT: &str = "/org/freedesktop/FileManager1";
/// The methods, each taking the URIs (`as`) and a startup id (`s`). yazi has no properties
/// window, so `ShowItemProperties` reveals the items as `ShowItems` does.
pub const METHODS: [&str; 3] = ["ShowFolders", "ShowItems", "ShowItemProperties"];

/// How long the service waits for another call before it quits: a few "Show in folder" clicks in
/// a row reuse it, and soon after the last one it is gone. D-Bus starts it again on the next call.
pub const DEFAULT_IDLE: Duration = Duration::from_secs(10);

const DBUS: &str = "org.freedesktop.DBus";
const DBUS_OBJECT: &str = "/org/freedesktop/DBus";
const INTROSPECTABLE: &str = "org.freedesktop.DBus.Introspectable";
const PEER: &str = "org.freedesktop.DBus.Peer";
/// `RequestName`'s flag to fail rather than queue behind another owner. A second copy that
/// started at the same time then gets no calls and quits once idle.
const DO_NOT_QUEUE: u32 = 4;

const INTROSPECTION: &str = r#"<!DOCTYPE node PUBLIC "-//freedesktop//DTD D-BUS Object Introspection 1.0//EN"
 "http://www.freedesktop.org/standards/dbus/1.0/introspect.dtd">
<node>
  <interface name="org.freedesktop.FileManager1">
    <method name="ShowFolders">
      <arg type="as" name="URIs" direction="in"/>
      <arg type="s" name="StartupId" direction="in"/>
    </method>
    <method name="ShowItems">
      <arg type="as" name="URIs" direction="in"/>
      <arg type="s" name="StartupId" direction="in"/>
    </method>
    <method name="ShowItemProperties">
      <arg type="as" name="URIs" direction="in"/>
      <arg type="s" name="StartupId" direction="in"/>
    </method>
  </interface>
  <interface name="org.freedesktop.DBus.Introspectable">
    <method name="Introspect">
      <arg type="s" name="xml_data" direction="out"/>
    </method>
  </interface>
  <interface name="org.freedesktop.DBus.Peer">
    <method name="Ping"/>
  </interface>
</node>
"#;

/// Why the service stopped before it went idle: the session bus couldn't be reached, or went away.
#[derive(Debug, thiserror::Error)]
#[error("talking to the session bus")]
pub struct ServeError(#[source] dbus::Error);

impl ServeError {
    fn bus(err: dbus::Error) -> Self {
        ServeError(err)
    }
}

/// What the URIs of a call open, and the ones that name no local path.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Entries {
    /// The local path of each URI, or its nearest folder that exists, in order and without repeats.
    pub paths: Vec<PathBuf>,
    pub refused: Vec<EntryError>,
}

/// What `uris` open, resolved by `resolver`.
pub fn entries<'a>(uris: impl IntoIterator<Item = &'a str>, resolver: &Resolver) -> Entries {
    let mut entries = Entries::default();
    for uri in uris {
        match resolver.uri(uri) {
            Ok(path) => {
                let path = entry::existing(&path);
                if !entries.paths.contains(&path) {
                    entries.paths.push(path);
                }
            }
            Err(err) => entries.refused.push(err),
        }
    }
    entries
}

/// Takes the name and answers its calls, passing what each one opens to `open`, until none has
/// come for `idle`; then gives the name back and answers the calls already on their way.
///
/// # Errors
///
/// When the session bus can't be reached, or hangs up.
pub fn serve(resolver: &Resolver, idle: Duration, mut open: impl FnMut(Entries)) -> Result<(), ServeError> {
    let mut bus = Bus::session().map_err(ServeError::bus)?;
    let mut request = Marshal::default();
    request.string(NAME);
    request.u32(DO_NOT_QUEUE);
    bus.send(&Call::new(DBUS_OBJECT, DBUS, "RequestName", DBUS).with_body("su", request))
        .map_err(ServeError::bus)?;
    answer_until_idle(&mut bus, resolver, idle, &mut open)?;

    // HAZARD: give the name back before the last reads, not after. The bus sends a call to the
    // owner of the name, and one sent to a process that has stopped reading is never answered:
    // its app waits out its whole timeout. Once the name is free, the next call starts a new copy.
    let mut release = Marshal::default();
    release.string(NAME);
    bus.send(&Call::new(DBUS_OBJECT, DBUS, "ReleaseName", DBUS).with_body("s", release))
        .map_err(ServeError::bus)?;
    answer_until_idle(&mut bus, resolver, Duration::ZERO, &mut open)
}

/// Answers calls until a read times out with none for `idle`. The bus module's reads time out
/// after 2 s, so a zero `idle` reads until the first quiet 2 s.
fn answer_until_idle(
    bus: &mut Bus,
    resolver: &Resolver,
    idle: Duration,
    open: &mut impl FnMut(Entries),
) -> Result<(), ServeError> {
    let mut last_call = Instant::now();
    loop {
        match bus.read() {
            Ok(message) => {
                if answer(bus, &message, resolver, open)? {
                    last_call = Instant::now();
                }
            }
            Err(dbus::Error::Io(err)) if is_timeout(&err) => {
                if last_call.elapsed() >= idle {
                    return Ok(());
                }
            }
            Err(err) => return Err(ServeError::bus(err)),
        }
    }
}

/// Answers `message` if it is a call to the service, and says whether it was. The reply goes out
/// before the window opens, so the app calling never waits on the terminal.
fn answer(
    bus: &mut Bus,
    message: &Message,
    resolver: &Resolver,
    open: &mut impl FnMut(Entries),
) -> Result<bool, ServeError> {
    if METHODS.iter().any(|method| message.is_call(OBJECT, NAME, method)) {
        let entries = entries(uris(message), resolver);
        bus.reply(message, "", &Marshal::default()).map_err(ServeError::bus)?;
        open(entries);
        return Ok(true);
    }
    if message.is_call(OBJECT, INTROSPECTABLE, "Introspect") {
        let mut xml = Marshal::default();
        xml.string(INTROSPECTION);
        bus.reply(message, "s", &xml).map_err(ServeError::bus)?;
        return Ok(true);
    }
    if message.is_call(OBJECT, PEER, "Ping") {
        bus.reply(message, "", &Marshal::default()).map_err(ServeError::bus)?;
        return Ok(true);
    }
    Ok(false)
}

/// The URIs a method call carries as its first argument; none when the body isn't an `as`.
fn uris(message: &Message) -> Vec<&str> {
    let mut body = message.body();
    let Some(mut array) = body.array("s") else {
        return Vec::new();
    };
    let mut uris = Vec::new();
    while !array.is_empty() {
        let Some(uri) = array.string() else {
            break;
        };
        uris.push(uri);
    }
    uris
}

fn is_timeout(err: &io::Error) -> bool {
    matches!(err.kind(), io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut)
}

#[cfg(test)]
mod tests;
