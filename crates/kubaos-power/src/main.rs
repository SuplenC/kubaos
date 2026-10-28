//! `kubaos-power`: the power stops' service and command. `kubaos power …` runs it.
//!
//! ```text
//! kubaos-power serve                      the session service (home/power.nix's user unit)
//! kubaos-power set STOP                   super-saver, power-saver, balanced, performance or
//!                                         max-performance, remembered for the power source
//! kubaos-power undo                       the low-battery drop's Undo
//! kubaos-power status                     what the stop does, every row read live
//! kubaos-power measure [--seconds N]      the battery's draw and the top wakeups, as a report
//! kubaos-power panel-power-saving on|off  AMD's panel power saving in the saver stops
//! kubaos-power charge-limit 80|90|100     how far the battery charges
//! kubaos-power low-battery power-saver|super-saver|off|PERCENT
//! kubaos-power panel STOP 60hz|vrr|native what the laptop panel does in a stop
//! kubaos-power apply                      put a hand edit of power.toml to work
//! kubaos-power scheduler [on|off]         which CPU scheduler runs and why; LAVD on or off
//! ```
//!
//! A change to the settings says what it came to on stdout and in a toast, and the service puts
//! it to work at once. A refusal says why on stderr and in a toast, exit 1; a usage error exits 2.
//!
//! # Owns
//!
//! - The command line, the service's loop and threads (the bus, the scheduler's unit, the socket,
//!   the toasts), and doing what [`Control`] decides: power-profiles-daemon, the root helper,
//!   Hyprland, the indexer.
//!
//! # Not here
//!
//! - Every decision and every value: the library's modules.

use std::env;
use std::error::Error;
use std::fs;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::os::unix::net::UnixListener;
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitCode;
use std::process::Stdio;
use std::sync::mpsc;
use std::sync::mpsc::RecvTimeoutError;
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;
use std::time::Instant;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use kubaos_power::control::Control;
use kubaos_power::control::Drop;
use kubaos_power::control::Effect;
use kubaos_power::control::Input;
use kubaos_power::control::Recheck;
use kubaos_power::control::Settle;
use kubaos_power::control::Toast;
use kubaos_power::live;
use kubaos_power::live::PPD_PATH;
use kubaos_power::live::PPD_SERVICE;
use kubaos_power::live::QUIET_UNIT;
use kubaos_power::machine::Machine;
use kubaos_power::measure;
use kubaos_power::memory;
use kubaos_power::memory::Memory;
use kubaos_power::panel;
use kubaos_power::plan::Plan;
use kubaos_power::runtime;
use kubaos_power::runtime::Published;
use kubaos_power::scheduler;
use kubaos_power::scheduler::UnitState;
use kubaos_power::settings;
use kubaos_power::settings::ChargeLimit;
use kubaos_power::settings::CpuScheduler;
use kubaos_power::settings::PanelMode;
use kubaos_power::settings::Settings;
use kubaos_power::status;
use kubaos_power::stop::Profile;
use kubaos_power::stop::Source;
use kubaos_power::stop::Stop;
use kubaos_power::writes;

const USAGE: &str = "\
usage: kubaos power set super-saver|power-saver|balanced|performance|max-performance
       kubaos power undo | status | apply
       kubaos power measure [--seconds N]
       kubaos power panel-power-saving on|off
       kubaos power charge-limit 80|90|100
       kubaos power low-battery power-saver|super-saver|off|PERCENT
       kubaos power panel STOP 60hz|vrr|native
       kubaos power scheduler [on|off]

set       move the power slider to a stop; each power source remembers its own
undo      undo the low-battery drop
status    what the stop does, every layer read live, and what this machine lacks
measure   the battery's average draw and what woke the machine most, as a report (default 120 s)
panel-power-saving  AMD's panel power saving in the saver stops; off for colour-accurate work
charge-limit        how far the battery charges, in percent
low-battery         where the stop drops at the low-battery level, or that level in percent
panel     the laptop panel in a stop: 60 Hz, the native rate with VRR, or native
apply     put a hand edit of the settings file to work
scheduler which CPU scheduler runs and why; on runs LAVD (sched_ext) in each stop's mode, off
          leaves Linux's own (EEVDF)

Settings: ~/.config/kubaos/desktop/power/power.toml";

/// The root helper `system/files/power/kubaos-power-write`, which `system/30-hardware.sh` installs.
const HELPER: &str = "/usr/local/libexec/kubaos-power-write";
/// Ubuntu's pkexec, by path: polkit's own, never one earlier on the PATH.
const PKEXEC: &str = "/usr/bin/pkexec";
const UPOWER_SERVICE: &str = "org.freedesktop.UPower";
const UPOWER_PATH: &str = "/org/freedesktop/UPower";
const DISPLAY_DEVICE: &str = "/org/freedesktop/UPower/devices/DisplayDevice";
const SYSTEMD: &str = "org.freedesktop.systemd1";

