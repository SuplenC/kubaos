//! `kubaos-counters`: the counters, from the Command menu, the key, the shell and a terminal.
//!
//! ```text
//! kubaos-counters list [--json]            every counter, its value and auto reset
//! kubaos-counters new [NAME]               a counter at 0 at the end; no name is "Counter"
//! kubaos-counters add NAME|--last          +1; --last, the counter used last
//! kubaos-counters sub NAME                 −1, never below 0
//! kubaos-counters set NAME N               to N, or its stop if N is past it
//! kubaos-counters reset NAME               back to 0
//! kubaos-counters rename NAME NEW
//! kubaos-counters rule NAME RULE|never     back to 0 by itself: every day at 9, mondays 8am,
//!                                          1st of the month, every 2 hours; never, no more
//! kubaos-counters stop NAME N              counts up to N and no further
//! kubaos-counters move NAME POSITION       where it is in the list, from 1
//! kubaos-counters remove NAME              the counter and its history
//! kubaos-counters log NAME [--group change|day|week|month] [--sort newest|oldest|highest] [--json]
//! kubaos-counters check RULE [--json]      what a rule reads as, and its next time
//! kubaos-counters tick                     the resets that came due; the timer and the login run it
//! ```
//!
//! NAME is one argument, matched exactly or else whatever the case; NEW and RULE are the rest of
//! the line. `--notify` anywhere also says the outcome in a toast, for the menu and the key, which
//! have no terminal. A change says what it came to on stdout; a refusal says why on stderr, exit 1;
//! a usage error exits 2.
//!
//! Every command holds `counters.lock` while it reads and writes, first does the resets that came
//! due, then writes the change log, `counters.toml` and the shell's state file, and arms the
//! transient `kubaos-counters-reset.timer` at the next reset. The timer is on the calendar, so one
//! that came due while asleep fires on resume; the login's `kubaos-counters.service` runs `tick`,
//! for the resets missed while switched off.
//!
//! # Owns
//!
//! - The command line, the lock, the order the files are written in, the timer and the toasts.
//!
//! # Not here
//!
//! - The counters, the log and the JSON: the library's modules.

use std::env;
use std::fs;
use std::process::Command;
use std::process::ExitCode;
use std::process::Stdio;

use jiff::Timestamp;
use jiff::Zoned;
use jiff::tz::TimeZone;
use kubaos_clock::recurrence::Recurrence;
use kubaos_clock::settings::TimeFormat;
use kubaos_counters::Paths;
use kubaos_counters::history;
use kubaos_counters::history::Entry;
use kubaos_counters::history::Group;
use kubaos_counters::history::Sort;
use kubaos_counters::live;
use kubaos_counters::store::AutoReset;
use kubaos_counters::store::ChangeError;
use kubaos_counters::store::Counters;

const USAGE: &str = "\
usage: kubaos-counters list [--json]
       kubaos-counters new [NAME]
       kubaos-counters add NAME|--last | sub NAME | set NAME N | reset NAME
       kubaos-counters rename NAME NEW
       kubaos-counters rule NAME RULE|never | stop NAME N
       kubaos-counters move NAME POSITION | remove NAME
       kubaos-counters log NAME [--group change|day|week|month] [--sort newest|oldest|highest] [--json]
       kubaos-counters check RULE [--json]
       kubaos-counters tick

new     a counter at 0 at the end of the list; no name is \"Counter\"
add     +1, to NAME or, with --last, the counter used last; at its stop it stays
sub     -1; never below 0
set     to N; past its stop, the stop
rule    back to 0 by itself: every day at 9, mondays 8am, 1st of the month, every 2 hours,
        every 30 minutes; never takes the rule or the stop away
stop    counts up to N and no further, till reset
remove  the counter and its history
log     its changes, each or by day, week or month, newest, oldest or highest first
check   what RULE reads as, and when it next comes
tick    the resets that came due (the reset timer and the login run it)

--notify anywhere also says it in a toast.
Counters: ~/.local/state/kubaos/counters.toml, their changes beside it in counters.log";

/// The transient timer, `.timer` and `.service`, that runs `tick` at the next reset.
const TIMER_UNIT: &str = "kubaos-counters-reset";

/// What a change came to: the lines to log, the counter whose lines go first, what to say, and
/// whether the counters changed.
struct Outcome {
    entries: Vec<Entry>,
    removed: Option<u32>,
    said: String,
    changed: bool,
}

impl Outcome {
    fn said(entries: Vec<Entry>, said: String) -> Self {
        Outcome {
            entries,
            removed: None,
            said,
            changed: true,
        }
    }

    /// A look that changed nothing.
    fn read(said: String) -> Self {
        Outcome {
            entries: Vec::new(),
            removed: None,
            said,
            changed: false,
        }
    }
}

