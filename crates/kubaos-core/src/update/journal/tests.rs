use std::fs;

use super::Journal;
use super::TAIL_LINES;
use super::strip_escapes;
use crate::update::UpdateError;
use crate::update::event::Command;
use crate::update::event::Outcome;
use crate::update::event::Phase;
use crate::update::event::UpdateEvent;
use crate::update::machine::Cmd;
use crate::update::machine::MachineError;

#[test]
fn colours_cursor_moves_and_titles_are_stripped_from_output() {
    let line = "\x1b[1;32mok\x1b[0m done\x1b]0;title\x07 \x1b[2K\tnext";

    assert_eq!(strip_escapes(line), "ok done  next");
}

#[test]
fn a_failure_carries_the_phase_and_the_end_of_the_failing_output() {
    let mut events = Vec::new();
    let mut j = Journal::new();
    j.send_to(&mut events);
    let cmd = Cmd::root("apt-get").arg("upgrade");

    j.phase_start(Phase::SystemPackages);
    j.command(&cmd);
    for n in 1..=30 {
        j.output(&format!("line {n}"));
    }
    j.phase_end(false);
    j.failed(&UpdateError::Step {
        what: "upgrading apt packages".into(),
        source: MachineError::exited(&cmd, Some(100), ""),
    });

    let Some(UpdateEvent::Failed {
        phase, error, output, ..
    }) = events.last()
    else {
        panic!("no failed event: {events:?}");
    };
    assert_eq!(*phase, Some(Phase::SystemPackages));
    assert_eq!(
        error,
        "upgrading apt packages: `sudo apt-get upgrade` exited with code 100"
    );
    assert_eq!(output.len(), TAIL_LINES);
    assert_eq!(output.first().map(String::as_str), Some("line 11"));
    assert!(events.iter().any(|e| matches!(
        e,
        UpdateEvent::PhaseFinished {
            outcome: Outcome::Failed,
            ..
        }
    )));
}

#[test]
fn the_output_of_a_program_that_succeeded_is_no_failures() {
    let mut events = Vec::new();
    let mut j = Journal::new();
    j.send_to(&mut events);

    j.command(&Cmd::user("nix"));
    j.output("all good");
    j.succeeded();
    j.failed(&UpdateError::NoUpgrade);

    assert!(matches!(events.last(), Some(UpdateEvent::Failed { output, .. }) if output.is_empty()));
}

#[test]
fn the_switch_and_nix_tell_what_reloaded_and_which_inputs_moved() {
    let mut j = Journal::new();

    j.output("dot-session: reloaded Hyprland");
    j.output("dot-session: restarting Quickshell");
    j.output("\x1b[1m•\x1b[0m Updated input 'nixpkgs':");
    j.output("• Updated input 'home-manager':");
    j.output("• Updated input 'nixpkgs':");

    assert_eq!(j.reloaded, ["reloaded Hyprland", "restarting Quickshell"]);
    assert_eq!(j.inputs, ["nixpkgs", "home-manager"]);
}

#[test]
fn apt_setting_up_lines_count_as_progress() {
    let mut events = Vec::new();
    let mut j = Journal::new();
    j.send_to(&mut events);

    j.phase_start(Phase::SystemPackages);
    j.count("Setting up ", 2);
    for line in ["Unpacking vim", "Setting up vim", "Setting up git", "Setting up extra"] {
        j.output(line);
    }

    let progress: Vec<(usize, usize)> = events
        .iter()
        .filter_map(|e| match e {
            UpdateEvent::Progress { done, total } => Some((*done, *total)),
            _ => None,
        })
        .collect();
    assert_eq!(progress, [(1, 2), (2, 2)]);
}

#[test]
fn the_log_keeps_every_line_and_only_the_newest_logs_stay() {
    let dir = tempfile::tempdir().unwrap();
    for n in 0..12 {
        fs::write(dir.path().join(format!("2026-01-{n:02}T00-00-00-update.log")), "old").unwrap();
    }
    let mut j = Journal::new();

    j.open_log(dir.path(), "2026-10-23T10-00-00", Command::Update).unwrap();
    j.phase_start(Phase::Home);
    j.command(&Cmd::user("nix").args(["flake", "update"]));
    j.output("• Updated input 'nixpkgs':");
    j.warn("snapper isn't set up");

    let path = j.log_path().unwrap();
    assert_eq!(path, dir.path().join("2026-10-23T10-00-00-update.log"));
    let text = fs::read_to_string(&path).unwrap();
    for wanted in [
        "== Home",
        "$ nix flake update",
        "   • Updated input 'nixpkgs':",
        "warning: snapper",
    ] {
        assert!(text.contains(wanted), "{wanted:?} not in the log:\n{text}");
    }
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 10);
    assert!(!dir.path().join("2026-01-02T00-00-00-update.log").exists());
    assert!(dir.path().join("2026-01-03T00-00-00-update.log").exists());
}