#[expect(
    clippy::print_stdout,
    reason = "The help, the scheduler's state and the Undo's outcome are the command's output, for whoever asked."
)]
fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    match args.as_slice() {
        ["serve"] => {
            if log::set_logger(&Journal).is_ok() {
                log::set_max_level(log::LevelFilter::Info);
            }
            report(serve())
        }
        ["set", stop] => report(set(stop)),
        ["undo"] => report(ask("undo").map(|reply| println!("{reply}"))),
        ["status"] => {
            status();
            ExitCode::SUCCESS
        }
        ["measure"] => {
            measure(measure::DEFAULT_SECONDS);
            ExitCode::SUCCESS
        }
        ["measure", "--seconds", seconds] | ["measure", seconds] => match seconds.parse() {
            Ok(seconds) if seconds > 0 => {
                measure(seconds);
                ExitCode::SUCCESS
            }
            _ => usage(),
        },
        ["panel-power-saving", on @ ("on" | "off")] => report_said(change(
            |path| settings::set_panel_power_saving(path, *on == "on"),
            |_| {
                if *on == "on" {
                    "Panel power saving is on in Super Saver and Power Saver.".to_owned()
                } else {
                    "Panel power saving is off: colours stay true in every stop.".to_owned()
                }
            },
        )),
        ["charge-limit", percent] => report_said(charge_limit(percent)),
        ["low-battery", what] => report_said(low_battery(what)),
        ["panel", stop, mode] => report_said(set_panel(stop, mode)),
        ["apply"] => report_said(apply_file()),
        ["scheduler"] | ["scheduler", "status"] => {
            let (settings, _) = load_settings();
            print!("{}", scheduler::text(&live::scheduler(&settings, &Machine::real())));
            ExitCode::SUCCESS
        }
        ["scheduler", on @ ("on" | "off")] => report_said(set_scheduler(*on == "on")),
        ["-h" | "--help" | "help"] => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        _ => usage(),
    }
}

#[expect(
    clippy::print_stderr,
    reason = "A usage error goes to whoever ran the command, on stderr."
)]
fn usage() -> ExitCode {
    eprintln!("{USAGE}");
    ExitCode::from(2)
}

/// A refusal on stderr, exit 1.
#[expect(
    clippy::print_stderr,
    reason = "The refusal is the command's answer to whoever ran it."
)]
fn report(result: Result<(), String>) -> ExitCode {
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("kubaos-power: {message}");
            ExitCode::FAILURE
        }
    }
}

/// What a settings change came to on stdout and in a toast; a refusal on stderr and in a toast.
#[expect(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "What a change came to, or the refusal, is the command's answer, beside its toast."
)]
fn report_said(result: Result<String, String>) -> ExitCode {
    match result {
        Ok(said) => {
            println!("{said}");
            notify("Power", &said, "low", None);
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("kubaos-power: {message}");
            notify("Power", &message, "critical", None);
            ExitCode::FAILURE
        }
    }
}

/// `err` and its causes, a `: ` between each.
fn chain(err: &dyn Error) -> String {
    let mut out = err.to_string();
    let mut source = err.source();
    while let Some(cause) = source {
        out.push_str(": ");
        out.push_str(&cause.to_string());
        source = cause.source();
    }
    out
}

/// Shows a toast; with `action`, waits for it and says whether it was clicked.
fn notify(title: &str, body: &str, urgency: &str, action: Option<&str>) -> bool {
    let mut command = Command::new("notify-send");
    command.args(["-a", "Power", "-i", "battery", "-u", urgency]);
    if let Some(action) = action {
        command.arg(format!("--action=go={action}"));
    }
    command
        .args([title, body])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .is_ok_and(|output| String::from_utf8_lossy(&output.stdout).trim() == "go")
}

fn settings_path() -> Result<PathBuf, String> {
    settings::path().ok_or_else(|| "Neither HOME nor XDG_CONFIG_HOME is set".to_owned())
}

/// The settings, or KubaOS's defaults with what is wrong with the file.
fn load_settings() -> (Settings, Option<String>) {
    let Ok(path) = settings_path() else {
        return (Settings::default(), None);
    };
    match Settings::load(&path) {
        Ok(settings) => (settings, None),
        Err(err) => (Settings::default(), Some(chain(&err))),
    }
}

fn drop_of(settings: &Settings) -> Drop {
    Drop {
        percent: settings.low_battery,
        to: settings.low_battery_stop,
    }
}

// ----- Talking to the service -------------------------------------------------------------------

/// Sends `line` to the service and returns its answer.
fn ask(line: &str) -> Result<String, String> {
    let socket = runtime::socket().ok_or_else(|| "XDG_RUNTIME_DIR isn't set".to_owned())?;
    let mut stream = UnixStream::connect(&socket).map_err(|err| format!("the power service isn't running ({err})"))?;
    let _ = stream.set_read_timeout(Some(Duration::from_secs(10)));
    writeln!(stream, "{line}").map_err(|err| format!("talking to the power service: {err}"))?;
    let mut reply = String::new();
    BufReader::new(stream)
        .read_line(&mut reply)
        .map_err(|err| format!("the power service didn't answer: {err}"))?;
    let reply = reply.trim();
    match reply.strip_prefix("error ") {
        Some(error) => Err(error.to_owned()),
        None => Ok(reply.strip_prefix("ok ").unwrap_or(reply).to_owned()),
    }
}

