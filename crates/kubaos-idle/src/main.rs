//! `kubaos-idle`: the idle settings, Keep awake and the scheduled lock, suspend, shut down or
//! reboot, from the Command menu, the keybinds and a terminal.
//!
//! ```text
//! kubaos-idle status                        every setting, Keep awake and a scheduled action
//! kubaos-idle set lock|screens-off|suspend WAIT [battery|ac]
//!                                           after how long idle: 90s, 2m, 1h30m or never; both
//!                                           power sources unless one is named
//! kubaos-idle set lid suspend|lock|nothing [battery|ac]
//! kubaos-idle reset                         KubaOS's defaults again
//! kubaos-idle apply                         put a hand edit of the file to work
//! kubaos-idle awake DURATION|on|off|toggle  Keep awake for 1h, until turned off, or no more
//! kubaos-idle schedule lock|suspend|shutdown|reboot in DURATION | at TIME
//!                                           once at a time, idle or not: 10m, 20:00,
//!                                           friday 17:00; `lock in|at` is short for the lock
//! kubaos-idle schedule cancel               (or `lock cancel`) whichever is scheduled
//! kubaos-idle write                         hypridle's listeners alone (home activation)
//! kubaos-idle session                       at login: the listeners, the lid, then exec hypridle
//! kubaos-idle lid closed                    Hyprland's lid bind
//! ```
//!
//! A change says what it came to on stdout and in a toast, and applies at once: hypridle restarts
//! in the session through `dot-session start-idle`, and the lid's holder starts or stops. A refusal
//! says why on stderr and in a toast, exit 1; a usage error exits 2.
//!
//! # Owns
//!
//! - The command line, running the programs, the toasts, and the holders' loops (`awake hold`, `lid
//!   hold`, which run as systemd user units under `systemd-inhibit`).
//!
//! # Not here
//!
//! - The settings, the listeners, the lid's decision and the states: the library's modules.

#![forbid(unsafe_code)]

use std::env;
use std::error::Error;
use std::fmt::Write as _;
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitCode;
use std::process::Stdio;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use jiff::Timestamp;
use jiff::Zoned;
use jiff::tz::TimeZone;
use kubaos_idle::awake;
use kubaos_idle::hypridle;
use kubaos_idle::lid;
use kubaos_idle::machine;
use kubaos_idle::machine::LidState;
use kubaos_idle::schedule;
use kubaos_idle::schedule::Action;
use kubaos_idle::settings;
use kubaos_idle::settings::LidAction;
use kubaos_idle::settings::Settings;
use kubaos_idle::settings::Sources;
use kubaos_idle::settings::Timer;
use kubaos_idle::settings::Wait;
use kubaos_idle::words;

const USAGE: &str = "\
usage: kubaos-idle status
       kubaos-idle set lock|screens-off|suspend WAIT [battery|ac]
       kubaos-idle set lid suspend|lock|nothing [battery|ac]
       kubaos-idle reset | apply
       kubaos-idle awake DURATION|on|off|toggle
       kubaos-idle schedule lock|suspend|shutdown|reboot in DURATION | at TIME
       kubaos-idle schedule cancel
       kubaos-idle write | session | lid closed

set     after how long idle the screen locks, the screens go off, or it suspends: 90s, 2m,
        1h30m, 5h or never; what closing the lid does. Both power sources unless battery or ac
        follows. Docked, the lid does nothing.
reset   KubaOS's defaults: lock 5 min, screens off 10 min, suspend 30 min on battery only, the
        lid suspends on battery
apply   put a hand edit of the settings file to work
awake   Keep awake: nothing locks, turns off or suspends by itself, for a while (1h, 2h) or until
        turned off
schedule  lock, suspend, shut down or reboot once, idle or not, in a while (10m) or at a time
        (20:00, friday 17:00); one at a time, a new one replaces it. Keep awake doesn't hold it
        back. `lock in|at|cancel` is short for `schedule lock in|at` and `schedule cancel`
write   write hypridle's listeners; session: at login, then exec hypridle
lid     closed: what Hyprland's lid bind runs

Settings: ~/.config/kubaos/desktop/idle/idle.toml";

