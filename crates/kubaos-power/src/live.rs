//! What `status` asks outside `/sys`: power-profiles-daemon's profile, Hyprland's laptop panel,
//! `iw`'s Wi-Fi power saving, the indexer's pause, the scheduler's unit and its journal, and the
//! stop the service published. The command line and the Command menu's "Power: what this stop
//! does" and "CPU scheduler: status" read it here.
//!
//! # Owns
//!
//! - [`read`] and [`scheduler`], and the questions they ask: [`profile`], [`laptop_panel`],
//!   [`wifi_power_save`], [`indexer_paused`], [`scheduler_unit`], [`lavd_exit`].
//!
//! # Not here
//!
//! - The rows and their words: [`crate::status`].

use std::fmt::Write as _;
use std::path::Path;
use std::process::Command;
use std::process::Stdio;

use serde_json::Value;

use crate::machine::Machine;
use crate::plan::Plan;
use crate::runtime;
use crate::runtime::Published;
use crate::scheduler;
use crate::scheduler::UnitState;
use crate::settings::Settings;
use crate::status;
use crate::status::Live;
use crate::status::Row;
use crate::stop::Profile;
use crate::stop::Source;
use crate::stop::Stop;

/// The transient user unit that holds the file indexer paused.
pub const QUIET_UNIT: &str = "kubaos-power-quiet";

/// power-profiles-daemon's bus name, path and interface since 0.20.
pub const PPD_SERVICE: &str = "org.freedesktop.UPower.PowerProfiles";
pub const PPD_PATH: &str = "/org/freedesktop/UPower/PowerProfiles";

/// The stop now, and every row of it.
#[derive(Clone, Debug)]
pub struct Now {
    pub stop: Stop,
    /// A line saying the stop, the power source and anything unusual.
    pub header: String,
    pub rows: Vec<Row>,
}

/// The stop now and its rows, under `settings`; `problem` is what is wrong with the settings
/// file, if anything.
pub fn read(settings: &Settings, problem: Option<&str>, machine: &Machine) -> Now {
    let profile = profile();
    let published = runtime::file().and_then(|file| Published::load(&file));
    let stop = stop_now(published.as_ref(), profile);
    let source = published
        .as_ref()
        .and_then(|published| match published.source.as_str() {
            "battery" => Some(Source::Battery),
            "ac" => Some(Source::Ac),
            _ => None,
        })
        .or_else(|| {
            machine
                .battery()
                .map(|battery| Source::from_on_battery(battery.discharging))
        });
    let live = Live {
        profile: profile.map(|profile| profile.name().to_owned()),
        source,
        panel: laptop_panel(),
        wifi: machine
            .wifi()
            .into_iter()
            .map(|interface| {
                let on = wifi_power_save(&interface);
                (interface, on)
            })
            .collect(),
        indexer_paused: indexer_paused(),
        scheduler_unit: scheduler_unit(),
        scheduler_exit: lavd_exit(),
    };
    let mut header = format!("Power: {stop}");
    if let Some(source) = source {
        let _ = write!(header, ", {}", source.words());
    }
    if published.is_none() {
        header.push_str(" (the power service isn't running: only power-profiles-daemon's rows apply)");
    }
    if let Some(from) = published
        .as_ref()
        .and_then(|published| Stop::from_id(published.dropped_from.as_deref()?))
    {
        let _ = write!(header, " (the low-battery drop, from {from})");
    }
    // The scheduler's own row says why LAVD didn't start.
    let refused: Vec<&str> = published
        .iter()
        .flat_map(|published| published.refused.iter().map(String::as_str))
        .filter(|arg| !arg.starts_with("scheduler="))
        .collect();
    if !refused.is_empty() {
        let _ = write!(
            header,
            "\nThe kernel refused {}; asked again at the next stop",
            refused.join(", ")
        );
    }
    if let Some(problem) = problem {
        let _ = write!(header, "\n{problem}; KubaOS's defaults apply");
    }
    let plan = Plan::for_stop(stop, settings);
    Now {
        stop,
        header,
        rows: status::rows(&plan, settings, machine, &live),
    }
}