#[expect(
    clippy::print_stdout,
    reason = "What the stop came to is the command's answer to whoever ran it."
)]
fn set(stop: &str) -> Result<(), String> {
    let stop = Stop::from_id(stop).ok_or_else(|| {
        format!("`{stop}` isn't a stop: super-saver, power-saver, balanced, performance or max-performance")
    })?;
    match ask(&format!("set {}", stop.id())) {
        Ok(reply) => {
            println!("{reply}");
            Ok(())
        }
        Err(problem) if problem.starts_with("the power service isn't running") => {
            // Without the service the profile still changes, so the menu's Items work on a
            // machine whose service is stopped; the extras wait for it.
            let bus = zbus::blocking::Connection::system().map_err(|err| chain(&err))?;
            set_profile(&bus, stop.profile())?;
            println!(
                "{problem}, so only power-profiles-daemon's {} profile changed",
                stop.profile().name()
            );
            Ok(())
        }
        Err(problem) => Err(problem),
    }
}

/// Tells the service the settings changed; nothing when it isn't running.
fn reload() {
    let _ = ask("reload");
}

fn change(
    edit: impl FnOnce(&Path) -> Result<Settings, settings::SettingsError>,
    said: impl FnOnce(&Settings) -> String,
) -> Result<String, String> {
    let settings = edit(&settings_path()?).map_err(|err| chain(&err))?;
    reload();
    Ok(said(&settings))
}

fn charge_limit(percent: &str) -> Result<String, String> {
    let limit = percent
        .trim_end_matches('%')
        .parse()
        .ok()
        .and_then(ChargeLimit::from_percent)
        .ok_or_else(|| format!("`{percent}` isn't a charge limit: 80, 90 or 100"))?;
    let charge = Machine::real().charge();
    change(
        |path| settings::set_charge_limit(path, limit),
        |_| match (limit, charge) {
            (_, None) => format!(
                "Charge limit {} % saved, but this laptop has no charge limit to set.",
                limit.percent()
            ),
            (ChargeLimit::Full, _) => "The battery charges full.".to_owned(),
            (limit, _) => format!("The battery stops charging at {} %.", limit.percent()),
        },
    )
}

fn low_battery(what: &str) -> Result<String, String> {
    if let Ok(percent) = what.trim_end_matches('%').parse::<u8>() {
        return change(
            |path| settings::set_low_battery(path, percent),
            |settings| match settings.low_battery_stop {
                Some(stop) => format!("At {percent} % on battery, the stop drops to {stop}."),
                None => format!("The drop's level is {percent} %, but the drop is off."),
            },
        );
    }
    let stop = settings::parse_low_battery_stop(what).map_err(|err| chain(&err))?;
    change(
        |path| settings::set_low_battery_stop(path, stop),
        |settings| match stop {
            Some(stop) => format!(
                "At {} % on battery, the stop drops to {stop}, with an Undo.",
                settings.low_battery
            ),
            None => "The low-battery drop is off: the stop stays where you put it.".to_owned(),
        },
    )
}

fn set_panel(stop: &str, mode: &str) -> Result<String, String> {
    let stop = Stop::from_id(stop).ok_or_else(|| format!("`{stop}` isn't a stop"))?;
    let mode = PanelMode::from_name(mode).ok_or_else(|| format!("`{mode}` isn't a panel mode: 60hz, vrr or native"))?;
    change(
        |path| settings::set_panel(path, stop, mode),
        |_| {
            let words = match mode {
                PanelMode::Fixed60 => "60 Hz",
                PanelMode::Vrr => "its native rate with VRR",
                PanelMode::Native => "its native rate",
            };
            format!("In {stop}, the laptop panel runs at {words}.")
        },
    )
}

/// Turns LAVD on or off. On also lets a LAVD that crashed until systemd gave up on it start again.
fn set_scheduler(on: bool) -> Result<String, String> {
    let choice = if on { CpuScheduler::Lavd } else { CpuScheduler::Kernel };
    settings::set_scheduler(&settings_path()?, choice).map_err(|err| chain(&err))?;
    let asked = ask(if on { "retry-scheduler" } else { "reload" });
    let machine = Machine::real();
    let said = match (on, machine.sched_ext().is_some(), machine.has_lavd()) {
        (false, ..) => "LAVD is off: Linux's own scheduler (EEVDF) runs.".to_owned(),
        (true, false, _) => "LAVD is on, but this kernel has no sched_ext, so Linux's own scheduler stays.".to_owned(),
        (true, true, false) => "LAVD is on, but scx_lavd isn't installed yet: `kubaos update` installs it.".to_owned(),
        (true, true, true) => "LAVD is on, in each stop's mode. If it fails, the kernel goes back to its own \
                               scheduler by itself; `kubaos power scheduler` says which runs and why."
            .to_owned(),
    };
    match asked {
        Ok(_) => Ok(said),
        Err(problem) if problem.starts_with("the power service isn't running") => {
            Ok(format!("{said} It takes effect when the power service runs."))
        }
        Err(problem) => Err(problem),
    }
}

