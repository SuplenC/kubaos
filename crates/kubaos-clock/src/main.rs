//! `kubaos-clock`: the date engine behind the clock popup, and the changes the Command menu makes
//! to its settings.
//!
//! ```text
//! kubaos-clock serve                       a line of JSON in, one out, until stdin ends (the popup)
//! kubaos-clock eval QUERY...               what a date query comes to: the answer, the line under
//!                                          it, and its ISO form
//! kubaos-clock state                       the popup's state, as JSON
//! kubaos-clock zone add|remove NAME...     an extra clock, by city or IANA name
//! kubaos-clock countdown add NAME WHEN...  a countdown to a date or time (`20 dec`, `friday 17:00`)
//! kubaos-clock countdown remove NAME
//! kubaos-clock set KEY VALUE               holidays CODE|none, first-weekday DAY, analog on|off|toggle,
//!                                          bar-countdown NAME|none, time-format 24h|12h|locale,
//!                                          date-order day-month|month-day|locale
//! ```
//!
//! # Owns
//!
//! - The command line and its exit codes: 2 for a usage error, 1 when the engine refused, with the
//!   reason on stderr, which the menu shows.
//!
//! # Not here
//!
//! - Everything else: the library's modules.

#![forbid(unsafe_code)]

use std::io;
use std::process::ExitCode;

use jiff::Zoned;
use kubaos_clock::serve;
use kubaos_clock::serve::Server;
use kubaos_clock::settings;

const USAGE: &str = "\
usage: kubaos-clock serve
       kubaos-clock eval QUERY...
       kubaos-clock state
       kubaos-clock zone add|remove NAME...
       kubaos-clock countdown add NAME WHEN...
       kubaos-clock countdown remove NAME
       kubaos-clock set KEY VALUE

serve      answer the clock popup: a line of JSON in, one out
eval       what a date query comes to (`days until christmas`, `15:00 tokyo`)
state      the popup's state: settings, clocks, countdowns, as JSON
zone       add or remove an extra clock, by city or IANA name (`tokyo`, `America/New_York`)
countdown  add a countdown to a date or time (`20 dec`, `friday 17:00`), or remove one
set        holidays CODE|none, first-weekday DAY, analog on|off|toggle, bar-countdown NAME|none,
           time-format 24h|12h|locale, date-order day-month|month-day|locale

Settings: ~/.config/kubaos/desktop/clock/clock.toml";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let mut server = Server::new(settings::dir());
    let now = Zoned::now();
    match args.as_slice() {
        ["serve"] => match serve::run(&mut server, io::stdin().lock(), io::stdout().lock()) {
            Ok(()) => ExitCode::SUCCESS,
            Err(err) => failed(&err),
        },
        ["eval", query @ ..] if !query.is_empty() => {
            let query = query.join(" ");
            if let Some(answer) = server.query(&query, &now) {
                println!("{}\n{}\n{}", answer.title, answer.detail, answer.iso);
                ExitCode::SUCCESS
            } else {
                eprintln!("kubaos-clock: no answer for {query:?}");
                ExitCode::FAILURE
            }
        }
        ["state"] => {
            println!("{}", server.handle(r#"{"op":"state"}"#, &now));
            ExitCode::SUCCESS
        }
        ["zone", "add", name @ ..] if !name.is_empty() => report(server.add_zone(&name.join(" ")).map(|name| {
            println!("added {name}");
        })),
        ["zone", "remove", name @ ..] if !name.is_empty() => report(server.remove_zone(&name.join(" "))),
        ["countdown", "add", name, when @ ..] if !when.is_empty() => {
            report(server.add_countdown(name, &when.join(" "), &now).map(|at| {
                println!("{name}: {at}");
            }))
        }
        ["countdown", "remove", name] => report(server.remove_countdown(name)),
        ["set", key, value @ ..] if !value.is_empty() => report(server.set(key, &value.join(" "))),
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

fn report(result: Result<(), serve::CommandError>) -> ExitCode {
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => failed(&err),
    }
}

fn failed(err: &dyn std::error::Error) -> ExitCode {
    eprintln!("kubaos-clock: {}", serve::chain(err));
    ExitCode::FAILURE
}
