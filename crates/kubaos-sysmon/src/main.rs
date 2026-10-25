//! `kubaos-sysmon`: the system monitor behind the bar's pill and its popup, and the changes the
//! Command menu makes to its settings.
//!
//! ```text
//! kubaos-sysmon serve              samples on stdout at the pace stdin asks for (the shell)
//! kubaos-sysmon sample             one detailed sample with processes, as JSON
//! kubaos-sysmon settings           the settings, as JSON
//! kubaos-sysmon set KEY VALUE      pill on|off|toggle, alerts on|off|toggle, style numbers|bars|sparklines,
//!                                  metrics cpu ram temp gpu net power, cpu-temp|ram|disk|battery-draw N
//! ```
//!
//! # Owns
//!
//! - The command line and its exit codes: 2 for a usage error, 1 when a setting was refused, with
//!   the reason on stderr, which the menu shows.
//!
//! # Not here
//!
//! - Everything else: the library's modules.

#![forbid(unsafe_code)]

use std::io;
use std::process::ExitCode;
use std::thread;
use std::time::Duration;

use kubaos_sysmon::Sampler;
use kubaos_sysmon::serve;
use kubaos_sysmon::serve::Server;
use kubaos_sysmon::settings;
use kubaos_sysmon::settings::Settings;

const USAGE: &str = "\
usage: kubaos-sysmon serve
       kubaos-sysmon sample
       kubaos-sysmon settings
       kubaos-sysmon set KEY VALUE

serve     stream samples for the shell: a line of JSON each, at the pace it asks for
sample    one detailed sample, processes included, as JSON
settings  the settings, as JSON
set       pill on|off|toggle, alerts on|off|toggle, style numbers|bars|sparklines,
          metrics cpu ram temp gpu net power (in order), cpu-temp|ram|disk|battery-draw NUMBER

Settings: ~/.config/kubaos/desktop/sysmon/sysmon.toml";

/// How long `sample` waits between its two reads, so CPU use and speeds have ticks to count.
const SAMPLE_GAP: Duration = Duration::from_millis(500);

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    match args.as_slice() {
        ["serve"] => {
            let mut server = Server::new("/", settings::dir(), serve::stream_file());
            match serve::run(&mut server, io::BufReader::new(io::stdin()), io::stdout().lock()) {
                Ok(()) => ExitCode::SUCCESS,
                // The shell went away: nobody is left to read the stream.
                Err(err) if err.kind() == io::ErrorKind::BrokenPipe => ExitCode::SUCCESS,
                Err(err) => failed(&err),
            }
        }
        ["sample"] => {
            let mut sampler = Sampler::new("/");
            sampler.sample(serve::now_ms(), true, true);
            thread::sleep(SAMPLE_GAP);
            println!("{}", serve::sample_line(&sampler.sample(serve::now_ms(), true, true)));
            ExitCode::SUCCESS
        }
        ["settings"] => {
            let path = settings::dir().map(|dir| dir.join(settings::FILE));
            match path.as_deref().map_or_else(|| Ok(Settings::default()), Settings::load) {
                Ok(settings) => {
                    println!("{}", serde_json::to_string(&settings).unwrap_or_default());
                    ExitCode::SUCCESS
                }
                Err(err) => failed(&err),
            }
        }
        ["set", key, value @ ..] if !value.is_empty() => {
            let Some(dir) = settings::dir() else {
                eprintln!("kubaos-sysmon: neither XDG_CONFIG_HOME nor HOME is set");
                return ExitCode::FAILURE;
            };
            match settings::set(&dir.join(settings::FILE), key, &value.join(" ")) {
                Ok(()) => ExitCode::SUCCESS,
                Err(err) => failed(&err),
            }
        }
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

fn failed(err: &dyn std::error::Error) -> ExitCode {
    eprintln!("kubaos-sysmon: {}", serve::chain(err));
    ExitCode::FAILURE
}