fn apply_file() -> Result<String, String> {
    Settings::load(&settings_path()?).map_err(|err| chain(&err))?;
    ask("reload")?;
    Ok("The power settings are in use.".to_owned())
}

// ----- status and measure -----------------------------------------------------------------------

#[expect(
    clippy::print_stdout,
    reason = "The status is the command's output, for the person or the menu that asked."
)]
fn status() {
    let (settings, problem) = load_settings();
    let now = live::read(&settings, problem.as_deref(), &Machine::real());
    print!("{}", status::text(&now.header, &now.rows));
}

#[expect(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "The report, and the note to leave the machine alone while it measures, are for the person who ran it."
)]
fn measure(seconds: u64) {
    let machine = Machine::real();
    let stop = runtime::file()
        .and_then(|file| Published::load(&file))
        .as_ref()
        .and_then(Published::current)
        .or_else(|| live::profile().map(Stop::from_profile))
        .unwrap_or(Stop::Balanced);
    eprintln!("Measuring {stop} for {seconds} s. Leave the machine as it is…");
    let report = measure::run(
        &machine,
        stop.title(),
        Duration::from_secs(seconds),
        Duration::from_secs(1),
        &mut thread::sleep,
    );
    let text = report.text();
    print!("{text}");
    if let Some(dir) = memory::dir() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |since| since.as_secs());
        let path = dir.join("measure").join(format!("{now}-{}.txt", stop.id()));
        match memory::write_atomic(&path, &text) {
            Ok(()) => println!("\nSaved to {}", path.display()),
            Err(err) => eprintln!("kubaos-power: couldn't save the report to {}: {err}", path.display()),
        }
    }
}

// ----- The bus ----------------------------------------------------------------------------------

fn ppd(bus: &zbus::blocking::Connection) -> zbus::Result<zbus::blocking::Proxy<'static>> {
    zbus::blocking::Proxy::new(bus, PPD_SERVICE, PPD_PATH, PPD_SERVICE)
}

fn set_profile(bus: &zbus::blocking::Connection, profile: Profile) -> Result<(), String> {
    let proxy = ppd(bus).map_err(|err| chain(&err))?;
    proxy
        .set_property("ActiveProfile", profile.name())
        .map_err(|err| format!("power-profiles-daemon refused {}: {}", profile.name(), chain(&err)))
}

// ----- The service ------------------------------------------------------------------------------

/// What the service's loop hears.
enum Event {
    /// Where things stood when the bus was first asked.
    Start {
        profile: Option<Profile>,
        on_battery: bool,
        percent: Option<f64>,
    },
    /// power-profiles-daemon's profile, and whether a program holds it.
    Profile(Profile, bool),
    OnBattery(bool),
    /// A GPU, a screen or a USB device came, went or changed (udev).
    Devices,
    Percent(f64),
    /// The scheduler's unit changed state.
    Scheduler(UnitState),
    /// A line on the socket, and where its answer goes.
    Ask(String, Sender<String>),
    Undo,
    /// A bus thread ended: the bus or its service is gone.
    Lost(String),
}

/// The service: follows power-profiles-daemon and UPower, answers on the socket, and puts each
/// stop to work.
fn serve() -> Result<(), String> {
    let runtime = runtime::dir().ok_or_else(|| "XDG_RUNTIME_DIR isn't set".to_owned())?;
    let bus =
        zbus::blocking::Connection::system().map_err(|err| format!("connecting to the system bus: {}", chain(&err)))?;
    let (events, received) = mpsc::channel();
    follow_bus(&bus, &events)?;
    follow_devices(events.clone());
    follow_scheduler(&bus, events.clone());
    listen(&runtime, events.clone())?;
    let mut service = Service::new(bus, runtime, events);
    service.run(&received)
}