#[expect(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "The help and the usage are the command's output, for whoever ran it."
)]
fn main() -> ExitCode {
    let mut args: Vec<String> = env::args().skip(1).collect();
    let notify = take_flag(&mut args, "--notify");
    let json = take_flag(&mut args, "--json");
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let result = match args.as_slice() {
        ["list"] => list(json),
        ["new", name @ ..] => {
            let name = name.join(" ");
            change(|counters, now| {
                let entries = counters.create(&name, now)?;
                let name = counters.last().map_or(String::new(), |counter| counter.name.clone());
                Ok(Outcome::said(entries, format!("New counter: {name}")))
            })
        }
        ["add", "--last"] => change(|counters, now| {
            let name = counters.last_name()?.to_owned();
            step(counters, &name, now, Counters::add)
        }),
        ["add", name] => change(|counters, now| step(counters, name, now, Counters::add)),
        ["sub", name] => change(|counters, now| step(counters, name, now, Counters::sub)),
        ["reset", name] => change(|counters, now| step(counters, name, now, Counters::reset)),
        ["set", name, value] => match value.parse::<u64>() {
            Ok(value) => change(|counters, now| step(counters, name, now, |c, n, now| c.set(n, value, now))),
            Err(_) => Err(format!("`{value}` isn't a whole number of 0 or more")),
        },
        ["rename", name, to @ ..] if !to.is_empty() => {
            let to = to.join(" ");
            change(|counters, now| {
                let entries = counters.rename(name, &to, now)?;
                Ok(Outcome::said(entries, format!("Renamed to {}", counters.find(&to)?.name)))
            })
        }
        ["rule", name, rule @ ..] if !rule.is_empty() => set_rule(name, &rule.join(" ")),
        ["stop", name, stop] => match stop.parse::<u64>() {
            Ok(stop) => change(|counters, now| {
                let entries = counters.set_reset(name, AutoReset::Stop(stop), now)?;
                let counter = counters.find(name)?;
                Ok(Outcome::said(entries, format!("{}: stops at {stop}", counter.name)))
            }),
            Err(_) => Err(format!("`{stop}` isn't a whole number of 1 or more")),
        },
        ["move", name, position] => match position.parse::<usize>() {
            Ok(position) => change(|counters, _| {
                counters.move_to(name, position)?;
                Ok(Outcome::said(Vec::new(), list_text(counters, time_format())))
            }),
            Err(_) => Err(format!("`{position}` isn't a place in the list")),
        },
        ["remove", name] => change(|counters, now| {
            let (counter, entries) = counters.remove(name, now)?;
            Ok(Outcome {
                entries,
                removed: Some(counter.id),
                said: format!("Removed {}", counter.name),
                changed: true,
            })
        }),
        ["log", name, rest @ ..] => log(name, rest, json),
        ["check", rule @ ..] if !rule.is_empty() => check(&rule.join(" "), json),
        ["tick"] => tick(),
        ["-h" | "--help" | "help"] => {
            println!("{USAGE}");
            return ExitCode::SUCCESS;
        }
        _ => {
            eprintln!("{USAGE}");
            return ExitCode::from(2);
        }
    };
    report(result, notify)
}

/// Whether `flag` is among `args`, taking it out.
fn take_flag(args: &mut Vec<String>, flag: &str) -> bool {
    let before = args.len();
    args.retain(|arg| arg != flag);
    args.len() != before
}

/// Says what a command came to: on stdout, or a refusal on stderr with exit 1; in a toast too when
/// `notify`.
#[expect(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "What a command came to is its answer, for whoever ran it."
)]
fn report(result: Result<String, String>, notify: bool) -> ExitCode {
    match result {
        Ok(said) => {
            if !said.is_empty() {
                println!("{said}");
            }
            if notify && !said.is_empty() {
                toast(&said, false);
            }
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("kubaos-counters: {message}");
            if notify {
                toast(&message, true);
            }
            ExitCode::FAILURE
        }
    }
}

