//! `kubaos suite update` and `kubaos suite upgrade`: the installed Suites' tools moved to their
//! newest patch releases, or past their requests to the newest minor and major ones.
//!
//!   kubaos suite update [NAME...] [--all] [--json]
//!   kubaos suite upgrade [NAME...] [--all] [--json] [--no-switch] [--detach | --foreground]
//!
//! Without a name or `--all`, each shows the Suites that have something newer, with each tool
//! `old → new`, and asks which to move. `--json` only lists that, for the named Suites or all of
//! them, and changes nothing. An upgrade that moves a Suite's main toolchain to another of its
//! lines switches, and puts the Suite back as it was when that switch fails. Neither reads or
//! changes a project's `mise.toml`.
//!
//! # Owns
//!
//! - The two commands' lines, the pick list and what they print.
//!
//! # Not here
//!
//! - What moves and how: `kubaos_core::suites` (`updates.rs`).

use std::io;
use std::io::BufRead;
use std::io::IsTerminal;
use std::io::Write;
use std::process::ExitCode;

use kubaos_core::state::Paths;
use kubaos_core::suites::Available;
use kubaos_core::suites::Reach;
use kubaos_core::suites::Suite;
use kubaos_core::suites::SuiteUpdates;
use kubaos_core::suites::Suites;
use kubaos_core::suites::catalog;
use serde_json::Value;
use serde_json::json;

use crate::job;
use crate::job::SwitchMode;
use crate::job::report;
use crate::out::err;
use crate::out::out;
use crate::usage_error;

/// Which Suites a command moves, as its line names them.
pub struct Chosen {
    pub names: Vec<String>,
    pub all: bool,
    pub json: bool,
}

/// What an upgrade does after it moves the tools.
pub struct AfterUpgrade {
    pub no_switch: bool,
    pub mode: SwitchMode,
}

/// What [`pick`] found to move.
enum Pick {
    Suites(Vec<&'static Suite>),
    /// Nothing to move, or only a list asked for; it is printed already.
    Done(ExitCode),
}

pub fn update(suites: &Suites, chosen: &Chosen) -> ExitCode {
    let picked = match pick(suites, chosen, Reach::Update) {
        Pick::Suites(picked) => picked,
        Pick::Done(code) => return code,
    };
    match suites.update(&picked) {
        Ok(moved) => {
            print_moved("updated", &moved);
            ExitCode::SUCCESS
        }
        Err(e) => report(e),
    }
}

pub fn upgrade(paths: &Paths, suites: &Suites, chosen: &Chosen, after: &AfterUpgrade) -> ExitCode {
    let picked = match pick(suites, chosen, Reach::Upgrade) {
        Pick::Suites(picked) => picked,
        Pick::Done(code) => return code,
    };
    let upgraded = match suites.upgrade(&picked) {
        Ok(upgraded) => upgraded,
        Err(e) => return report(e),
    };
    print_moved("upgraded", &upgraded.moved);
    if upgraded.lines.is_empty() {
        return ExitCode::SUCCESS;
    }
    for (suite, _) in &upgraded.lines {
        if let Ok(Some(config)) = suites.config(suite) {
            out!(
                "the {} Suite is on version {} now",
                suite.title,
                config.version.unwrap_or_default()
            );
        }
    }
    if after.no_switch {
        return ExitCode::SUCCESS;
    }
    let code = job::switch(paths, after.mode);
    if code == ExitCode::SUCCESS || !after.mode.waits() {
        return code;
    }
    for (suite, before) in &upgraded.lines {
        match suites.restore(suite, before) {
            Ok(()) => err!(
                "kubaos: the switch failed, so the {} Suite is back as it was",
                suite.title
            ),
            Err(e) => {
                err!(
                    "kubaos: the switch failed, and putting the {} Suite back failed too",
                    suite.title
                );
                return report(e);
            }
        }
    }
    code
}

/// The Suites to move: those `chosen` names, every one with something newer for `--all`, or
/// the ones picked from the list.
fn pick(suites: &Suites, chosen: &Chosen, reach: Reach) -> Pick {
    let mut named = Vec::new();
    for name in &chosen.names {
        let Some(suite) = catalog::find(name) else {
            return Pick::Done(usage_error(format!("no Suite called {name:?}")));
        };
        match suites.config(suite) {
            Ok(Some(_)) => named.push(suite),
            Ok(None) => {
                return Pick::Done(usage_error(format!(
                    "the {} Suite is not installed; add it with `kubaos suite add {}`",
                    suite.title, suite.name
                )));
            }
            Err(e) => return Pick::Done(report(e)),
        }
    }
    let available = match suites.available(reach) {
        Ok(available) => available,
        Err(e) => return Pick::Done(report(e)),
    };
    let available: Vec<SuiteUpdates> = available
        .into_iter()
        .filter(|s| named.is_empty() || named.iter().any(|n| n.name == s.suite.name))
        .collect();
    if chosen.json {
        out!(
            "{}",
            serde_json::to_string_pretty(&available_json(&available)).expect("the list serializes")
        );
        return Pick::Done(ExitCode::SUCCESS);
    }
    if available.is_empty() {
        out!("{}", nothing(reach));
        return Pick::Done(ExitCode::SUCCESS);
    }
    if chosen.all || !named.is_empty() {
        return Pick::Suites(available.iter().map(|s| s.suite).collect());
    }
    if !io::stdin().is_terminal() {
        return Pick::Done(usage_error(
            "name the Suites to move, or pass --all: there is no terminal to pick them in",
        ));
    }
    print_list(&available, reach);
    let Some(answer) = ask("Which? Their numbers (1 3), Enter for all, n for none:") else {
        return Pick::Done(ExitCode::SUCCESS);
    };
    match parse_pick(&answer, available.len()) {
        Ok(picked) => Pick::Suites(picked.into_iter().map(|i| available[i].suite).collect()),
        Err(e) => Pick::Done(usage_error(e)),
    }
}

fn nothing(reach: Reach) -> &'static str {
    match reach {
        Reach::Update => "every Suite's tools are at their newest patch releases",
        Reach::Upgrade => "no Suite's tools have a newer minor or major release",
    }
}

