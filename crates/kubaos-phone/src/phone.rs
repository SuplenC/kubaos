//! What the phone link does, on any [`Link`]: turn it on and off, accept a pairing, and send to
//! the paired phone. Kōbō's card and `kubaos-phone` both come through here, so they refuse the
//! same things for the same reasons.
//!
//! # Owns
//!
//! - The rules: nothing reaches kdeconnectd while the link is off, a pairing is accepted only for a
//!   device that asked, and which paired phone a send goes to.
//! - The words for each refusal ([`Error`], [`Error::hint`]).
//!
//! # Not here
//!
//! - The machine: [`crate::link`].

use std::fmt::Write;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

use thiserror::Error;

use crate::device;
use crate::device::Device;
use crate::device::Status;
use crate::link::Action;
use crate::link::Link;
use crate::link::LinkError;

/// What the test copy puts on the clipboard, and what the phone should then paste.
pub const TEST_TEXT: &str = "Hello from KubaOS";

/// How long kdeconnectd gets to appear on the bus after its unit starts. It takes about a
/// second on the laptop; a cold start after a switch takes longer.
pub const START_WAIT: Duration = Duration::from_secs(10);

/// The KDE Connect plugins a device keeps: what the phone link is for (the clipboard, files and
/// text both ways, ping, ring the phone or the laptop) and the phone's own reports (battery,
/// signal). Every other plugin is turned off for every device, the ones a later KDE Connect adds
/// included: most of them let the phone act on the laptop (remote input, run commands, the
/// presenter, media control, locking, keeping the screen awake, browsing files).
pub const LINK_PLUGINS: [&str; 7] = [
    "kdeconnect_clipboard",
    "kdeconnect_share",
    "kdeconnect_ping",
    "kdeconnect_battery",
    "kdeconnect_findmyphone",
    "kdeconnect_findthisdevice",
    "kdeconnect_connectivity_report",
];

/// How long an accepted pairing gets to settle into paired.
pub const PAIR_WAIT: Duration = Duration::from_secs(5);

const POLL: Duration = Duration::from_millis(250);

/// Why the phone link didn't do what was asked.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    #[error("the phone link is off")]
    Off,
    #[error("KDE Connect isn't running")]
    NotRunning,
    #[error("KDE Connect didn't start")]
    NotStarted,
    #[error("no phone is paired")]
    NoPhone,
    #[error("no paired phone is called {0}")]
    Unknown(String),
    #[error("{0} isn't in reach")]
    Unreachable(String),
    #[error("{0} isn't asking to pair")]
    NoRequest(String),
    #[error("{0} accepted, but it isn't paired")]
    NotPaired(String),
    #[error("{} isn't a file", .0.display())]
    NoFile(PathBuf),
    #[error("nothing new from the phone on the clipboard")]
    NothingNew,
    #[error("talking to KDE Connect")]
    Link(#[source] LinkError),
}

impl Error {
    /// What the user can do about it, in a sentence.
    pub fn hint(&self) -> &'static str {
        match self {
            Error::Off => "Set up a phone first: \"Set up a phone\" in the Command menu.",
            Error::NotRunning | Error::NotStarted => {
                "journalctl --user -u kubaos-phone says why. Turning the phone link off and on again restarts it."
            }
            Error::NoPhone | Error::NotPaired(_) => {
                "Pair one: \"Set up a phone\" in the Command menu walks you through it."
            }
            Error::Unknown(_) => "kubaos-phone status lists the paired ones.",
            Error::Unreachable(_) => {
                "Open KDE Connect on the phone, on the same Wi-Fi as this laptop. On iPhone the app has to be open."
            }
            Error::NoRequest(_) => "In the phone's KDE Connect app, tap this laptop, then Request pairing.",
            Error::NoFile(_) => "Pick a file, not a folder.",
            Error::NothingNew => {
                "On the phone: copy something, then Send clipboard in KDE Connect. iPhone never sends it on its own."
            }
            Error::Link(_) => "journalctl --user -u kubaos-phone has KDE Connect's side.",
        }
    }
}

/// The phone link as it stands. While it is off, kdeconnectd isn't asked anything: the name is
/// the host name the link would start with.
pub fn status(link: &mut dyn Link) -> Status {
    let on = link.is_on();
    let running = on && link.running();
    let name = running.then(|| link.name().ok()).flatten().or_else(|| link.hostname());
    let devices = if running {
        link.devices().unwrap_or_default()
    } else {
        vec![]
    };
    Status {
        on,
        running,
        name,
        addresses: if on { link.addresses() } else { vec![] },
        devices,
    }
}

