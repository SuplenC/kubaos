//! The KubaOS Extension's protocol loop: each config the Daemon sends makes
//! the Items over, sending only the ones that changed, and each run is
//! answered with its outcome, after the View it pushes or pops.
//!
//! # Owns
//!
//! - The loop, logging the config's problems, and turning what a run came to into messages.
//!
//! # Not here
//!
//! - The Items and the runs: the library's modules.

#![forbid(unsafe_code)]

use std::process::ExitCode;

use kotodama_extension::Connection;
use kotodama_extension::Event;
use kotodama_extension::SentItems;
use kotodama_extension::SentStatus;
use kotodama_extension::config;
use kotodama_kubaos::bus::SystemBus;
use kotodama_kubaos::extension::Extension;
use kotodama_kubaos::run::Programs;
use kotodama_kubaos::toggles::Ran;
use kotodama_protocol::Error;
use kotodama_protocol::FromExtension;
use kotodama_protocol::Outcome;
use kotodama_protocol::ToExtension;

fn main() -> ExitCode {
    kotodama_extension::main("kotodama-kubaos", run)
}

fn run() -> Result<(), Error> {
    let Connection {
        mut writer, received, ..
    } = kotodama_extension::connect::<()>()?;
    let mut extension = Extension::new(SystemBus, Programs);
    let mut sent = SentItems::default();
    // The toggles work on any machine, and the actions need only their programs.
    SentStatus::default().usable(&mut writer)?;

    for event in received {
        match event {
            Event::Message(ToExtension::Config(table)) => {
                for problem in extension.configure(&table) {
                    config::warn(problem);
                }
                sent.publish(&mut writer, extension.items())?;
            }
            Event::Message(ToExtension::RunAction {
                invocation,
                action,
                item,
                arguments,
            }) => {
                let outcome = match extension.run(&action, &item, &arguments) {
                    Ok(Ran::Close) => Outcome::Close,
                    Ok(Ran::KeepOpen) => Outcome::KeepOpen,
                    Ok(Ran::Push(view)) => {
                        writer.send(&FromExtension::PushView(view))?;
                        Outcome::KeepOpen
                    }
                    Ok(Ran::Pop(view)) => {
                        writer.send(&FromExtension::PopView { view })?;
                        Outcome::Close
                    }
                    Err(message) => Outcome::Failed { message },
                };
                writer.send(&FromExtension::ActionDone { invocation, outcome })?;
            }
            Event::Message(_) | Event::Own(()) => {}
            Event::Hangup => break,
        }
    }
    Ok(())
}