/// The numbered list the pick reads from: a Suite a number, its tools under it.
fn print_list(available: &[SuiteUpdates], reach: Reach) {
    out!(
        "{}",
        match reach {
            Reach::Update => "Suites with patch releases:",
            Reach::Upgrade => "Suites with minor or major releases:",
        }
    );
    for (i, s) in available.iter().enumerate() {
        out!("  {}  {}", i + 1, s.suite.title);
        for tool in &s.tools {
            out!("       {}", describe(tool));
        }
    }
}

/// Asks `question` on the terminal; `None` when it closes without an answer.
#[expect(
    clippy::print_stderr,
    reason = "The question stays on the line the answer is typed on, as `read -p` leaves it."
)]
fn ask(question: &str) -> Option<String> {
    eprint!("{question} ");
    let _ = io::stderr().flush();
    let mut line = String::new();
    match io::stdin().lock().read_line(&mut line) {
        Ok(n) if n > 0 => Some(line.trim().to_owned()),
        _ => None,
    }
}

/// The indices `answer` picks from a list of `count`: every one for an empty answer, none for
/// `n`, else the numbers it lists, 1 for the first, separated by spaces or commas.
pub fn parse_pick(answer: &str, count: usize) -> Result<Vec<usize>, String> {
    let answer = answer.trim();
    if answer.is_empty() {
        return Ok((0..count).collect());
    }
    if matches!(answer, "n" | "N" | "no" | "No") {
        return Ok(Vec::new());
    }
    let mut picked = Vec::new();
    for word in answer.split([' ', ',']).filter(|w| !w.is_empty()) {
        let n: usize = word
            .parse()
            .ok()
            .filter(|n| (1..=count).contains(n))
            .ok_or_else(|| format!("{word:?} isn't one of the numbers 1 to {count}"))?;
        if !picked.contains(&(n - 1)) {
            picked.push(n - 1);
        }
    }
    picked.sort_unstable();
    Ok(picked)
}

/// "kubaos-test-tool 1.0.0 → 1.0.1", or for an upgrade "kubaos-test-tool 1.0 → 3.0 (3.0.0)".
fn describe(tool: &Available) -> String {
    let from = tool.current.as_deref().unwrap_or("not installed");
    match &tool.target {
        Some(target) => format!("{} {} → {target} ({})", tool.tool, tool.request, tool.latest),
        None => format!("{} {from} → {}", tool.tool, tool.latest),
    }
}

fn print_moved(verb: &str, moved: &[SuiteUpdates]) {
    for s in moved {
        for tool in &s.tools {
            out!("{verb} {}: {}", s.suite.name, describe(tool));
        }
    }
}

fn available_json(available: &[SuiteUpdates]) -> Value {
    available
        .iter()
        .map(|s| {
            json!({
                "name": s.suite.name,
                "title": s.suite.title,
                "tools": s.tools.iter().map(|t| json!({
                    "tool": t.tool,
                    "request": t.request,
                    "current": t.current,
                    "latest": t.latest,
                    "target": t.target,
                })).collect::<Vec<_>>(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::parse_pick;

    #[test]
    fn an_empty_answer_picks_every_suite_and_n_none() {
        assert_eq!(parse_pick("", 3), Ok(vec![0, 1, 2]));
        assert_eq!(parse_pick("  n ", 3), Ok(vec![]));
    }

    #[test]
    fn numbers_pick_those_suites_once_in_list_order() {
        assert_eq!(parse_pick("3 1,3", 3), Ok(vec![0, 2]));
    }

    #[test]
    fn a_number_off_the_list_is_refused() {
        assert!(parse_pick("4", 3).is_err());
        assert!(parse_pick("0", 3).is_err());
        assert!(parse_pick("go", 3).is_err());
    }
}