/// Starts the threads that follow power-profiles-daemon's profile, UPower's power source and the
/// battery's charge. Each sends the value it starts at with [`Event::Start`], then each change.
fn follow_bus(bus: &zbus::blocking::Connection, events: &Sender<Event>) -> Result<(), String> {
    let ppd = ppd(bus).map_err(|err| chain(&err))?;
    let upower =
        zbus::blocking::Proxy::new(bus, UPOWER_SERVICE, UPOWER_PATH, UPOWER_SERVICE).map_err(|err| chain(&err))?;
    let display = zbus::blocking::Proxy::new(bus, UPOWER_SERVICE, DISPLAY_DEVICE, "org.freedesktop.UPower.Device")
        .map_err(|err| chain(&err))?;
    // The streams start before the first values are read, so no change falls between the two.
    let profiles = ppd.receive_property_changed::<String>("ActiveProfile");
    let sources = upower.receive_property_changed::<bool>("OnBattery");
    let percents = display.receive_property_changed::<f64>("Percentage");
    let profile = ppd
        .get_property::<String>("ActiveProfile")
        .ok()
        .and_then(|name| Profile::from_name(&name));
    let on_battery = upower.get_property::<bool>("OnBattery").unwrap_or(false);
    let percent = display.get_property::<f64>("Percentage").ok();
    let _ = events.send(Event::Start {
        profile,
        on_battery,
        percent,
    });

    let sent = events.clone();
    thread::spawn(move || {
        let mut last = profile;
        for change in profiles {
            let Some(profile) = change.get().ok().and_then(|name| Profile::from_name(&name)) else {
                continue;
            };
            if last != Some(profile) {
                last = Some(profile);
                let _ = sent.send(Event::Profile(profile, holds(&ppd)));
            }
        }
        let _ = sent.send(Event::Lost("power-profiles-daemon".to_owned()));
    });
    let sent = events.clone();
    thread::spawn(move || {
        let mut last = on_battery;
        for change in sources {
            let Ok(on_battery) = change.get() else {
                continue;
            };
            if on_battery != last {
                last = on_battery;
                let _ = sent.send(Event::OnBattery(on_battery));
            }
        }
        let _ = sent.send(Event::Lost("UPower".to_owned()));
    });
    let sent = events.clone();
    thread::spawn(move || {
        for change in percents {
            if let Ok(percent) = change.get() {
                let _ = sent.send(Event::Percent(percent));
            }
        }
    });
    Ok(())
}

/// Whether a program holds power-profiles-daemon's profile (`ActiveProfileHolds`, GameMode's and
/// the like); an answer that doesn't read counts as none.
fn holds(ppd: &zbus::blocking::Proxy<'static>) -> bool {
    ppd.get_property::<zbus::zvariant::OwnedValue>("ActiveProfileHolds")
        .is_ok_and(|holds| matches!(&*holds, zbus::zvariant::Value::Array(array) if !array.is_empty()))
}

/// Follows udev's events for GPUs, screens and USB devices through `udevadm monitor`, so a device
/// plugged in gets the stop's settings, and the GPU's level is put back after
/// power-profiles-daemon resets it on a screen's arrival. Without `udevadm` the stop's settings
/// reach new devices at the next stop.
fn follow_devices(events: Sender<Event>) {
    thread::spawn(move || {
        let child = Command::new("udevadm")
            .args(["monitor", "--udev", "--subsystem-match=drm", "--subsystem-match=usb"])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn();
        let Some(stdout) = child.ok().and_then(|mut child| child.stdout.take()) else {
            log::warn!("couldn't follow udev; new devices get the stop's settings at the next stop");
            return;
        };
        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
            let words: Vec<&str> = line.split_whitespace().collect();
            if words.first() == Some(&"UDEV") && matches!(words.get(2), Some(&("add" | "change" | "bind"))) {
                let _ = events.send(Event::Devices);
            }
        }
    });
}

/// Follows the scheduler's unit on systemd's bus, so the service hears the moment LAVD falls back
/// to the kernel's scheduler: its state at the start, then at every change. Without the bus,
/// `status` still says it.
fn follow_scheduler(bus: &zbus::blocking::Connection, events: Sender<Event>) {
    let bus = bus.clone();
    thread::spawn(move || {
        let follow = || -> zbus::Result<()> {
            let manager = zbus::blocking::Proxy::new(
                &bus,
                SYSTEMD,
                "/org/freedesktop/systemd1",
                "org.freedesktop.systemd1.Manager",
            )?;
            // systemd sends its units' changes only while some client is subscribed.
            manager.call_method("Subscribe", &())?;
            let properties =
                zbus::blocking::Proxy::new(&bus, SYSTEMD, scheduler::UNIT_PATH, "org.freedesktop.DBus.Properties")?;
            let changes = properties.receive_signal("PropertiesChanged")?;
            let _ = events.send(Event::Scheduler(unit_state(&properties)));
            for _ in changes {
                let _ = events.send(Event::Scheduler(unit_state(&properties)));
            }
            Ok(())
        };
        if let Err(err) = follow() {
            log::warn!(
                "couldn't follow {} ({}); LAVD's fallbacks show in status only",
                scheduler::UNIT,
                chain(&err)
            );
        }
    });
}

