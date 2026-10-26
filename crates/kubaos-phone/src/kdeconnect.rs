//! The real [`Link`]: kdeconnectd on the session bus through `busctl`, its user unit through
//! `systemctl --user`, and the clipboard through `wl-copy` and `wl-paste`.
//!
//! Every bus call passes `--auto-start=no`. KDE Connect ships a D-Bus activation file, and a
//! call that started kdeconnectd would put the laptop on the network without the user's say.
//! KubaOS doesn't install that file either (`home/phone.nix`).
//!
//! # Owns
//!
//! - The bus names, object paths and methods of kdeconnectd (26.04, the release's, and 26.08),
//!   and reading its answers.
//!
//! # Not here
//!
//! - Which phone, and when to refuse: [`crate::phone`].

use std::process::Command;
use std::process::Stdio;
use std::time::Duration;

use serde_json::Value;

use crate::device::Battery;
use crate::device::Device;
use crate::link::Action;
use crate::link::Link;
use crate::link::LinkError;
use crate::setting::Setting;

/// kdeconnectd's name on the session bus.
pub const BUS: &str = "org.kde.kdeconnect";
/// The user unit that runs kdeconnectd (`home/phone.nix`).
pub const UNIT: &str = "kubaos-phone.service";

const DAEMON: &str = "/modules/kdeconnect";
const DAEMON_IFACE: &str = "org.kde.kdeconnect.daemon";
const DEVICE_IFACE: &str = "org.kde.kdeconnect.device";
const PROPERTIES: &str = "org.freedesktop.DBus.Properties";
/// Interfaces a container runtime or a VM puts addresses on, which a phone can't reach.
const VIRTUAL: [&str; 4] = ["docker", "br-", "veth", "virbr"];

/// KDE Connect on this machine.
#[derive(Clone, Debug)]
pub struct KdeConnect {
    /// None without a home folder: the link is then off and can't be turned on.
    setting: Option<Setting>,
}

impl Default for KdeConnect {
    fn default() -> Self {
        KdeConnect::new(Setting::from_env())
    }
}

impl KdeConnect {
    #[must_use]
    pub fn new(setting: Option<Setting>) -> Self {
        KdeConnect { setting }
    }
}

impl Link for KdeConnect {
    fn is_on(&self) -> bool {
        self.setting.as_ref().is_some_and(Setting::is_on)
    }

    fn set_on(&mut self, on: bool) -> Result<(), LinkError> {
        let setting = self.setting.as_ref().ok_or(LinkError::NoHome)?;
        setting.set(on).map_err(LinkError::Setting)
    }

    fn start(&mut self) -> Result<(), LinkError> {
        run("systemctl", &["--user".into(), "start".into(), UNIT.into()]).map(drop)
    }

    fn stop(&mut self) -> Result<(), LinkError> {
        run("systemctl", &["--user".into(), "stop".into(), UNIT.into()]).map(drop)
    }

    fn running(&mut self) -> bool {
        bus_call(&[
            "org.freedesktop.DBus",
            "/org/freedesktop/DBus",
            "org.freedesktop.DBus",
            "NameHasOwner",
            "s",
            BUS,
        ])
        .is_ok_and(|v| v.as_bool() == Some(true))
    }

    fn name(&mut self) -> Result<String, LinkError> {
        let name = bus_call(&[BUS, DAEMON, DAEMON_IFACE, "announcedName"])?;
        name.as_str()
            .map(str::to_string)
            .ok_or_else(|| LinkError::Answer("kdeconnectd".into()))
    }

    fn hostname(&mut self) -> Option<String> {
        std::fs::read_to_string("/proc/sys/kernel/hostname")
            .ok()
            .map(|h| h.trim().to_string())
            .filter(|h| !h.is_empty())
    }

    fn devices(&mut self) -> Result<Vec<Device>, LinkError> {
        let ids = parse_ids(&bus_call(&[BUS, DAEMON, DAEMON_IFACE, "devices", "bb", "false", "false"])?);
        let mut devices = Vec::with_capacity(ids.len());
        for id in ids {
            // A device gone between the two calls is left out.
            if let Ok(device) = device(&id) {
                devices.push(device);
            }
        }
        Ok(devices)
    }

    fn addresses(&mut self) -> Vec<String> {
        run("ip", &["-j", "-4", "addr", "show", "scope", "global"].map(String::from))
            .map(|out| parse_addresses(&out))
            .unwrap_or_default()
    }

    fn call(&mut self, id: &str, action: &Action) -> Result<(), LinkError> {
        run("busctl", &busctl_owned(device_call(id, action)?)).map(drop)
    }

    fn copy(&mut self, text: &str) -> Result<(), LinkError> {
        // wl-copy stays behind to serve the clipboard: with a pipe on its output, waiting for the
        // output would wait for the next copy.
        let status = Command::new("wl-copy")
            .args(["--", text])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|err| LinkError::Spawn("wl-copy".into(), err))?;
        if status.success() {
            Ok(())
        } else {
            Err(LinkError::Failed {
                program: "wl-copy".into(),
                message: status.to_string(),
            })
        }
    }

    fn paste(&mut self) -> Result<String, LinkError> {
        run("wl-paste", &["--no-newline", "--type", "text"].map(String::from))
    }

    fn pause(&mut self, how_long: Duration) {
        std::thread::sleep(how_long);
    }
}

/// One `busctl call`, its first answer.
fn bus_call(args: &[&str]) -> Result<Value, LinkError> {
    let mut all = vec!["call"];
    all.extend_from_slice(args);
    first(&run("busctl", &busctl(&all))?)
}