/// Turns the phone link on and waits for kdeconnectd. The setting goes first, since the unit
/// starts only once it exists.
///
/// # Errors
///
/// When the setting can't be written, systemd refuses, or kdeconnectd doesn't appear within
/// [`START_WAIT`]. The setting stays on then, so the next login tries again.
pub fn turn_on(link: &mut dyn Link) -> Result<Status, Error> {
    link.set_on(true).map_err(Error::Link)?;
    link.start().map_err(Error::Link)?;
    if !wait_for(link, START_WAIT, |link| link.running()) {
        return Err(Error::NotStarted);
    }
    restrict(link)?;
    Ok(status(link))
}

/// Turns off every plugin outside [`LINK_PLUGINS`] and the user's own list, for every device
/// KDE Connect knows, paired or asking to be. A plugin the user turned off is left off. kdeconnectd
/// writes plugins into each device's config as it meets the device, on by default, so this runs
/// at `on`, around a pairing, and each time the unit starts (`home/phone.nix`).
///
/// # Errors
///
/// When the link isn't running, or kdeconnectd refuses a change.
///
/// Returns what it turned off, as `<device>: <plugin> off`.
pub fn restrict(link: &mut dyn Link) -> Result<Vec<String>, Error> {
    let extra = link.extra_plugins();
    let allowed = |p: &str| LINK_PLUGINS.contains(&p) || extra.iter().any(|e| e == p);
    let mut changed = vec![];
    for device in devices(link)? {
        for (plugin, on) in link.plugins(&device.id).map_err(Error::Link)? {
            if on && !allowed(&plugin) {
                link.set_plugin(&device.id, &plugin, false).map_err(Error::Link)?;
                changed.push(format!("{}: {plugin} off", device.name));
            }
        }
    }
    Ok(changed)
}

/// Turns the phone link off and stops kdeconnectd. The setting goes first, so nothing starts it
/// again on the way.
///
/// # Errors
///
/// When the setting can't be removed or systemd refuses.
pub fn turn_off(link: &mut dyn Link) -> Result<(), Error> {
    link.set_on(false).map_err(Error::Link)?;
    link.stop().map_err(Error::Link)
}

/// Accepts the pairing request of the device `id`, and waits for it to be paired.
///
/// # Errors
///
/// When the link isn't running, the device isn't asking to pair, or the pairing doesn't settle.
pub fn accept(link: &mut dyn Link, id: &str) -> Result<Device, Error> {
    let devices = devices(link)?;
    let Some(device) = devices.into_iter().find(|d| d.id == id) else {
        return Err(Error::NoRequest(id.to_string()));
    };
    if !device.wants_to_pair {
        return Err(Error::NoRequest(device.name));
    }
    // Before as well as after: the plugins load the moment the pairing is in.
    restrict(link)?;
    link.call(id, &Action::Accept).map_err(Error::Link)?;
    let paired = |link: &mut dyn Link| {
        link.devices()
            .ok()
            .and_then(|all| all.into_iter().find(|d| d.id == id && d.paired))
    };
    if !wait_for(link, PAIR_WAIT, |link| paired(link).is_some()) {
        return Err(Error::NotPaired(device.name));
    }
    restrict(link)?;
    paired(link).ok_or(Error::NotPaired(device.name))
}

/// Sends the laptop's clipboard to the phone `wanted` (a name or an id), else the paired one.
///
/// # Errors
///
/// When the link isn't running or the phone isn't paired and in reach.
pub fn send_clipboard(link: &mut dyn Link, wanted: Option<&str>) -> Result<Device, Error> {
    send(link, wanted, &Action::SendClipboard)
}

/// Sends the file `path` to the phone.
///
/// # Errors
///
/// As [`send_clipboard`], and when `path` isn't a file.
pub fn send_file(link: &mut dyn Link, wanted: Option<&str>, path: &Path) -> Result<Device, Error> {
    if !path.is_file() {
        return Err(Error::NoFile(path.to_path_buf()));
    }
    let path = std::path::absolute(path).map_err(|_| Error::NoFile(path.to_path_buf()))?;
    send(link, wanted, &Action::Share(file_url(&path)))
}

/// Rings the phone, even when it is on silent.
///
/// # Errors
///
/// As [`send_clipboard`].
pub fn ring(link: &mut dyn Link, wanted: Option<&str>) -> Result<Device, Error> {
    send(link, wanted, &Action::Ring)
}