/// The scheduler's unit as systemd has it now, read through `properties`, its
/// `org.freedesktop.DBus.Properties`; a property that doesn't read stays empty.
fn unit_state(properties: &zbus::blocking::Proxy<'_>) -> UnitState {
    let get = |interface: &str, name: &str| {
        properties
            .call::<_, _, zbus::zvariant::OwnedValue>("Get", &(interface, name))
            .ok()
    };
    let text = |interface: &str, name: &str| {
        get(interface, name)
            .and_then(|value| String::try_from(value).ok())
            .unwrap_or_default()
    };
    UnitState {
        load: text("org.freedesktop.systemd1.Unit", "LoadState"),
        active: text("org.freedesktop.systemd1.Unit", "ActiveState"),
        sub: text("org.freedesktop.systemd1.Unit", "SubState"),
        result: text("org.freedesktop.systemd1.Service", "Result"),
        restarts: get("org.freedesktop.systemd1.Service", "NRestarts")
            .and_then(|value| u32::try_from(value).ok())
            .unwrap_or(0),
    }
}

/// Answers the socket: a line in, the loop's answer out.
fn listen(runtime: &Path, events: Sender<Event>) -> Result<(), String> {
    let path = runtime.join(runtime::SOCKET);
    let _ = fs::remove_file(&path);
    let listener = UnixListener::bind(&path).map_err(|err| format!("listening on {}: {err}", path.display()))?;
    thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let events = events.clone();
            thread::spawn(move || answer(stream, &events));
        }
    });
    Ok(())
}

fn answer(stream: UnixStream, events: &Sender<Event>) {
    let mut line = String::new();
    let Ok(clone) = stream.try_clone() else {
        return;
    };
    if BufReader::new(clone).read_line(&mut line).is_err() {
        return;
    }
    let (reply, replied) = mpsc::channel();
    if events.send(Event::Ask(line.trim().to_owned(), reply)).is_err() {
        return;
    }
    let reply = replied
        .recv_timeout(Duration::from_secs(10))
        .unwrap_or_else(|_| "error the power service is busy".to_owned());
    let mut stream = stream;
    let _ = writeln!(stream, "{reply}");
}

struct Service {
    bus: zbus::blocking::Connection,
    runtime: PathBuf,
    events: Sender<Event>,
    machine: Machine,
    settings: Settings,
    control: Option<Control>,
    settle: Settle,
    recheck: Recheck,
    /// The panel file's text last asked of Hyprland.
    panel: Option<String>,
    /// The helper is missing or refused; said once, not at every stop.
    helper_said: bool,
    /// The knobs the kernel refused for the stop in use.
    refused: writes::Refused,
    /// The scheduler's unit as last heard.
    scheduler_unit: Option<UnitState>,
    /// LAVD was just turned on: the next stop's writes let a LAVD that kept crashing start again.
    retry_scheduler: bool,
}

impl Service {
    fn new(bus: zbus::blocking::Connection, runtime: PathBuf, events: Sender<Event>) -> Service {
        let (settings, problem) = load_settings();
        if let Some(problem) = problem {
            notify(
                "Power",
                &format!("{problem}. KubaOS's power settings apply until it's fixed."),
                "critical",
                None,
            );
        }
        Service {
            bus,
            runtime,
            events,
            machine: Machine::real(),
            settings,
            control: None,
            settle: Settle::default(),
            recheck: Recheck::default(),
            panel: None,
            helper_said: false,
            refused: writes::Refused::default(),
            scheduler_unit: None,
            retry_scheduler: false,
        }
    }

    fn run(&mut self, received: &mpsc::Receiver<Event>) -> Result<(), String> {
        loop {
            let now = Instant::now();
            let wait = [self.settle.wait(now), self.recheck.wait(now)]
                .into_iter()
                .flatten()
                .min()
                .unwrap_or(Duration::from_hours(1));
            match received.recv_timeout(wait) {
                Ok(Event::Lost(what)) => return Err(format!("lost {what} on the system bus")),
                Ok(event) => self.hear(event),
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => return Ok(()),
            }
            if let Some(source) = self.settle.due(Instant::now()) {
                self.input(Input::Source(source));
            }
            if self.recheck.due(Instant::now()) {
                self.write_plan(false);
            }
        }
    }

    fn hear(&mut self, event: Event) {
        match event {
            Event::Start {
                profile,
                on_battery,
                percent,
            } => {
                let memory = memory::path().map(|path| Memory::load(&path)).unwrap_or_default();
                let source = Source::from_on_battery(on_battery);
                let (mut control, effects) = Control::start(memory, source, profile, drop_of(&self.settings));
                let mut more = Vec::new();
                if let Some(percent) = percent {
                    more = control.handle(Input::Percent(percent));
                }
                self.control = Some(control);
                // The panel is asked of Hyprland at every start: a new session has none yet.
                self.panel = None;
                self.act(effects.into_iter().chain(more).collect());
            }
            Event::Profile(profile, held) => {
                self.input(Input::Profile { profile, held });
                // ppd rewrites the GPU's level and the CPU's boost on every profile it activates.
                self.recheck.ask(Instant::now());
            }
            Event::Devices => self.recheck.ask(Instant::now()),
            Event::OnBattery(on_battery) => {
                if let Some(control) = &self.control {
                    self.settle
                        .observe(Source::from_on_battery(on_battery), control.source(), Instant::now());
                }
            }
            Event::Percent(percent) => self.input(Input::Percent(percent)),
            Event::Scheduler(now) => self.scheduler_changed(now),
            Event::Undo => self.input(Input::Undo),
            Event::Ask(line, reply) => {
                let answer = self.ask(&line);
                let _ = reply.send(answer);
            }
            Event::Lost(_) => {}
        }
    }