/// Who a toast is from.
#[derive(Clone, Copy)]
enum Toast {
    Idle,
    KeepAwake,
    Scheduled(Action),
}

impl Toast {
    fn app(self) -> &'static str {
        match self {
            Toast::Idle => "Idle",
            Toast::KeepAwake => "Keep awake",
            Toast::Scheduled(Action::Lock) => "Scheduled lock",
            Toast::Scheduled(Action::Suspend) => "Scheduled suspend",
            Toast::Scheduled(Action::Shutdown) => "Scheduled shut down",
            Toast::Scheduled(Action::Reboot) => "Scheduled reboot",
        }
    }

    fn icon(self) -> &'static str {
        match self {
            Toast::Idle | Toast::KeepAwake => "preferences-desktop-screensaver",
            Toast::Scheduled(Action::Lock) => "system-lock-screen",
            Toast::Scheduled(Action::Suspend) => "system-suspend",
            Toast::Scheduled(Action::Shutdown) => "system-shutdown",
            Toast::Scheduled(Action::Reboot) => "system-reboot",
        }
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    match args.as_slice() {
        ["status"] => {
            status();
            ExitCode::SUCCESS
        }
        ["set", "lid", rest @ ..] if !rest.is_empty() => report(Toast::Idle, set_lid(rest)),
        ["set", timer, rest @ ..] if !rest.is_empty() => report(Toast::Idle, set_wait(timer, rest)),
        ["reset"] => report(Toast::Idle, reset()),
        ["apply"] => report(Toast::Idle, apply_file()),
        ["write"] => report(Toast::Idle, write_listeners()),
        ["session"] => session(),
        ["awake", "hold", until] => report(Toast::KeepAwake, awake_hold(until)),
        ["awake", "on"] => report(Toast::KeepAwake, awake_start(0)),
        ["awake", "off"] => report(Toast::KeepAwake, awake_off()),
        ["awake", "toggle"] => report(Toast::KeepAwake, awake_toggle()),
        ["awake", "for", rest @ ..] | ["awake", rest @ ..] if !rest.is_empty() => {
            report(Toast::KeepAwake, awake_for(&rest.join(" ")))
        }
        ["schedule" | "lock", "cancel"] => report(Toast::Scheduled(Action::Lock), schedule_cancel()),
        ["schedule", "fire", action, at] => report(Toast::Idle, schedule_fire(action, at)),
        ["schedule", "warn", secs, action, at] => report(Toast::Idle, schedule_warn(secs, action, at)),
        ["schedule", action, when, rest @ ..] | [action @ "lock", when, rest @ ..] if !rest.is_empty() => {
            let Some(action) = Action::from_name(action) else {
                eprintln!("{USAGE}");
                return ExitCode::from(2);
            };
            let typed = rest.join(" ");
            match *when {
                "in" => report(Toast::Scheduled(action), schedule_in(action, &typed)),
                "at" => report(Toast::Scheduled(action), schedule_at(action, &typed)),
                _ => {
                    eprintln!("{USAGE}");
                    ExitCode::from(2)
                }
            }
        }
        ["lid", "closed"] => report(Toast::Idle, lid_closed()),
        ["lid", "hold"] => report(Toast::Idle, lid_hold()),
        ["-h" | "--help" | "help"] => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!("{USAGE}");
            ExitCode::from(2)
        }
    }
}