/// Unpairs the phone. It needn't be in reach: the laptop forgets it either way.
///
/// # Errors
///
/// When the link isn't running or no such phone is paired.
pub fn unpair(link: &mut dyn Link, wanted: Option<&str>) -> Result<Device, Error> {
    let phone = pick(&devices(link)?, wanted)?;
    link.call(&phone.id, &Action::Unpair).map_err(Error::Link)?;
    Ok(phone)
}

/// The test of the laptop-to-phone direction: [`TEST_TEXT`] on the clipboard, which KDE Connect
/// passes on as it changes, and sent once more by hand in case it doesn't.
///
/// # Errors
///
/// As [`send_clipboard`], and when the clipboard can't be written.
pub fn send_test(link: &mut dyn Link) -> Result<Device, Error> {
    let phone = reachable(link, None)?;
    link.copy(TEST_TEXT).map_err(Error::Link)?;
    link.call(&phone.id, &Action::SendClipboard).map_err(Error::Link)?;
    Ok(phone)
}

/// The test of the phone-to-laptop direction: the clipboard's text, once it is something other
/// than [`TEST_TEXT`].
///
/// # Errors
///
/// When the clipboard holds nothing new.
pub fn received(link: &mut dyn Link) -> Result<String, Error> {
    let text = link.paste().unwrap_or_default();
    let text = text.trim();
    if text.is_empty() || text == TEST_TEXT {
        return Err(Error::NothingNew);
    }
    Ok(text.to_string())
}

/// `arg` as a path, `~` and `~/…` under `home`. The Command menu passes what was typed, with no
/// shell to expand it.
pub fn typed_path(arg: &str, home: Option<&Path>) -> PathBuf {
    match (arg, home) {
        ("~", Some(home)) => home.to_path_buf(),
        (_, Some(home)) if arg.starts_with("~/") => home.join(&arg[2..]),
        _ => PathBuf::from(arg),
    }
}

fn send(link: &mut dyn Link, wanted: Option<&str>, action: &Action) -> Result<Device, Error> {
    let phone = reachable(link, wanted)?;
    link.call(&phone.id, action).map_err(Error::Link)?;
    Ok(phone)
}

fn reachable(link: &mut dyn Link, wanted: Option<&str>) -> Result<Device, Error> {
    let phone = pick(&devices(link)?, wanted)?;
    if !phone.reachable {
        return Err(Error::Unreachable(phone.name));
    }
    Ok(phone)
}

/// The devices, once the link is on and kdeconnectd answers.
fn devices(link: &mut dyn Link) -> Result<Vec<Device>, Error> {
    if !link.is_on() {
        return Err(Error::Off);
    }
    if !link.running() {
        return Err(Error::NotRunning);
    }
    link.devices().map_err(Error::Link)
}

/// The paired device `wanted` names (its id, or its name in any case), else the paired one in
/// reach, else any paired one.
fn pick(devices: &[Device], wanted: Option<&str>) -> Result<Device, Error> {
    let found = match wanted {
        Some(w) => devices
            .iter()
            .find(|d| d.paired && (d.id == w || d.name.eq_ignore_ascii_case(w)))
            .ok_or_else(|| Error::Unknown(w.to_string()))?,
        None => device::preferred(devices).ok_or(Error::NoPhone)?,
    };
    Ok(found.clone())
}

/// Waits up to `limit` for `ready`, asking every [`POLL`].
fn wait_for(link: &mut dyn Link, limit: Duration, mut ready: impl FnMut(&mut dyn Link) -> bool) -> bool {
    let mut waited = Duration::ZERO;
    loop {
        if ready(link) {
            return true;
        }
        if waited >= limit {
            return false;
        }
        link.pause(POLL);
        waited += POLL;
    }
}

/// `path` as a `file://` URL, every byte outside the unreserved set and `/` escaped. KDE Connect
/// parses the URL, so a `#` or `?` in a file name would otherwise cut the path short.
fn file_url(path: &Path) -> String {
    let mut url = String::from("file://");
    for byte in path.as_os_str().as_encoded_bytes() {
        if byte.is_ascii_alphanumeric() || b"/-._~".contains(byte) {
            url.push(char::from(*byte));
        } else {
            let _ = write!(url, "%{byte:02X}");
        }
    }
    url
}

#[cfg(test)]
mod tests;