fn toast(body: &str, critical: bool) {
    let mut command = Command::new("notify-send");
    command.args(["-a", "Counters", "-i", "accessories-calculator"]);
    if critical {
        command.args(["-u", "critical"]);
    }
    let _ = command
        .args(["Counters", body])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

/// A change by one, or to a value, of `name`: `Pushups: 13`.
fn step(
    counters: &mut Counters,
    name: &str,
    now: &Zoned,
    change: impl FnOnce(&mut Counters, &str, &Zoned) -> Result<Vec<Entry>, ChangeError>,
) -> Result<Outcome, ChangeError> {
    let entries = change(counters, name, now)?;
    let counter = counters.find(name)?;
    Ok(Outcome::said(entries, format!("{}: {}", counter.name, counter.value)))
}

fn set_rule(name: &str, typed: &str) -> Result<String, String> {
    let reset = if typed.trim().eq_ignore_ascii_case("never") {
        AutoReset::Never
    } else {
        AutoReset::Rule(Recurrence::parse(typed).map_err(|err| err.to_string())?)
    };
    let format = time_format();
    change(|counters, now| {
        let entries = counters.set_reset(name, reset, now)?;
        let counter = counters.find(name)?;
        let said = match counter.next_reset {
            Some(next) => format!(
                "{}: {}; next {}",
                counter.name,
                live::detail(reset, format),
                live::when(&next.to_zoned(now.time_zone().clone()), now, format)
            ),
            None => format!("{}: no auto reset", counter.name),
        };
        Ok(Outcome::said(entries, said))
    })
}

/// Reads the counters under the lock, does the resets that came due, makes the change `apply`
/// asks for, and writes the log, the counters and the shell's state file, then arms the timer if
/// the next reset moved. What `apply` says is the answer.
fn change(
    apply: impl FnOnce(&mut Counters, &Zoned) -> Result<Outcome, ChangeError>,
) -> Result<String, String> {
    let paths = paths()?;
    let _lock = lock(&paths)?;
    let mut counters = Counters::load(&paths.counters).map_err(|err| chain(&err))?;
    let armed = counters.next_reset();
    let now = Zoned::now();
    let due = counters.catch_up(&now);
    history::append(&paths.log, &due).map_err(|err| format!("Couldn't write {}: {err}", paths.log.display()))?;
    let caught_up = !due.is_empty() || armed != counters.next_reset();
    let outcome = match apply(&mut counters, &now) {
        Ok(outcome) => outcome,
        Err(err) => {
            if caught_up {
                save(&paths, &counters, armed)?;
            }
            return Err(err.to_string());
        }
    };
    if !outcome.changed && !caught_up {
        return Ok(outcome.said);
    }
    if let Some(id) = outcome.removed {
        history::without(&paths.log, id).map_err(|err| format!("Couldn't write {}: {err}", paths.log.display()))?;
    }
    history::append(&paths.log, &outcome.entries)
        .map_err(|err| format!("Couldn't write {}: {err}", paths.log.display()))?;
    save(&paths, &counters, armed)?;
    Ok(outcome.said)
}

/// Writes the counters and the shell's state file, and arms the timer when the next reset isn't
/// `armed` any more.
fn save(paths: &Paths, counters: &Counters, armed: Option<Timestamp>) -> Result<(), String> {
    counters.save(&paths.counters).map_err(|err| chain(&err))?;
    write_live(counters);
    if counters.next_reset() != armed {
        arm(counters.next_reset());
    }
    Ok(())
}

#[expect(
    clippy::print_stderr,
    reason = "A state file the bar can't get is a warning to whoever ran it, or the journal."
)]
fn write_live(counters: &Counters) {
    let Some(path) = kubaos_counters::live_file() else {
        return;
    };
    if let Err(err) = live::write(&path, &live::state(counters, time_format())) {
        eprintln!("kubaos-counters: couldn't write {}: {err}", path.display());
    }
}

/// The resets that came due; the shell's state file written again and the timer armed whatever
/// they were, as at login or when the timer fires.
fn tick() -> Result<String, String> {
    let paths = paths()?;
    let _lock = lock(&paths)?;
    let mut counters = Counters::load(&paths.counters).map_err(|err| chain(&err))?;
    let due = counters.catch_up(&Zoned::now());
    history::append(&paths.log, &due).map_err(|err| format!("Couldn't write {}: {err}", paths.log.display()))?;
    if !due.is_empty() || fs::exists(&paths.counters).unwrap_or(false) {
        counters.save(&paths.counters).map_err(|err| chain(&err))?;
    }
    write_live(&counters);
    arm(counters.next_reset());
    Ok(String::new())
}