fn properties(path: &str, iface: &str) -> Result<Value, LinkError> {
    bus_call(&[BUS, path, PROPERTIES, "GetAll", "s", iface])
}

fn device(id: &str) -> Result<Device, LinkError> {
    if !valid_id(id) {
        return Err(LinkError::BadId(id.to_string()));
    }
    let path = format!("{DAEMON}/devices/{id}");
    let mut device = parse_device(id, &properties(&path, DEVICE_IFACE)?);
    if device.paired && device.reachable {
        device.battery = properties(&format!("{path}/battery"), "org.kde.kdeconnect.device.battery")
            .ok()
            .and_then(|v| parse_battery(&v));
    }
    Ok(device)
}

/// `busctl`'s arguments for `rest`: the user's bus, JSON out, and never starting a service.
fn busctl(rest: &[&str]) -> Vec<String> {
    ["--user", "--auto-start=no", "--json=short"]
        .iter()
        .chain(rest)
        .map(|arg| (*arg).to_string())
        .collect()
}

fn busctl_owned(rest: Vec<String>) -> Vec<String> {
    let mut args = busctl(&[]);
    args.extend(rest);
    args
}

/// The `busctl` call (from `call` on) asking device `id` for `action`.
fn device_call(id: &str, action: &Action) -> Result<Vec<String>, LinkError> {
    if !valid_id(id) {
        return Err(LinkError::BadId(id.to_string()));
    }
    let (plugin, iface, method, arg) = match action {
        Action::Accept => ("", DEVICE_IFACE, "acceptPairing", None),
        Action::Unpair => ("", DEVICE_IFACE, "unpair", None),
        Action::SendClipboard => ("/clipboard", "org.kde.kdeconnect.device.clipboard", "sendClipboard", None),
        Action::Share(url) => ("/share", "org.kde.kdeconnect.device.share", "shareUrl", Some(url)),
        Action::Ring => ("/findmyphone", "org.kde.kdeconnect.device.findmyphone", "ring", None),
    };
    let mut args = vec![
        "call".to_string(),
        BUS.into(),
        format!("{DAEMON}/devices/{id}{plugin}"),
        iface.into(),
        method.into(),
    ];
    if let Some(arg) = arg {
        args.extend(["s".to_string(), arg.clone()]);
    }
    Ok(args)
}

/// KDE Connect's ids are an object path's last element: letters, digits and underscores. Anything
/// else would point the call at another object.
fn valid_id(id: &str) -> bool {
    !id.is_empty() && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// The first value of a `busctl --json=short call` answer: `{"type":"as","data":[[...]]}`.
fn first(answer: &str) -> Result<Value, LinkError> {
    let bad = || LinkError::Answer("busctl".into());
    let mut value: Value = serde_json::from_str(answer).map_err(|_| bad())?;
    value
        .get_mut("data")
        .and_then(Value::as_array_mut)
        .filter(|data| !data.is_empty())
        .map(|data| data.swap_remove(0))
        .ok_or_else(bad)
}

fn parse_ids(value: &Value) -> Vec<String> {
    value
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect()
}

/// One property of a `GetAll` answer: `{"name":{"type":"s","data":"iKuba"}}`.
fn property<'a>(props: &'a Value, name: &str) -> Option<&'a Value> {
    props.get(name).and_then(|p| p.get("data"))
}

fn parse_device(id: &str, props: &Value) -> Device {
    let text = |name| property(props, name).and_then(Value::as_str).map(str::to_string);
    let flag = |name| property(props, name).and_then(Value::as_bool).unwrap_or(false);
    Device {
        id: id.to_string(),
        name: text("name").unwrap_or_else(|| id.to_string()),
        kind: text("type").unwrap_or_default(),
        paired: flag("isPaired"),
        reachable: flag("isReachable"),
        wants_to_pair: flag("isPairRequestedByPeer"),
        verification_key: text("verificationKey").filter(|k| !k.is_empty()),
        battery: None,
    }
}

/// A phone that hasn't told its charge yet answers -1.
fn parse_battery(props: &Value) -> Option<Battery> {
    let charge = property(props, "charge")?.as_i64()?;
    Some(Battery {
        charge: u8::try_from(charge).ok().filter(|c| *c <= 100)?,
        charging: property(props, "isCharging").and_then(Value::as_bool).unwrap_or(false),
    })
}

/// The IPv4 addresses in `ip -j -4 addr show scope global`, leaving out virtual interfaces.
fn parse_addresses(json: &str) -> Vec<String> {
    let Ok(Value::Array(links)) = serde_json::from_str::<Value>(json) else {
        return vec![];
    };
    links
        .iter()
        .filter(|l| {
            let name = l.get("ifname").and_then(Value::as_str).unwrap_or_default();
            !VIRTUAL.iter().any(|v| name.starts_with(v))
        })
        .flat_map(|l| l.get("addr_info").and_then(Value::as_array).into_iter().flatten())
        .filter_map(|a| a.get("local").and_then(Value::as_str))
        .map(str::to_string)
        .collect()
}

/// Runs `program`; a failure carries the last line it wrote on stderr.
fn run(program: &str, args: &[String]) -> Result<String, LinkError> {
    let out = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .output()
        .map_err(|err| LinkError::Spawn(program.to_string(), err))?;
    if out.status.success() {
        return Ok(String::from_utf8_lossy(&out.stdout).into_owned());
    }
    let stderr = String::from_utf8_lossy(&out.stderr);
    let message = stderr
        .lines()
        .rev()
        .find(|l| !l.trim().is_empty())
        .map_or_else(|| out.status.to_string(), |l| l.trim().to_string());
    Err(LinkError::Failed {
        program: program.to_string(),
        message,
    })
}

#[cfg(test)]
mod tests;
