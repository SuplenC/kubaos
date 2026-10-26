//! `kubaos-phone`: the phone link from a terminal and from the Command menu.
//!
//! ```text
//! kubaos-phone status [--json]        on or off, the name phones see, and each device
//! kubaos-phone on|off                 turn the link on (kdeconnectd starts) or off (it stops)
//! kubaos-phone accept DEVICE          accept DEVICE's pairing request (its id or name)
//! kubaos-phone send-clipboard [PHONE] send the laptop's clipboard to the paired phone
//! kubaos-phone send-file PATH [PHONE] send a file to the paired phone
//! kubaos-phone ring [PHONE]           ring the paired phone, even on silent
//! kubaos-phone unpair [PHONE]         forget the paired phone
//! kubaos-phone restrict               turn off every plugin the phone link isn't for, on every
//!                                     device (the user unit runs it as kdeconnectd starts)
//! ```
//!
//! # Owns
//!
//! - The command line and its exit codes: 2 for a usage error, 1 when the command failed, with
//!   the reason and what to do on stderr. Every command but `status` and `restrict` also says how
//!   it went in a notification, since the menu runs them with no terminal.
//!
//! # Not here
//!
//! - Everything else: the library's modules.

#![forbid(unsafe_code)]

use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitCode;

use kubaos_phone::Device;
use kubaos_phone::Error;
use kubaos_phone::KdeConnect;
use kubaos_phone::Status;
use kubaos_phone::phone;

const USAGE: &str = "\
usage: kubaos-phone status [--json]
       kubaos-phone on|off
       kubaos-phone accept DEVICE
       kubaos-phone send-clipboard [PHONE]
       kubaos-phone send-file PATH [PHONE]
       kubaos-phone ring [PHONE]
       kubaos-phone unpair [PHONE]
       kubaos-phone restrict

status          on or off, the name phones see this laptop by, and each device KDE Connect knows
on|off          set up the phone link (KDE Connect starts), or turn it off (it stops)
accept          accept the pairing request of DEVICE, by its id or name
send-clipboard  send the laptop's clipboard to the paired phone, or to PHONE
send-file       send the file PATH to the paired phone, or to PHONE
ring            ring the paired phone, even when it is on silent
unpair          forget the paired phone, or PHONE
restrict        turn off every KDE Connect plugin the phone link isn't for, on every device";

const ICON: &str = "phone";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let mut link = KdeConnect::default();
    let link = &mut link;
    let done = match args.as_slice() {
        ["status"] => return show(&phone::status(link), false),
        ["status", "--json"] => return show(&phone::status(link), true),
        ["restrict"] => return restrict(link),
        ["on"] => phone::turn_on(link).map(|st| on_message(&st)),
        ["off"] => phone::turn_off(link).map(|()| "Phone link off. KDE Connect stopped.".to_string()),
        ["accept", device] => phone::accept(link, device).map(|d| format!("Paired with {}.", d.name)),
        ["send-clipboard", rest @ ..] if rest.len() <= 1 => {
            phone::send_clipboard(link, rest.first().copied()).map(|d| format!("Clipboard sent to {}.", d.name))
        }
        ["send-file", path, rest @ ..] if rest.len() <= 1 => {
            let home = std::env::var_os("HOME").map(PathBuf::from);
            let path = phone::typed_path(path, home.as_deref());
            phone::send_file(link, rest.first().copied(), &path).map(|d| sent_file(&path, &d))
        }
        ["ring", rest @ ..] if rest.len() <= 1 => {
            phone::ring(link, rest.first().copied()).map(|d| format!("Ringing {}.", d.name))
        }
        ["unpair", rest @ ..] if rest.len() <= 1 => {
            phone::unpair(link, rest.first().copied()).map(|d| format!("{} unpaired.", d.name))
        }
        ["-h" | "--help"] => {
            println!("{USAGE}");
            return ExitCode::SUCCESS;
        }
        _ => {
            eprintln!("{USAGE}");
            return ExitCode::from(2);
        }
    };
    match done {
        Ok(message) => {
            println!("{message}");
            notify("Phone", &message);
            ExitCode::SUCCESS
        }
        Err(err) => {
            let reason = chain(&err);
            eprintln!("kubaos-phone: {reason}\n{}", err.hint());
            notify(&capital(&reason), err.hint());
            ExitCode::FAILURE
        }
    }
}

/// The plugin restriction, quiet but for what it turned off: it runs at every start of the unit.
fn restrict(link: &mut KdeConnect) -> ExitCode {
    match phone::restrict(link) {
        Ok(changed) => {
            for line in changed {
                println!("{line}");
            }
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("kubaos-phone: {}\n{}", chain(&err), err.hint());
            ExitCode::FAILURE
        }
    }
}

fn on_message(st: &Status) -> String {
    let name = st.name.as_deref().unwrap_or("this laptop");
    format!(
        "Phone link on. Open KDE Connect on the phone, on the same Wi-Fi, and pair with {name}: \
         \"Set up a phone\" in the Command menu shows the request."
    )
}

fn sent_file(path: &Path, phone: &Device) -> String {
    let file = path.file_name().map_or_else(|| path.display().to_string(), |f| f.to_string_lossy().into_owned());
    format!("{file} sent to {}.", phone.name)
}

/// The status for a person, or as JSON for the shell and scripts.
fn show(st: &Status, json: bool) -> ExitCode {
    if json {
        match serde_json::to_string_pretty(st) {
            Ok(text) => println!("{text}"),
            Err(err) => {
                eprintln!("kubaos-phone: {err}");
                return ExitCode::FAILURE;
            }
        }
        return ExitCode::SUCCESS;
    }
    let name = st.name.as_deref().unwrap_or("?");
    match (st.on, st.running) {
        (false, _) => println!("off (KDE Connect would announce this laptop as {name})"),
        (true, false) => println!("on, but KDE Connect isn't running"),
        (true, true) => println!("on, announced as {name} at {}", st.addresses.join(", ")),
    }
    for d in &st.devices {
        let state = match (d.paired, d.reachable, d.wants_to_pair) {
            (_, _, true) => "asks to pair",
            (true, true, _) => "paired, in reach",
            (true, false, _) => "paired, out of reach",
            (false, _, _) => "seen on the network",
        };
        let battery = d.battery.map(|b| format!(", battery {}%", b.charge)).unwrap_or_default();
        println!("{}\t{}\t{state}{battery}", d.id, d.name);
    }
    ExitCode::SUCCESS
}

/// The error and its causes, each once.
fn chain(err: &Error) -> String {
    let mut text = err.to_string();
    let mut cause = std::error::Error::source(err);
    while let Some(c) = cause {
        text.push_str(": ");
        text.push_str(&c.to_string());
        cause = c.source();
    }
    text
}

fn capital(s: &str) -> String {
    let mut chars = s.chars();
    chars
        .next()
        .map(|first| first.to_uppercase().chain(chars).collect())
        .unwrap_or_default()
}

/// A notification through `notify-send` (libnotify-bin, `system/packages.toml`). Without it the
/// notification only goes unseen; the result is on stdout and stderr too.
fn notify(summary: &str, body: &str) {
    let _ = Command::new("notify-send")
        .args(["--app-name=Phone", &format!("--icon={ICON}"), summary, body])
        .status();
}