/// Starts `kubaos-counters-reset.timer` at `at`, replacing one that runs; with no time, stops it.
#[expect(
    clippy::print_stderr,
    reason = "A timer that won't start is a warning to whoever ran it, or the journal; the next command catches up."
)]
fn arm(at: Option<Timestamp>) {
    let _ = Command::new("systemctl")
        .args(["--user", "stop", &format!("{TIMER_UNIT}.timer")])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    let Some(at) = at else {
        return;
    };
    let program = env::current_exe().map_or_else(
        |_| "kubaos-counters".to_owned(),
        |exe| exe.to_string_lossy().into_owned(),
    );
    let calendar = at.to_zoned(TimeZone::UTC).strftime("%Y-%m-%d %H:%M:%S UTC").to_string();
    let started = Command::new("systemd-run")
        .args([
            "--user",
            "--quiet",
            "--collect",
            &format!("--unit={TIMER_UNIT}"),
            &format!("--on-calendar={calendar}"),
            "--timer-property=AccuracySec=1s",
            "--",
            &program,
            "tick",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output();
    match started {
        Ok(output) if output.status.success() => {}
        Ok(output) => eprintln!(
            "kubaos-counters: couldn't arm the reset timer: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ),
        Err(err) => eprintln!("kubaos-counters: couldn't run systemd-run: {err}"),
    }
}

fn list(json: bool) -> Result<String, String> {
    let format = time_format();
    change(|counters, _| {
        Ok(Outcome::read(if json {
            live::state(counters, format).to_string()
        } else {
            list_text(counters, format)
        }))
    })
}

/// A line a counter: `Pushups  12  Resets every day at 9:00`.
fn list_text(counters: &Counters, format: Option<TimeFormat>) -> String {
    if counters.all().is_empty() {
        return "No counters yet".to_owned();
    }
    let width = counters
        .all()
        .iter()
        .map(|counter| counter.name.chars().count())
        .max()
        .unwrap_or(0);
    counters
        .all()
        .iter()
        .map(|counter| {
            let line = format!("{:width$}  {}", counter.name, counter.value);
            let detail = live::detail(counter.reset, format);
            if detail.is_empty() { line } else { format!("{line}  {detail}") }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn log(name: &str, rest: &[&str], json: bool) -> Result<String, String> {
    let (mut group, mut sort) = (Group::default(), Sort::default());
    let mut options = rest.iter();
    while let Some(option) = options.next() {
        let value = options.next().copied().unwrap_or("");
        match *option {
            "--group" => {
                group = Group::from_name(value)
                    .ok_or_else(|| format!("`{value}` isn't a group: change, day, week or month"))?;
            }
            "--sort" => {
                sort = Sort::from_name(value)
                    .ok_or_else(|| format!("`{value}` isn't a sort: newest, oldest or highest"))?;
            }
            _ => return Err(format!("`{option}` isn't an option of log: --group, --sort or --json")),
        }
    }
    let paths = paths()?;
    let id = {
        let _lock = lock(&paths)?;
        let counters = Counters::load(&paths.counters).map_err(|err| chain(&err))?;
        counters.find(name).map_err(|err| err.to_string())?.id
    };
    let entries = history::read(&paths.log).map_err(|err| format!("Couldn't read {}: {err}", paths.log.display()))?;
    let tz = TimeZone::system();
    let rows = history::history(&entries, id, group, sort, &tz);
    if json {
        return Ok(live::history(&rows).to_string());
    }
    if rows.is_empty() {
        return Ok("No changes yet".to_owned());
    }
    let lines = rows.iter().map(|row| {
        let at = row.at.to_zoned(tz.clone());
        match row.kind {
            Some(kind) => format!("{}  {:>10}  {}", at.strftime("%Y-%m-%d %H:%M"), kind.label(), row.value),
            None => format!(
                "{}  {:+}  {} changes, ending at {}",
                at.strftime("%Y-%m-%d"),
                row.added,
                row.changes,
                row.value
            ),
        }
    });
    Ok(lines.collect::<Vec<_>>().join("\n"))
}

fn check(typed: &str, json: bool) -> Result<String, String> {
    let now = Zoned::now();
    let format = time_format();
    let answer = live::check(typed, &now, format);
    if json {
        return Ok(answer.to_string());
    }
    if answer["ok"] == true {
        Ok(format!(
            "{}\n{}",
            answer["words"].as_str().unwrap_or(""),
            answer["when"].as_str().unwrap_or("")
        ))
    } else {
        Err(answer["error"].as_str().unwrap_or("").to_owned())
    }
}

fn paths() -> Result<Paths, String> {
    Paths::from_env().ok_or_else(|| "Neither HOME nor XDG_STATE_HOME is set".to_owned())
}

/// Holds `counters.lock` till it is dropped, so two commands never read and write at once.
fn lock(paths: &Paths) -> Result<fs::File, String> {
    if let Some(dir) = paths.lock.parent() {
        fs::create_dir_all(dir).map_err(|err| format!("Couldn't make {}: {err}", dir.display()))?;
    }
    let file = fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(&paths.lock)
        .map_err(|err| format!("Couldn't open {}: {err}", paths.lock.display()))?;
    file.lock()
        .map_err(|err| format!("Couldn't lock {}: {err}", paths.lock.display()))?;
    Ok(file)
}

/// The clock's time format, which the words of the rules follow.
fn time_format() -> Option<TimeFormat> {
    let dir = kubaos_clock::settings::dir()?;
    kubaos_clock::settings::Settings::load(&dir.join(kubaos_clock::settings::FILE))
        .ok()?
        .time_format
}

/// `err` and its causes, a `: ` between each.
fn chain(err: &dyn std::error::Error) -> String {
    let mut out = err.to_string();
    let mut source = err.source();
    while let Some(cause) = source {
        out.push_str(": ");
        out.push_str(&cause.to_string());
        source = cause.source();
    }
    out
}