    fn ask(&mut self, line: &str) -> String {
        let words: Vec<&str> = line.split_whitespace().collect();
        match words.as_slice() {
            ["set", stop] => match Stop::from_id(stop) {
                Some(stop) => {
                    self.input(Input::Set(stop));
                    format!("ok {stop}")
                }
                None => format!("error `{stop}` isn't a stop"),
            },
            ["undo"] => {
                let dropped = self.control.as_ref().and_then(Control::dropped_from).is_some();
                self.input(Input::Undo);
                if dropped {
                    let stop = self.control.as_ref().map_or(Stop::Balanced, Control::stop);
                    format!("ok Back to {stop}")
                } else {
                    "ok There's no low-battery drop to undo".to_owned()
                }
            }
            [line @ ("reload" | "retry-scheduler")] => {
                let (settings, problem) = load_settings();
                if let Some(problem) = problem {
                    return format!("error {problem}");
                }
                self.settings = settings;
                self.retry_scheduler = *line == "retry-scheduler";
                if let Some(control) = &mut self.control {
                    control.set_drop(drop_of(&self.settings));
                    let stop = control.stop();
                    self.act(vec![Effect::Apply(stop)]);
                }
                "ok".to_owned()
            }
            _ => format!("error `{line}` isn't something the power service does"),
        }
    }

    fn input(&mut self, input: Input) {
        let Some(control) = &mut self.control else {
            return;
        };
        let effects = control.handle(input);
        self.act(effects);
    }

    fn act(&mut self, effects: Vec<Effect>) {
        for effect in effects {
            match effect {
                Effect::SetProfile(profile) => {
                    if let Err(problem) = set_profile(&self.bus, profile) {
                        log::error!("{problem}");
                    }
                }
                Effect::Apply(stop) => self.apply(stop),
                Effect::Remember(memory) => {
                    if let Some(path) = memory::path()
                        && let Err(err) = memory.save(&path)
                    {
                        log::error!("couldn't write {}: {err}", path.display());
                    }
                }
                Effect::Toast(toast) => self.toast(toast),
            }
        }
        self.publish();
    }

    /// Puts every row of `stop` to work: what shows first (the shell, the panel), then the rest.
    fn apply(&mut self, stop: Stop) {
        let plan = Plan::for_stop(stop, &self.settings);
        // A new stop, or new settings, asks the kernel again.
        self.refused.clear();
        self.publish();
        self.apply_panel(&plan);
        Self::quiet(plan.quiet_background);
        self.write_plan(true);
    }

    /// Hears the scheduler's unit change, and says when LAVD fell back to the kernel's scheduler:
    /// in the journal, and in a toast.
    fn scheduler_changed(&mut self, now: UnitState) {
        let before = self.scheduler_unit.take();
        let fallback = if self.settings.scheduler == CpuScheduler::Lavd {
            scheduler::fallback(before.as_ref(), &now)
        } else {
            None
        };
        if let Some(fallback) = fallback {
            let (title, body, urgency) = fallback.toast();
            log::warn!("LAVD fell back to the kernel's scheduler ({}): {body}", now.words());
            thread::spawn(move || notify(title, &body, urgency, None));
        }
        self.scheduler_unit = Some(now);
    }

    /// The stop's writes that the machine lacks now: all of them at a stop (`at_stop`), the
    /// scheduler last, as restarting it takes longest; on a recheck, the ones a new device or
    /// power-profiles-daemon undid, never the scheduler.
    fn write_plan(&mut self, at_stop: bool) {
        let Some(stop) = self.control.as_ref().map(Control::stop) else {
            return;
        };
        let plan = Plan::for_stop(stop, &self.settings);
        let mut writes = writes::for_plan(&plan, &self.settings, &self.machine, &live::wifi_power_save);
        if at_stop {
            let retry = std::mem::take(&mut self.retry_scheduler);
            writes.extend(scheduler::writes(&plan, &self.machine, retry));
        }
        let writes = self.refused.filter(writes);
        if !writes.is_empty() {
            self.write(&writes);
        }
    }

    fn apply_panel(&mut self, plan: &Plan) {
        let target = panel::Target::of(plan.panel, self.machine.seamless_refresh());
        let text = target.text();
        if self.panel.as_deref() == Some(text.as_str()) {
            return;
        }
        let path = panel::path(&self.runtime);
        if let Err(err) = memory::write_atomic(&path, &text) {
            log::error!("couldn't write {}: {err}", path.display());
            return;
        }
        let Some(signature) = running_hyprland(&self.runtime) else {
            return;
        };
        let asked = Command::new("hyprctl")
            .args(["-i", &signature, "eval", panel::EVAL])
            .stdin(Stdio::null())
            .output();
        match asked {
            Ok(output) if output.status.success() => self.panel = Some(text),
            Ok(output) => log::warn!(
                "Hyprland didn't take the panel: {}",
                String::from_utf8_lossy(&output.stdout).trim()
            ),
            Err(err) => log::error!("couldn't run hyprctl: {err}"),
        }
    }

