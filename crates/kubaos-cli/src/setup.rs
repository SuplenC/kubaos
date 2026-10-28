//! `kubaos setup`: first-boot setup without Kōbō (docs/INSTALL-INTERNALS.md,
//! "First boot"). Runs as the desktop user, like `kubaos-setup`.
//!
//!   kubaos setup status [--json] [--demo vm|tuxedo]
//!   kubaos setup run CARD [--input-stdin] [--events json|human] [--demo vm|tuxedo]
//!   kubaos setup done
//!
//! `run` takes the card's inputs (`kubaos_core::setup::Inputs`) as one
//! JSON object on stdin, the way `run --secrets-stdin` does. Exit codes: 0
//! ok, 1 the card failed (the last event is `failed`), 2 usage.

use std::io::Read;
use std::process::ExitCode;

use clap::Subcommand;
use clap::ValueEnum;
use kubaos_core::Level;
use kubaos_core::setup;
use kubaos_core::setup::CardId;
use kubaos_core::setup::Env;
use kubaos_core::setup::Inputs;
use kubaos_core::setup::RealSys;
use kubaos_core::setup::SetupEvent;
use kubaos_core::setup::State;
use kubaos_core::setup::demo;

use crate::Events;
use crate::out::err;
use crate::out::out;

#[derive(Clone, Copy, PartialEq, ValueEnum)]
pub enum DemoMachine {
    Vm,
    Tuxedo,
}

#[derive(Subcommand)]
pub enum SetupCmd {
    /// Every card and its state on this machine.
    Status {
        #[arg(long)]
        json: bool,
        /// A made-up machine instead of this one.
        #[arg(long, value_enum)]
        demo: Option<DemoMachine>,
    },
    /// Do one card: tuxedo, fingerprint, yubikey, bitwarden, git or phone.
    Run {
        #[arg(value_parser = parse_card)]
        card: CardId,
        /// Read the inputs (passwords, codes) as one JSON object from stdin.
        #[arg(long)]
        input_stdin: bool,
        #[arg(long, value_enum, default_value = "human")]
        events: Events,
        #[arg(long, value_enum)]
        demo: Option<DemoMachine>,
    },
    /// Write the marker, so setup no longer opens at login.
    Done,
}

fn parse_card(s: &str) -> Result<CardId, String> {
    CardId::parse(s).ok_or_else(|| format!("unknown card {s}: tuxedo, fingerprint, yubikey, bitwarden, git, phone"))
}

fn machine(d: DemoMachine) -> demo::Machine {
    match d {
        DemoMachine::Vm => demo::Machine::Vm,
        DemoMachine::Tuxedo => demo::Machine::Tuxedo,
    }
}

pub fn main(cmd: &SetupCmd) -> ExitCode {
    if rustix::process::geteuid().is_root() {
        err!("kubaos setup: run it as yourself, not root; it asks for root when it needs it");
        return ExitCode::from(2);
    }
    let env = Env::current();
    match *cmd {
        SetupCmd::Status { json, demo } => {
            let st = match demo {
                Some(d) => demo::Demo::new(machine(d), 1.0).status(),
                None => setup::status(&mut RealSys::new(), &env),
            };
            if json {
                out!("{}", serde_json::to_string_pretty(&st).expect("status serializes"));
            } else {
                out!("online {}", if st.online { "yes" } else { "no" });
                for c in &st.cards {
                    out!("{:<12} {:<8} {}", c.id.title(), state_word(c.state), c.summary);
                }
                if env.marker().exists() {
                    out!("marker {} (setup stays closed at login)", env.marker().display());
                }
            }
            ExitCode::SUCCESS
        }
        SetupCmd::Run {
            card,
            input_stdin,
            events,
            demo,
        } => {
            let inputs = if input_stdin {
                let mut s = String::new();
                if let Err(e) = std::io::stdin().read_to_string(&mut s) {
                    return crate::usage_error(format!("reading stdin: {e}"));
                }
                match serde_json::from_str::<Inputs>(&s) {
                    Ok(i) => i,
                    Err(e) => return crate::usage_error(format!("inputs on stdin: {e}")),
                }
            } else {
                Inputs::default()
            };
            let mut print = |e: SetupEvent| print_event(&e, events);
            let ok = match demo {
                Some(d) => demo::Demo::new(machine(d), 1.0).run(card, &inputs, &mut print),
                None => setup::run(&mut RealSys::new(), &env, card, &inputs, &mut print),
            };
            if ok { ExitCode::SUCCESS } else { ExitCode::from(1) }
        }
        SetupCmd::Done => match setup::mark_done(&mut RealSys::new(), &env) {
            Ok(()) => {
                out!("{}", env.marker().display());
                ExitCode::SUCCESS
            }
            Err(e) => {
                err!("kubaos setup: {e}");
                ExitCode::from(1)
            }
        },
    }
}

fn state_word(s: State) -> &'static str {
    match s {
        State::Done => "done",
        State::Needed => "needed",
        State::Reboot => "reboot",
        State::Waiting => "waiting",
        State::Absent => "absent",
    }
}

fn print_event(e: &SetupEvent, events: Events) {
    if events == Events::Json {
        out!("{}", e.to_json_line());
        return;
    }
    match e {
        SetupEvent::Started { card } => out!("==> {}", card.title()),
        SetupEvent::Prompt { text, count, .. } => match count {
            Some((k, n)) => out!("  >> {text} ({k}/{n})"),
            None => out!("  >> {text}"),
        },
        SetupEvent::Log { level, line, .. } => match level {
            Level::Debug => {}
            Level::Info => out!("     {line}"),
            Level::Warn => out!("  !! {line}"),
            Level::Error => out!("  xx {line}"),
        },
        SetupEvent::Finished { status, .. } => {
            out!("  ok {}: {}", state_word(status.state), status.summary);
        }
        SetupEvent::Failed { error, hint, .. } => {
            out!("  xx Fuck. {error}.");
            if let Some(h) = hint {
                out!("     {h}");
            }
        }
    }
}