/// The stop the service published, else the one power-profiles-daemon's profile is named after.
fn stop_now(published: Option<&Published>, profile: Option<Profile>) -> Stop {
    published
        .and_then(Published::current)
        .or_else(|| profile.map(Stop::from_profile))
        .unwrap_or(Stop::Balanced)
}

/// Which CPU scheduler runs and why, for the stop now under `settings`.
pub fn scheduler(settings: &Settings, machine: &Machine) -> scheduler::Report {
    let published = runtime::file().and_then(|file| Published::load(&file));
    let profile = if published.is_some() { None } else { profile() };
    let plan = Plan::for_stop(stop_now(published.as_ref(), profile), settings);
    scheduler::report(&plan, machine, scheduler_unit().as_ref(), lavd_exit().as_deref())
}

/// The scheduler's unit, from `systemctl show`, which needs no root; `None` when systemctl can't
/// be run.
pub fn scheduler_unit() -> Option<UnitState> {
    let output = Command::new("systemctl")
        .args([
            "show",
            scheduler::UNIT,
            "--property=LoadState,ActiveState,SubState,Result,NRestarts",
        ])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| UnitState::parse(&String::from_utf8_lossy(&output.stdout)))
}

/// scx_lavd's last exit message this boot (`EXIT: runtime error (…)`), which it prints when the
/// kernel ejects it; `None` when there is none or the journal can't be read (reading the system
/// journal takes the `adm` or `systemd-journal` group).
pub fn lavd_exit() -> Option<String> {
    let output = Command::new("journalctl")
        .args([
            "--boot",
            "--unit",
            scheduler::UNIT,
            "--output=cat",
            "--lines=200",
            "--quiet",
            "--no-pager",
        ])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    scheduler::last_exit(&String::from_utf8_lossy(&output.stdout))
}

/// power-profiles-daemon's profile, `None` when it can't be asked.
pub fn profile() -> Option<Profile> {
    let bus = zbus::blocking::Connection::system().ok()?;
    let proxy = zbus::blocking::Proxy::new(&bus, PPD_SERVICE, PPD_PATH, PPD_SERVICE).ok()?;
    let name: String = proxy.get_property("ActiveProfile").ok()?;
    Profile::from_name(&name)
}

/// The laptop panel's refresh rate and VRR, as Hyprland says.
pub fn laptop_panel() -> Option<(f64, bool)> {
    let output = Command::new("hyprctl")
        .args(["-j", "monitors"])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    let monitors: Value = serde_json::from_slice(&output.stdout).ok()?;
    let panel = monitors
        .as_array()?
        .iter()
        .find(|monitor| monitor["name"].as_str().is_some_and(|name| name.starts_with("eDP-")))?;
    Some((panel["refreshRate"].as_f64()?, panel["vrr"].as_bool()?))
}

/// Whether the Wi-Fi `interface` saves power, as `iw` says; it needs no root to ask.
pub fn wifi_power_save(interface: &str) -> Option<bool> {
    // `iw` is in /usr/sbin, which a user's PATH may leave out.
    let iw = if Path::new("/usr/sbin/iw").exists() {
        "/usr/sbin/iw"
    } else {
        "iw"
    };
    let output = Command::new(iw)
        .args(["dev", interface, "get", "power_save"])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    match String::from_utf8_lossy(&output.stdout).trim() {
        "Power save: on" => Some(true),
        "Power save: off" => Some(false),
        _ => None,
    }
}

/// Whether the machine has the file indexer, `localsearch`.
pub fn has_indexer() -> bool {
    Command::new("sh")
        .args(["-c", "command -v localsearch"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

/// Whether the file indexer is held paused; `None` without one.
pub fn indexer_paused() -> Option<bool> {
    has_indexer().then(|| unit_active(QUIET_UNIT))
}

/// Whether the systemd user unit `unit`'s service runs.
pub fn unit_active(unit: &str) -> bool {
    Command::new("systemctl")
        .args(["--user", "is-active", "--quiet", &format!("{unit}.service")])
        .stdin(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}