    /// Holds the file indexer paused while `quiet`, through `localsearch inhibit` in a transient
    /// unit, so stopping the unit lets it go and a crash of this service doesn't leave it paused.
    fn quiet(quiet: bool) {
        if !live::has_indexer() {
            return;
        }
        let held = live::unit_active(QUIET_UNIT);
        if quiet && !held {
            let _ = Command::new("systemd-run")
                .args([
                    "--user",
                    "--quiet",
                    "--collect",
                    &format!("--unit={QUIET_UNIT}"),
                    "--",
                    "localsearch",
                    "inhibit",
                    "sleep",
                    "infinity",
                ])
                .stdin(Stdio::null())
                .status();
        } else if !quiet && held {
            let _ = Command::new("systemctl")
                .args(["--user", "stop", &format!("{QUIET_UNIT}.service")])
                .stdin(Stdio::null())
                .status();
        }
    }

    /// The writes, through the root helper. pkexec runs without its own agent: an admin at the
    /// machine needs no password (`system/files/polkit/50-kubaos-power.rules`).
    fn write(&mut self, writes: &[writes::Write]) {
        if !Path::new(HELPER).exists() {
            if !self.helper_said {
                self.helper_said = true;
                log::warn!("{HELPER} isn't installed (kubaos update installs it); only the profile changes");
            }
            return;
        }
        let output = Command::new(PKEXEC)
            .arg("--disable-internal-agent")
            .arg(HELPER)
            .args(writes.iter().map(writes::Write::arg))
            .stdin(Stdio::null())
            .output();
        match output {
            Ok(output) if output.status.success() => {}
            Ok(output) => {
                let said = String::from_utf8_lossy(&output.stderr);
                for line in said.lines().filter(|line| !line.trim().is_empty()) {
                    log::warn!("{line}");
                }
                self.refused.record(&String::from_utf8_lossy(&output.stdout));
                self.publish();
            }
            Err(err) => log::error!("couldn't run pkexec: {err}"),
        }
    }

    fn toast(&self, toast: Toast) {
        match toast {
            Toast::Followed { source, stop } => {
                let title = match source {
                    Source::Battery => "On battery",
                    Source::Ac => "Plugged in",
                };
                let body = format!("{stop}, as you left it {}.", source.words());
                thread::spawn(move || notify(title, &body, "low", None));
            }
            Toast::Dropped { to, from, percent } => {
                let events = self.events.clone();
                thread::spawn(move || {
                    let title = format!("Battery at {percent} %");
                    let body = format!("{to} until you plug in. Undo goes back to {from}.");
                    if notify(&title, &body, "normal", Some("Undo")) {
                        let _ = events.send(Event::Undo);
                    }
                });
            }
        }
    }

    /// Writes what the shell reads.
    fn publish(&self) {
        let Some(control) = &self.control else { return };
        let quiet = Plan::for_stop(control.stop(), &self.settings).quiet_background;
        let mut published = Published::new(control.stop(), control.source(), control.dropped_from(), quiet);
        published.refused = self.refused.args().to_vec();
        let path = self.runtime.join(runtime::FILE);
        if Published::load(&path).as_ref() == Some(&published) {
            return;
        }
        if let Err(err) = memory::write_atomic(&path, &published.text()) {
            log::error!("couldn't write {}: {err}", path.display());
        }
    }
}

/// The instance signature of this user's running Hyprland: its lock file under the runtime dir
/// names its pid, which `/proc` still lists. A crashed one left behind doesn't count.
fn running_hyprland(runtime: &Path) -> Option<String> {
    let instances = fs::read_dir(runtime.join("hypr")).ok()?;
    instances.flatten().find_map(|instance| {
        let text = fs::read_to_string(instance.path().join("hyprland.lock")).ok()?;
        let pid: u32 = text.lines().next()?.trim().parse().ok()?;
        Path::new("/proc")
            .join(pid.to_string())
            .is_dir()
            .then(|| instance.file_name().to_string_lossy().into_owned())
    })
}

/// The service's log: a line on stderr each, with the priority prefix the journal reads
/// (`sd-daemon(3)`), as `kubaos-proxy`'s service writes it.
struct Journal;

impl log::Log for Journal {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        metadata.level() <= log::Level::Info
    }

    fn log(&self, record: &log::Record<'_>) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let priority = match record.level() {
            log::Level::Error => 3,
            log::Level::Warn => 4,
            log::Level::Info => 6,
            log::Level::Debug | log::Level::Trace => 7,
        };
        let _ = writeln!(std::io::stderr(), "<{priority}>{}", record.args());
    }

    fn flush(&self) {}
}
