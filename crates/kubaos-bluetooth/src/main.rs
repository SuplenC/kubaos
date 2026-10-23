//! `kubaos-bluetooth`: the pairing agent, and the commands the shell runs to reach it.
//!
//! ```text
//! kubaos-bluetooth agent                       the agent, until a bus goes (the user unit runs it)
//! kubaos-bluetooth pair DEVICE                 start pairing the device at that object path
//! kubaos-bluetooth cancel DEVICE               cancel its pairing
//! kubaos-bluetooth answer ID yes|no [VALUE]    answer the prompt ID, with the PIN or passkey typed
//! kubaos-bluetooth scan on|off ADAPTER         start or stop scanning on the adapter
//! ```
//!
//! # Owns
//!
//! - The command line and its exit codes: 2 for a usage error, 1 when the agent refused or isn't
//!   running, with the reason on stderr, which the shell shows.
//!
//! # Not here
//!
//! - Everything else: the library's modules.

#![forbid(unsafe_code)]

use std::process::ExitCode;

use kotodama_extension::Chain;
use kubaos_bluetooth::service;
use kubaos_bluetooth::service::CallError;
use zbus::zvariant::ObjectPath;

const USAGE: &str = "\
usage: kubaos-bluetooth agent
       kubaos-bluetooth pair DEVICE
       kubaos-bluetooth cancel DEVICE
       kubaos-bluetooth answer ID yes|no [VALUE]
       kubaos-bluetooth scan on|off ADAPTER

agent   the pairing agent (the kubaos-bluetooth user unit runs it)
pair    start pairing the device at that BlueZ object path
cancel  cancel its pairing
answer  answer the agent's prompt ID, with the PIN or passkey typed
scan    start or stop scanning on the adapter at that object path";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    match args.as_slice() {
        ["agent"] => agent(),
        ["pair", device] => with_path(device, |device| service::call("Pair", &(device,))),
        ["cancel", device] => {
            with_path(device, |device| service::call("CancelPairing", &(device,)))
        }
        ["answer", id, answer, rest @ ..] if rest.len() <= 1 => {
            let (Ok(id), Some(accept)) = (id.parse::<u32>(), yes_no(answer)) else {
                return usage();
            };
            let value = rest.first().copied().unwrap_or_default();
            report(service::call("Answer", &(id, accept, value)))
        }
        ["scan", on, adapter] => {
            let on = match *on {
                "on" => true,
                "off" => false,
                _ => return usage(),
            };
            with_path(adapter, |adapter| service::call("Scan", &(adapter, on)))
        }
        ["-h" | "--help"] => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        _ => usage(),
    }
}

fn agent() -> ExitCode {
    match async_io::block_on(service::run()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("kubaos-bluetooth: {}", Chain(&err));
            ExitCode::FAILURE
        }
    }
}

fn with_path(path: &str, call: impl FnOnce(ObjectPath<'_>) -> Result<(), CallError>) -> ExitCode {
    let Ok(object) = ObjectPath::try_from(path) else {
        eprintln!("kubaos-bluetooth: {path} isn't a D-Bus object path");
        return ExitCode::from(2);
    };
    report(call(object))
}

fn yes_no(answer: &str) -> Option<bool> {
    match answer {
        "yes" => Some(true),
        "no" => Some(false),
        _ => None,
    }
}

/// The call's outcome as the exit code. A refusal or a missing agent is one bare line on stderr,
/// since the shell shows that line as it is.
fn report(result: Result<(), CallError>) -> ExitCode {
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err @ (CallError::Refused(_) | CallError::NotRunning)) => {
            eprintln!("{err}");
            ExitCode::FAILURE
        }
        Err(err) => {
            eprintln!("kubaos-bluetooth: {}", Chain(&err));
            ExitCode::FAILURE
        }
    }
}

fn usage() -> ExitCode {
    eprintln!("{USAGE}");
    ExitCode::from(2)
}