/// A refusal on stderr and in a toast, exit 1.
fn report(from: Toast, result: Result<(), String>) -> ExitCode {
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("kubaos-idle: {message}");
            notify(from, &message, true);
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

/// Says `body` on stdout and in a toast.
fn say(from: Toast, body: &str) {
    println!("{body}");
    notify(from, body, false);
}

fn notify(from: Toast, body: &str, critical: bool) {
    let mut command = Command::new("notify-send");
    command.args(["-a", from.app(), "-i", from.icon()]);
    if critical {
        command.args(["-u", "critical"]);
    }
    let _ = command
        .args([from.app(), body])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

fn settings_path() -> Result<PathBuf, String> {
    settings::path().ok_or_else(|| "Neither HOME nor XDG_CONFIG_HOME is set".to_owned())
}

/// The settings, or KubaOS's defaults with what is wrong with the file.
fn load() -> (Settings, Option<String>) {
    let Ok(path) = settings_path() else {
        return (Settings::default(), None);
    };
    match Settings::load(&path) {
        Ok(settings) => (settings, None),
        Err(err) => (Settings::default(), Some(chain(&err))),
    }
}

fn this_program() -> String {
    env::current_exe().map_or_else(|_| "kubaos-idle".to_owned(), |exe| exe.to_string_lossy().into_owned())
}

fn status() {
    let (settings, problem) = load();
    if let Some(problem) = problem {
        println!("{problem}; KubaOS's defaults apply");
    }
    println!("{}", words::all(&settings));
    let now = Zoned::now();
    let format = clock_format();
    let runtime = kubaos_idle::runtime_dir();
    let keep = runtime
        .as_deref()
        .map(awake::path)
        .map(|path| awake::State::read(&path))
        .unwrap_or_default();
    if keep.holds(now.timestamp().as_second()) {
        if keep.until == 0 {
            println!("Keep awake: on until you turn it off");
        } else {
            println!(
                "Keep awake: on until {}",
                words::time_of_day(&at(keep.until), &now, format)
            );
        }
    }
    let scheduled = runtime
        .as_deref()
        .map(schedule::path)
        .map(|path| schedule::State::read(&path))
        .unwrap_or_default();
    if scheduled.pending(now.timestamp().as_second()) {
        println!(
            "Scheduled {}: {}",
            scheduled.action.noun(),
            words::time_of_day(&at(scheduled.at), &now, format)
        );
    }
}

/// The words before a trailing `battery` or `ac`, and the sources it names; both without one.
fn split_sources<'a>(words: &[&'a str]) -> (Vec<&'a str>, Sources) {
    match words.split_last() {
        Some((last, rest)) if !rest.is_empty() => match Sources::from_word(last) {
            Some(sources) => (rest.to_vec(), sources),
            None => (words.to_vec(), Sources::Both),
        },
        _ => (words.to_vec(), Sources::Both),
    }
}

fn set_wait(timer: &str, rest: &[&str]) -> Result<(), String> {
    let timer = Timer::from_name(timer)
        .ok_or_else(|| format!("There's no `{timer}` to set: lock, screens-off, suspend or lid"))?;
    let (typed, sources) = split_sources(rest);
    let wait = Wait::parse(&typed.join(" ")).map_err(|err| chain(&err))?;
    let settings = settings::set_wait(&settings_path()?, timer, wait, sources).map_err(|err| chain(&err))?;
    apply(&settings, true)?;
    say(Toast::Idle, &words::timer(&settings, timer));
    Ok(())
}

fn set_lid(rest: &[&str]) -> Result<(), String> {
    let (typed, sources) = split_sources(rest);
    let typed = typed.join(" ");
    let action = LidAction::from_name(&typed)
        .ok_or_else(|| format!("`{typed}` isn't something the lid can do: suspend, lock or nothing"))?;
    let settings = settings::set_lid(&settings_path()?, action, sources).map_err(|err| chain(&err))?;
    apply(&settings, false)?;
    say(Toast::Idle, &words::lid(&settings));
    Ok(())
}

fn reset() -> Result<(), String> {
    let settings = settings::reset(&settings_path()?).map_err(|err| chain(&err))?;
    apply(&settings, true)?;
    say(
        Toast::Idle,
        &format!("KubaOS's defaults again.\n{}", words::all(&settings)),
    );
    Ok(())
}

fn apply_file() -> Result<(), String> {
    let settings = Settings::load(&settings_path()?).map_err(|err| chain(&err))?;
    apply(&settings, true)?;
    say(Toast::Idle, &words::all(&settings));
    Ok(())
}

fn home() -> Result<PathBuf, String> {
    env::var_os("HOME")
        .filter(|home| !home.is_empty())
        .map(PathBuf::from)
        .ok_or_else(|| "HOME isn't set".to_owned())
}

fn write_listeners() -> Result<(), String> {
    let (settings, problem) = load();
    if let Some(problem) = problem {
        eprintln!("kubaos-idle: {problem}; writing KubaOS's defaults");
    }
    write_config(&settings)
}

