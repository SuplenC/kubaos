//! The system's file picker, through xdg-desktop-portal's `FileChooser`: the Command menu's file
//! and folder fields open it from their Browse… button. The portal shows the picker of the
//! desktop's portal backend (GTK's, under Hyprland), filtered to the kinds of file the field takes,
//! or a folder picker, and answers with what was chosen.
//!
//! The interface: <https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.FileChooser.html>
//!
//! # Owns
//!
//! - Asking the portal for one file or folder, and reading its answer.
//! - Turning a field's `accept` (extensions and MIME types) into the portal's filter.
//!
//! # Not here
//!
//! - Marshalling and the socket: `kotodama_extension::dbus`.
//! - Turning the chosen `file://` URI into a path: [`crate::entry`].
//! - The field and its button: `config/quickshell/modules/menu/FileField.qml`.

use std::path::PathBuf;

use kotodama_extension::dbus;
use kotodama_extension::dbus::Bus;
use kotodama_extension::dbus::Call;
use kotodama_extension::dbus::Marshal;
use kotodama_extension::dbus::Unmarshal;

use crate::entry::EntryError;
use crate::entry::Resolver;

const PORTAL: &str = "org.freedesktop.portal.Desktop";
const PORTAL_OBJECT: &str = "/org/freedesktop/portal/desktop";
const FILE_CHOOSER: &str = "org.freedesktop.portal.FileChooser";
const DBUS: &str = "org.freedesktop.DBus";
const DBUS_OBJECT: &str = "/org/freedesktop/DBus";
/// The portal answers on a Request object of its own with this signal, sent to the caller alone,
/// so one process asking once needs no more than the signal's name to know the answer is its own.
const RESPONSE_MATCH: &str = "type='signal',interface='org.freedesktop.portal.Request',member='Response'";
/// A pattern filter: a glob over the file's name, as `*.conf`.
const GLOB: u32 = 0;
/// A MIME type filter, as `image/*`.
const MIME: u32 = 1;
/// The response that carries a choice; 1 is a cancel and 2 the picker closing some other way.
const CHOSEN: u32 = 0;

/// What to ask the picker for.
#[derive(Debug, Default)]
pub struct Pick {
    /// The picker's window title: what the path is for.
    pub title: String,
    /// A folder, not a file.
    pub folder: bool,
    /// The file extensions (`conf`) or MIME types (`image/*`) a file may have; none for any.
    pub accept: Vec<String>,
}

/// Why nothing could be asked: no session bus, no portal, or an answer that can't be read.
#[derive(Debug, thiserror::Error)]
pub enum PickError {
    #[error("asking xdg-desktop-portal for a file")]
    Bus(#[source] dbus::Error),
    #[error("xdg-desktop-portal's answer can't be read")]
    Malformed,
    #[error("the file chosen isn't a local path")]
    Uri(#[source] EntryError),
}

impl PickError {
    fn bus(err: dbus::Error) -> Self {
        PickError::Bus(err)
    }
}

/// The portal's filter entries for `accept`: a glob over the name, either case, for an extension,
/// a MIME type for a word with a `/`.
#[must_use]
pub fn patterns(accept: &[String]) -> Vec<(u32, String)> {
    accept
        .iter()
        .map(|kind| {
            if kind.contains('/') {
                (MIME, kind.clone())
            } else {
                (GLOB, format!("*.{}", either_case(kind.trim_start_matches('.'))))
            }
        })
        .collect()
}

/// `extension` as a glob that takes it in any case, `[cC][oO][nN][fF]`: the portal's globs match
/// case, and a config mailed as `WORK.CONF` is still one.
fn either_case(extension: &str) -> String {
    extension
        .chars()
        .map(|c| {
            let (lower, upper) = (
                c.to_lowercase().collect::<String>(),
                c.to_uppercase().collect::<String>(),
            );
            if lower == upper {
                lower
            } else {
                format!("[{lower}{upper}]")
            }
        })
        .collect()
}

/// Opens the picker and waits, however long the user takes, for what they choose: its path, or
/// `None` once they cancel.
///
/// # Errors
///
/// When the session bus or the portal can't be reached, or the portal's answer can't be read.
pub fn pick(request: &Pick, resolver: &Resolver) -> Result<Option<PathBuf>, PickError> {
    let mut bus = Bus::session().map_err(PickError::bus)?;
    let mut rule = Marshal::default();
    rule.string(RESPONSE_MATCH);
    bus.call(
        &Call::new(DBUS_OBJECT, DBUS, "AddMatch", DBUS).with_body("s", rule),
        |_| {},
    )
    .map_err(PickError::bus)?;
    bus.call(
        &Call::new(PORTAL_OBJECT, FILE_CHOOSER, "OpenFile", PORTAL).with_body("ssa{sv}", open_file(request)),
        |_| {},
    )
    .map_err(PickError::bus)?;
    bus.wait_forever().map_err(PickError::bus)?;
    let response = loop {
        let message = bus.read().map_err(PickError::bus)?;
        if message.is_signal("Response", "ua{sv}") {
            break message;
        }
    };
    match answer(response.body()).ok_or(PickError::Malformed)? {
        Answer::Chosen(uri) => resolver.uri(&uri).map(Some).map_err(PickError::Uri),
        Answer::Cancelled => Ok(None),
    }
}

/// `OpenFile`'s arguments: no parent window, the title, and the options.
fn open_file(request: &Pick) -> Marshal {
    let mut body = Marshal::default();
    body.string("");
    body.string(&request.title);
    body.array(8, |options| {
        let token = format!("kubaos{}", std::process::id());
        options.property("handle_token", "s", |m| m.string(&token));
        if request.folder {
            options.property("directory", "b", |m| m.bool(true));
        }
        let patterns = patterns(&request.accept);
        if patterns.is_empty() {
            return;
        }
        let name = request.accept.join(", ");
        options.property("filters", "a(sa(us))", |m| {
            m.array(8, |filters| filter(filters, &name, &patterns));
        });
        options.property("current_filter", "(sa(us))", |m| filter(m, &name, &patterns));
    });
    body
}

/// One `(sa(us))`: the filter's name and its entries.
fn filter(m: &mut Marshal, name: &str, patterns: &[(u32, String)]) {
    m.pad(8);
    m.string(name);
    m.array(8, |entries| {
        for (kind, pattern) in patterns {
            entries.pad(8);
            entries.u32(*kind);
            entries.string(pattern);
        }
    });
}

/// How the picker closed.
enum Answer {
    /// With the URI of what was chosen.
    Chosen(String),
    /// With nothing chosen.
    Cancelled,
}

/// What a `Response`'s body says, or `None` when it can't be read.
fn answer(mut body: Unmarshal<'_>) -> Option<Answer> {
    if body.u32()? != CHOSEN {
        return Some(Answer::Cancelled);
    }
    let mut results = body.array("{sv}")?;
    while !results.is_empty() {
        results.align_struct();
        let key = results.string()?;
        let signature = results.variant()?;
        if key == "uris" && signature == "as" {
            let mut uris = results.array("s")?;
            if uris.is_empty() {
                return Some(Answer::Cancelled);
            }
            return Some(Answer::Chosen(uris.string()?.to_owned()));
        }
        results.skip(signature)?;
    }
    Some(Answer::Cancelled)
}

#[cfg(test)]
mod tests;