fn write_config(settings: &Settings) -> Result<(), String> {
    let path = hypridle::path(&home()?);
    hypridle::write(&path, settings).map_err(|err| format!("Couldn't write {}: {err}", path.display()))
}

/// Puts `settings` to work: hypridle's listeners, hypridle restarted when `restart`, and the lid's
/// holder started or stopped.
fn apply(settings: &Settings, restart: bool) -> Result<(), String> {
    write_config(settings)?;
    if restart {
        restart_hypridle();
    }
    sync_lid_holder(settings, false)
}

/// Restarts hypridle in the session, or starts it if it isn't running. Through Hyprland, so it
/// runs with the session's environment and outlives this command; without a Hyprland to ask, the
/// next login reads the new listeners.
fn restart_hypridle() {
    let _ = Command::new("hyprctl")
        .args(["dispatch", "hl.dsp.exec_cmd([[dot-session start-idle]])"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

fn session() -> ExitCode {
    let (settings, problem) = load();
    if let Some(problem) = problem {
        notify(
            Toast::Idle,
            &format!("{problem}. KubaOS's idle defaults apply until it's fixed."),
            true,
        );
    }
    if let Err(message) = write_config(&settings) {
        eprintln!("kubaos-idle: {message}");
    }
    if let Err(message) = sync_lid_holder(&settings, true) {
        eprintln!("kubaos-idle: {message}");
    }
    let err = Command::new("hypridle").exec();
    eprintln!("kubaos-idle: couldn't start hypridle: {err}");
    ExitCode::FAILURE
}

/// Whether the systemd user unit `unit`'s service runs.
fn unit_active(unit: &str) -> bool {
    Command::new("systemctl")
        .args(["--user", "is-active", "--quiet", &format!("{unit}.service")])
        .stdin(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

fn stop_units(units: &[String]) {
    let _ = Command::new("systemctl")
        .args(["--user", "stop"])
        .args(units)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

/// Runs `argv` as the transient user unit `unit`, apart from this command and the menu, so it
/// outlives both and stopping the unit ends it.
fn run_unit(unit: &str, argv: &[String]) -> Result<(), String> {
    run(Command::new("systemd-run")
        .args(["--user", "--quiet", "--collect", &format!("--unit={unit}"), "--"])
        .args(argv))
}

/// Runs `command` to the end; its last line on stderr is the error.
fn run(command: &mut Command) -> Result<(), String> {
    let program = command.get_program().to_string_lossy().into_owned();
    let output = command
        .stdin(Stdio::null())
        .output()
        .map_err(|err| format!("Couldn't run {program}: {err}"))?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let said = stderr
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .unwrap_or("")
        .trim();
    Err(format!("{program} failed: {said}"))
}

/// Starts the lid's holder when the settings need one and none runs, and stops it when they don't.
/// `fresh` starts it over, so a login runs the holder of the kubaos-idle installed now.
fn sync_lid_holder(settings: &Settings, fresh: bool) -> Result<(), String> {
    if fresh {
        stop_units(&[format!("{}.service", lid::UNIT)]);
    }
    let running = !fresh && unit_active(lid::UNIT);
    if lid::needs_holder(settings) && !running {
        let argv = [
            "systemd-inhibit",
            "--what=handle-lid-switch",
            "--who=KubaOS",
            "--why=Your lid setting (kubaos-idle)",
            "--mode=block",
            &this_program(),
            "lid",
            "hold",
        ]
        .map(str::to_owned);
        run_unit(lid::UNIT, &argv)?;
    } else if !lid::needs_holder(settings) && running {
        stop_units(&[format!("{}.service", lid::UNIT)]);
    }
    Ok(())
}

/// Whether the user's Hyprland session still runs; without a runtime dir to tell, true.
fn session_alive() -> bool {
    kubaos_idle::runtime_dir().is_none_or(|runtime| machine::any_hyprland_runs(&runtime, Path::new("/proc")))
}

fn logind_lid() -> Option<LidState> {
    let output = Command::new("busctl")
        .args([
            "get-property",
            "org.freedesktop.login1",
            "/org/freedesktop/login1",
            "org.freedesktop.login1.Manager",
            "LidClosed",
            "Docked",
        ])
        .stdin(Stdio::null())
        .output()
        .ok()?;
    LidState::parse(&String::from_utf8_lossy(&output.stdout))
}

fn suspend() {
    let _ = Command::new("systemctl").arg("suspend").stdin(Stdio::null()).status();
}

/// Locks this user's graphical session. A transient unit isn't in the session, so the session is
/// named: logind's `Display` session of the user.
fn lock_session() -> Result<(), String> {
    let uid = fs::metadata("/proc/self").map(|meta| meta.uid()).unwrap_or_default();
    let display = Command::new("loginctl")
        .args(["show-user", &uid.to_string(), "-p", "Display", "--value"])
        .stdin(Stdio::null())
        .output()
        .ok()
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_owned())
        .unwrap_or_default();
    let mut command = Command::new("loginctl");
    command.arg("lock-session");
    if !display.is_empty() {
        command.arg(display);
    }
    run(&mut command)
}

fn lid_closed() -> Result<(), String> {
    if !unit_active(lid::UNIT) {
        return Ok(());
    }
    let (settings, _) = load();
    // A lid logind can't be asked about counts as docked: a docked laptop must never suspend.
    let docked = logind_lid().is_none_or(|state| state.docked);
    match lid::action(&settings, docked, machine::power(Path::new(machine::POWER_SUPPLY))) {
        LidAction::Suspend => suspend(),
        LidAction::Lock => lock_session()?,
        LidAction::Nothing => {}
    }
    Ok(())
}

fn lid_hold() -> Result<(), String> {
    let awake = Instant::now();
    let mut recheck = lid::Recheck::default();
    let (mut settings, _) = load();
    loop {
        thread::sleep(lid::TICK);
        if !session_alive() {
            return Ok(());
        }
        // A file broken mid-edit keeps the last settings that read.
        if let (read, None) = load() {
            settings = read;
        }
        if !lid::needs_holder(&settings) {
            return Ok(());
        }
        let Some(state) = logind_lid() else { continue };
        let action = lid::action(
            &settings,
            state.docked,
            machine::power(Path::new(machine::POWER_SUPPLY)),
        );
        if recheck.tick(awake.elapsed(), state.closed, action) {
            suspend();
        }
    }
}

/// The clock's time format, which the toasts write times in.
fn clock_format() -> Option<kubaos_clock::settings::TimeFormat> {
    let dir = kubaos_clock::settings::dir()?;
    kubaos_clock::settings::Settings::load(&dir.join(kubaos_clock::settings::FILE))
        .ok()?
        .time_format
}

/// The moment `secs` seconds after the epoch, here.
fn at(secs: i64) -> Zoned {
    Timestamp::from_second(secs)
        .unwrap_or(Timestamp::UNIX_EPOCH)
        .to_zoned(TimeZone::system())
}

fn runtime() -> Result<PathBuf, String> {
    kubaos_idle::runtime_dir().ok_or_else(|| "XDG_RUNTIME_DIR isn't set".to_owned())
}

fn awake_for(typed: &str) -> Result<(), String> {
    let duration = kubaos_clock::duration(typed)
        .ok_or_else(|| format!("`{typed}` isn't a time to keep awake for: try 1h, 2h or 30m"))?;
    let until = Zoned::now()
        .checked_add(duration)
        .map_err(|err| format!("{typed} from now is out of range: {err}"))?;
    awake_start(until.timestamp().as_second())
}

/// Keeps awake until `until`, 0 for until turned off, in place of any Keep awake before it.
fn awake_start(until: i64) -> Result<(), String> {
    let path = awake::path(&runtime()?);
    stop_units(&[format!("{}.service", awake::UNIT)]);
    awake::State::on(until)
        .write(&path)
        .map_err(|err| format!("Couldn't write {}: {err}", path.display()))?;
    let argv = [
        "systemd-inhibit",
        "--what=idle",
        "--who=KubaOS",
        "--why=Keep awake",
        "--mode=block",
        &this_program(),
        "awake",
        "hold",
        &until.to_string(),
    ]
    .map(str::to_owned);
    let started = run_unit(awake::UNIT, &argv).and_then(|()| {
        // systemd-inhibit exits at once when logind refuses the inhibitor.
        thread::sleep(Duration::from_millis(300));
        if unit_active(awake::UNIT) {
            Ok(())
        } else {
            Err(format!(
                "Couldn't keep awake: logind refused the idle inhibitor (journalctl --user -u {})",
                awake::UNIT
            ))
        }
    });
    if let Err(message) = started {
        let _ = awake::State::default().write(&path);
        return Err(message);
    }
    let now = Zoned::now();
    let body = if until == 0 {
        "On until you turn it off. Nothing locks, goes dark or suspends by itself.".to_owned()
    } else {
        format!(
            "On for {}, until {}. Nothing locks, goes dark or suspends by itself till then.",
            words::left(until - now.timestamp().as_second()),
            words::time_of_day(&at(until), &now, clock_format())
        )
    };
    say(Toast::KeepAwake, &body);
    Ok(())
}

fn awake_off() -> Result<(), String> {
    let path = awake::path(&runtime()?);
    stop_units(&[format!("{}.service", awake::UNIT)]);
    awake::State::default()
        .write(&path)
        .map_err(|err| format!("Couldn't write {}: {err}", path.display()))?;
    let (settings, _) = load();
    say(
        Toast::KeepAwake,
        &format!("Off. {}.", words::timer(&settings, Timer::Lock)),
    );
    Ok(())
}

fn awake_toggle() -> Result<(), String> {
    let state = awake::State::read(&awake::path(&runtime()?));
    if state.holds(Zoned::now().timestamp().as_second()) || unit_active(awake::UNIT) {
        awake_off()
    } else {
        awake_start(0)
    }
}

/// The holder: waits until `until` (forever for 0) or until the session is gone, then says so.
fn awake_hold(until: &str) -> Result<(), String> {
    let until: i64 = until
        .parse()
        .map_err(|_| format!("`{until}` isn't a time in seconds"))?;
    let path = awake::path(&runtime()?);
    loop {
        let now = Zoned::now().timestamp().as_second();
        if until != 0 && now >= until {
            break;
        }
        if !session_alive() {
            let _ = awake::State::default().write(&path);
            return Ok(());
        }
        let left = if until == 0 {
            lid::TICK.as_secs()
        } else {
            (until - now).unsigned_abs()
        };
        thread::sleep(Duration::from_secs(left.min(lid::TICK.as_secs())));
    }
    let _ = awake::State::default().write(&path);
    let (settings, _) = load();
    notify(
        Toast::KeepAwake,
        &format!("Over. {}.", words::timer(&settings, Timer::Lock)),
        false,
    );
    Ok(())
}

fn schedule_in(action: Action, typed: &str) -> Result<(), String> {
    let now = Zoned::now();
    let at =
        schedule::after(typed, &now).ok_or_else(|| format!("`{typed}` isn't a time to wait: try 10m, 1h30m or 90s"))?;
    schedule_action(action, &at, &now)
}

fn schedule_at(action: Action, typed: &str) -> Result<(), String> {
    let now = Zoned::now();
    let at = schedule::at_time(typed, &now)
        .ok_or_else(|| format!("`{typed}` isn't a time to come: try 20:00, 9pm or friday 17:00"))?;
    schedule_action(action, &at, &now)
}

/// Schedules `action` at `at` in place of whatever was scheduled before, and says when, and what
/// it replaced.
fn schedule_action(action: Action, at: &Zoned, now: &Zoned) -> Result<(), String> {
    let path = schedule::path(&runtime()?);
    let before = schedule::State::read(&path);
    stop_units(&schedule::timers());
    for timer in schedule::plan(action, at, now, &this_program()) {
        let argv = timer.systemd_run();
        if let Err(message) = run(Command::new(&argv[0]).args(&argv[1..])) {
            stop_units(&schedule::timers());
            let _ = schedule::State::default().write(&path);
            return Err(format!("Couldn't schedule the {}: {message}", action.noun()));
        }
    }
    schedule::State::new(action, at.timestamp().as_second())
        .write(&path)
        .map_err(|err| format!("Couldn't write {}: {err}", path.display()))?;
    let format = clock_format();
    let mut body = format!(
        "{} {}, in {}, even if you're working or Keep awake is on.",
        action.does(),
        words::when(at, now, format),
        words::left(now.duration_until(at).as_secs())
    );
    if before.pending(now.timestamp().as_second()) {
        let _ = write!(
            body,
            " It replaces the {} {}.",
            before.action.noun(),
            words::when(&self::at(before.at), now, format)
        );
    }
    say(Toast::Scheduled(action), &body);
    Ok(())
}

fn schedule_cancel() -> Result<(), String> {
    let path = schedule::path(&runtime()?);
    let before = schedule::State::read(&path);
    stop_units(&schedule::timers());
    schedule::State::default()
        .write(&path)
        .map_err(|err| format!("Couldn't write {}: {err}", path.display()))?;
    let now = Zoned::now();
    if before.pending(now.timestamp().as_second()) {
        say(
            Toast::Scheduled(before.action),
            &format!(
                "The {} {} is cancelled.",
                before.action.noun(),
                words::when(&at(before.at), &now, clock_format())
            ),
        );
    } else {
        say(Toast::Scheduled(Action::Lock), "Nothing was scheduled.");
    }
    Ok(())
}

/// The action's own command, `action` and `at` as its timer names them. It runs only while the
/// state still says so, and in the Hyprland session through `hyprctl dispatch`, as a keybind's
/// would: a transient unit isn't in the session, which polkit asks of a suspend or a power off.
fn schedule_fire(action: &str, at: &str) -> Result<(), String> {
    let (action, at) = timer_words(action, at)?;
    let path = schedule::path(&runtime()?);
    if !schedule::State::read(&path).is(action, at) {
        return Ok(());
    }
    let _ = schedule::State::default().write(&path);
    let session =
        kubaos_idle::runtime_dir().and_then(|runtime| machine::running_hyprland(&runtime, Path::new("/proc")));
    let Some(signature) = session else {
        // No session to run it in: the lock names the user's session, the rest ask logind as is.
        return match action {
            Action::Lock => lock_session(),
            _ => run(Command::new(action.command()[0]).args(&action.command()[1..])),
        };
    };
    let exec = format!("hl.dsp.exec_cmd([[{}]])", action.command().join(" "));
    run(Command::new("hyprctl").args(["-i", &signature, "dispatch", &exec]))
}

/// A heads-up `secs` before `action` at `at`, with a Cancel button. It waits for the toast, so a
/// click on Cancel cancels the action, unless another has replaced it since.
fn schedule_warn(secs: &str, action: &str, at: &str) -> Result<(), String> {
    let secs: u32 = secs
        .parse()
        .map_err(|_| format!("`{secs}` isn't a number of seconds"))?;
    let (action, at) = timer_words(action, at)?;
    let path = schedule::path(&runtime()?);
    if !schedule::State::read(&path).is(action, at) {
        return Ok(());
    }
    let toast = Toast::Scheduled(action);
    let body = format!(
        "{} in {}. Cancel to stop it, or \"Cancel scheduled action\" in the Command menu.",
        action.doing(),
        words::wait(Wait::from_secs(secs))
    );
    let mut command = Command::new("notify-send");
    command.args(["-a", toast.app(), "-i", toast.icon(), "--action=cancel=Cancel"]);
    if secs <= 10 || matches!(action, Action::Shutdown | Action::Reboot) {
        command.args(["-u", "critical"]);
    }
    let clicked = command
        .args([toast.app(), body.as_str()])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .is_ok_and(|output| String::from_utf8_lossy(&output.stdout).trim() == "cancel");
    if clicked && schedule::State::read(&path).is(action, at) {
        schedule_cancel()?;
    }
    Ok(())
}

/// The action and the time its timer was made with.
fn timer_words(action: &str, at: &str) -> Result<(Action, i64), String> {
    let action = Action::from_name(action).ok_or_else(|| format!("`{action}` isn't a scheduled action"))?;
    let at = at.parse().map_err(|_| format!("`{at}` isn't a time in seconds"))?;
    Ok((action, at))
}
