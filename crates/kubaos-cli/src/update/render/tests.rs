use std::io;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use std::time::Instant;

use kubaos_core::update::event::Command;
use kubaos_core::update::event::GroupChange;
use kubaos_core::update::event::Outcome;
use kubaos_core::update::event::Phase;
use kubaos_core::update::event::Preview;
use kubaos_core::update::event::Prompt;
use kubaos_core::update::event::Release;
use kubaos_core::update::event::Restart;
use kubaos_core::update::event::Summary;
use kubaos_core::update::event::UpdateEvent;
use kubaos_core::update::record::Channel;

use super::Json;
use super::Plain;
use super::Screen;
use super::Style;
use super::Text;

/// Bytes written, shared with the test.
#[derive(Clone, Default)]
struct Shared(Arc<Mutex<Vec<u8>>>);

impl Write for Shared {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Shared {
    fn bytes(&self) -> Vec<u8> {
        self.0.lock().unwrap().clone()
    }

    fn text(&self) -> String {
        String::from_utf8(self.bytes()).unwrap()
    }
}

/// What a terminal shows after `bytes`: it moves on `\n` and `\r`, erases a line on `ESC[2K`, goes
/// up on `ESC[1A`, and draws no colours.
fn terminal(bytes: &[u8]) -> String {
    let text = String::from_utf8_lossy(bytes);
    let mut rows: Vec<Vec<char>> = vec![Vec::new()];
    let (mut row, mut col) = (0, 0);
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        match c {
            '\n' => {
                row += 1;
                col = 0;
                if rows.len() == row {
                    rows.push(Vec::new());
                }
            }
            '\r' => col = 0,
            '\x1b' => {
                assert_eq!(chars.next(), Some('['), "an escape that is not CSI");
                let mut params = String::new();
                let end = loop {
                    let c = chars.next().expect("an unfinished escape");
                    if c.is_ascii_alphabetic() {
                        break c;
                    }
                    params.push(c);
                };
                match (params.as_str(), end) {
                    ("2", 'K') => rows[row].clear(),
                    ("1", 'A') => row -= 1,
                    (_, 'm') => {}
                    other => panic!("an escape the live view doesn't use: {other:?}"),
                }
            }
            c => {
                let line = &mut rows[row];
                while line.len() < col {
                    line.push(' ');
                }
                if col < line.len() {
                    line[col] = c;
                } else {
                    line.push(c);
                }
                col += 1;
            }
        }
    }
    let lines: Vec<String> = rows.iter().map(|r| r.iter().collect::<String>().trim_end().to_owned()).collect();
    lines.join("\n").trim_end().to_owned()
}

fn release(version: &str, codename: Option<&str>, commit: &str) -> Release {
    Release {
        version: version.into(),
        codename: codename.map(Into::into),
        commit: commit.into(),
    }
}

fn finished(phase: Phase, elapsed_ms: u64, detail: Option<&str>) -> UpdateEvent {
    UpdateEvent::PhaseFinished {
        phase,
        outcome: Outcome::Done,
        elapsed_ms,
        detail: detail.map(Into::into),
    }
}

fn output(line: &str) -> UpdateEvent {
    UpdateEvent::Output { line: line.into() }
}

fn step(text: &str) -> UpdateEvent {
    UpdateEvent::Step { text: text.into() }
}

const LOG: &str = "/home/kuba/.local/state/kubaos/update/2026-10-23T09-14-03-update.log";

/// An edge update that asks, runs a migration and reloads the session, as the updater reports it.
fn edge_update() -> Vec<UpdateEvent> {
    let from = release("0.1.0", Some("shinjuku"), "1a2b3c4d5e");
    let to = release("0.1.1", Some("shinjuku"), "9f8e7d6c5b");
    let changes = vec![
        "9f8e7d6 fix(hypr): keep each screen's workspaces as a group".to_owned(),
        "d8c8ecf docs: describe workspace groups".to_owned(),
    ];
    vec![
        UpdateEvent::Started {
            command: Command::Update,
            log: Some(LOG.into()),
        },
        UpdateEvent::PhaseStarted { phase: Phase::Checking },
        finished(Phase::Checking, 40, Some("KubaOS Shinjuku 0.1.0 (1a2b3c4), edge")),
        UpdateEvent::PhaseStarted { phase: Phase::Fetching },
        step("git fetch origin"),
        finished(Phase::Fetching, 1240, None),
        UpdateEvent::PhaseStarted { phase: Phase::Verifying },
        UpdateEvent::Warning {
            text: "no allowed_signers on this machine: main's head is taken unchecked".into(),
        },
        finished(Phase::Verifying, 30, Some("found main at 9f8e7d6 (0.1.1)")),
        UpdateEvent::PhaseStarted { phase: Phase::Reviewing },
        step("Refreshing apt's package lists"),
        finished(Phase::Reviewing, 3410, Some("12 apt packages, 2 commits")),
        UpdateEvent::Preview(Preview {
            from: from.clone(),
            to: Some(to.clone()),
            channel: Channel::Edge,
            changes: changes.clone(),
            packages: 12,
            apt: vec![
                GroupChange {
                    group: "files".into(),
                    installed: vec!["loupe".into(), "showtime".into()],
                    removed: Vec::new(),
                },
                GroupChange {
                    group: "tools".into(),
                    installed: vec!["jq".into()],
                    removed: Vec::new(),
                },
            ],
            hyprland_pin: false,
            migrations: vec!["1792765570-backlight-rule.sh".into()],
        }),
        UpdateEvent::Prompt {
            prompt: Prompt::Confirm {
                question: "Update now?".into(),
            },
        },
        UpdateEvent::Answered { yes: true },
        UpdateEvent::PhaseStarted { phase: Phase::Snapshot },
        finished(Phase::Snapshot, 820, Some("before kubaos update 2026-10-23 09:14:03 (0.1.0 → 0.1.1)")),
        UpdateEvent::PhaseStarted {
            phase: Phase::SystemPackages,
        },
        step("apt-get upgrade"),
        output("Unpacking firefox (131.0) over (130.0) ..."),
        UpdateEvent::Progress { done: 1, total: 12 },
        output("Setting up firefox (131.0) ..."),
        finished(Phase::SystemPackages, 42_300, Some("12 packages")),
        UpdateEvent::PhaseStarted { phase: Phase::Release },
        finished(Phase::Release, 110, Some("release is at 9f8e7d6")),
        UpdateEvent::PhaseStarted { phase: Phase::Manifest },
        step("Installing loupe, showtime"),
        finished(Phase::Manifest, 5_400, Some("2 installed, 1 removed")),
        UpdateEvent::PhaseStarted {
            phase: Phase::SystemMigrations,
        },
        step("1792765570-backlight-rule.sh"),
        finished(Phase::SystemMigrations, 310, Some("1 migration ran")),
        UpdateEvent::PhaseStarted { phase: Phase::Inputs },
        output("• Updated input 'nixpkgs':"),
        finished(Phase::Inputs, 18_200, Some("nixpkgs")),
        UpdateEvent::PhaseStarted { phase: Phase::Home },
        step("Switching your home"),
        output("dot-session: reloaded Hyprland"),
        output("dot-session: restarting Quickshell"),
        finished(Phase::Home, 65_000, Some("reloaded Hyprland, restarting Quickshell")),
        UpdateEvent::Summary(Summary {
            command: Command::Update,
            from,
            to,
            channel: Channel::Edge,
            changes,
            packages: 12,
            apt: vec![
                GroupChange {
                    group: "files".into(),
                    installed: vec!["loupe".into(), "showtime".into()],
                    removed: Vec::new(),
                },
                GroupChange {
                    group: "tools".into(),
                    installed: Vec::new(),
                    removed: vec!["old-tool".into()],
                },
            ],
            inputs: vec!["nixpkgs".into()],
            migrations: vec!["1792765570-backlight-rule.sh".into()],
            reloaded: vec!["reloaded Hyprland".into(), "restarting Quickshell".into()],
            restart: Restart::None,
            restart_reason: None,
            undo: Some("kubaos rollback".into()),
            log: Some(LOG.into()),
        }),
    ]
}

fn text(style: Style) -> Text {
    Text::new(style, Some(PathBuf::from("/home/kuba")))
}

fn screen(style: Style, width: usize) -> (Screen<Shared>, Shared) {
    let out = Shared::default();
    (Screen::new(out.clone(), text(style), move || width), out)
}

/// Plays `events` on a screen, a second apart, with the user's answer echoed after a question.
fn play(screen: &mut Screen<Shared>, out: &Shared, events: &[UpdateEvent]) -> Instant {
    let mut now = Instant::now();
    for event in events {
        screen.event(event, now);
        screen.tick(now);
        if matches!(event, UpdateEvent::Prompt { prompt: Prompt::Confirm { .. } }) {
            out.clone().write_all(b"y\n").unwrap();
        }
        now += Duration::from_secs(1);
    }
    now
}

const EDGE_UPDATE_SCREEN: &str = "\
KubaOS update

  ✓ Checking            0.0s  KubaOS Shinjuku 0.1.0 (1a2b3c4), edge
  ✓ Fetching            1.2s
    ! no allowed_signers on this machine: main's head is taken unchecked
  ✓ Verifying           0.0s  found main at 9f8e7d6 (0.1.1)
  ✓ Reviewing changes   3.4s  12 apt packages, 2 commits

  KubaOS Shinjuku 0.1.0 (1a2b3c4) → Shinjuku 0.1.1 (9f8e7d6), edge
    9f8e7d6 fix(hypr): keep each screen's workspaces as a group
    d8c8ecf docs: describe workspace groups
  12 apt packages to upgrade
  New apt packages to install:
    files: loupe, showtime
    tools: jq
  System migrations to run: 1792765570-backlight-rule.sh

  Update now? [Y/n] y
  ✓ Snapshot            0.8s  before kubaos update 2026-10-23 09:14:03 (0.1.0 → 0.1.1)
  ✓ System packages      42s  12 packages
  ✓ Release             0.1s  release is at 9f8e7d6
  ✓ Package manifest    5.4s  2 installed, 1 removed
  ✓ System migrations   0.3s  1 migration ran
  ✓ Nix inputs           18s  nixpkgs
  ✓ Home               1m05s  reloaded Hyprland, restarting Quickshell

  Updated     Shinjuku 0.1.0 (1a2b3c4) → Shinjuku 0.1.1 (9f8e7d6), edge
  Changes     fix(hypr): keep each screen's workspaces as a group
              docs: describe workspace groups
  Packages    12 apt packages, Nix inputs nixpkgs
  Apt         files: installed loupe, showtime
              tools: removed old-tool
  Migrations  1792765570-backlight-rule.sh
  Session     reloaded Hyprland, restarting Quickshell
  Next        nothing, it's all running
  Undo        kubaos rollback
  Log         ~/.local/state/kubaos/update/2026-10-23T09-14-03-update.log";

#[test]
fn the_live_view_leaves_each_phase_with_its_result_and_ends_on_the_summary() {
    let (mut screen, out) = screen(Style::COLOR, 100);

    play(&mut screen, &out, &edge_update());

    let shown = terminal(&out.bytes());
    println!("{shown}");
    assert_eq!(shown, EDGE_UPDATE_SCREEN);
}

#[test]
fn the_live_area_shows_the_running_phase_its_step_and_the_last_line_of_output() {
    let (mut screen, out) = screen(Style::COLOR, 100);
    let start = Instant::now();

    screen.event(&UpdateEvent::PhaseStarted { phase: Phase::SystemPackages }, start);
    screen.event(&step("apt-get upgrade"), start);
    screen.event(&output("Unpacking vim (2:9.1) over (2:9.0) ..."), start);
    screen.event(&UpdateEvent::Progress { done: 3, total: 12 }, start);
    screen.tick(start + Duration::from_secs(12));

    assert_eq!(
        terminal(&out.bytes()),
        "  ⠙ System packages      12s  3/12  apt-get upgrade\n    Unpacking vim (2:9.1) over (2:9.0) ..."
    );
}

#[test]
fn a_prompt_clears_the_live_area_and_nothing_is_drawn_until_it_is_answered() {
    let (mut screen, out) = screen(Style::COLOR, 100);
    let now = Instant::now();
    screen.event(&UpdateEvent::PhaseStarted { phase: Phase::Fetching }, now);
    screen.event(&output("remote: Counting objects"), now);

    screen.event(&UpdateEvent::Prompt { prompt: Prompt::Password }, now);
    out.clone().write_all(b"[sudo] password for kuba: \n").unwrap();
    screen.tick(now + Duration::from_secs(1));

    assert_eq!(
        terminal(&out.bytes()),
        "  sudo asks for your password, for the steps that run as root.\n[sudo] password for kuba:"
    );

    screen.event(&UpdateEvent::Answered { yes: true }, now);

    assert!(terminal(&out.bytes()).ends_with("[sudo] password for kuba:\n  ⠋ Fetching              0s\n    remote: Counting objects"));
}

#[test]
fn live_lines_are_cut_to_the_terminal_width() {
    let (mut screen, out) = screen(Style::PLAIN, 40);
    let now = Instant::now();

    screen.event(&UpdateEvent::PhaseStarted { phase: Phase::Inputs }, now);
    screen.event(&step("nix flake update --flake /home/kuba/.config/kubaos/home"), now);
    screen.event(&output(&"x".repeat(200)), now);

    let shown = terminal(&out.bytes());
    assert!(shown.lines().all(|line| line.chars().count() < 40), "{shown}");
    assert!(shown.lines().all(|line| line.ends_with('…')), "{shown}");
}

#[test]
fn colours_come_from_the_terminal_palette_and_no_color_has_none() {
    let event = finished(Phase::Home, 1000, Some("reloaded Hyprland"));
    let (mut coloured, colour_out) = screen(Style::COLOR, 80);
    let (mut plain, plain_out) = screen(Style::PLAIN, 80);

    coloured.event(&event, Instant::now());
    plain.event(&event, Instant::now());

    assert!(colour_out.text().contains("\x1b[32m✓\x1b[0m"));
    assert!(colour_out.text().contains("\x1b[2mreloaded Hyprland\x1b[0m"));
    assert!(!plain_out.text().contains('\x1b'));
}

#[test]
fn plain_output_is_one_line_per_fact_without_escapes_or_programs_output() {
    let out = Shared::default();
    let err = Shared::default();
    let mut plain = Plain::new(out.clone(), err.clone(), text(Style::PLAIN));

    for event in &edge_update() {
        plain.event(event);
    }

    let text = out.text();
    println!("{text}");
    assert!(!text.contains('\x1b'));
    assert!(!text.contains("Unpacking firefox"), "a program's output is in the plain view:\n{text}");
    assert!(text.starts_with("KubaOS update\n\n  > Checking\n  ✓ Checking            0.0s  KubaOS Shinjuku"));
    assert!(text.contains("  > System packages\n      apt-get upgrade\n  ✓ System packages      42s  12 packages\n"));
    assert!(text.contains("    ! no allowed_signers on this machine: main's head is taken unchecked\n"));
    assert!(text.contains("  Update now? [Y/n] yes\n"));
    assert!(text.ends_with("  Log         ~/.local/state/kubaos/update/2026-10-23T09-14-03-update.log\n"));
    assert!(err.text().is_empty());
}

#[test]
fn a_failure_shows_the_end_of_the_failing_output_and_the_log_on_stderr() {
    let out = Shared::default();
    let err = Shared::default();
    let mut plain = Plain::new(out.clone(), err.clone(), text(Style::PLAIN));

    plain.event(&UpdateEvent::Failed {
        phase: Some(Phase::SystemPackages),
        error: "upgrading apt packages: `sudo apt-get upgrade` exited with code 100".into(),
        output: vec!["E: Sub-process /usr/bin/dpkg returned an error code (1)".into()],
        log: Some(LOG.into()),
    });

    assert_eq!(
        err.text(),
        "\n  ✗ upgrading apt packages: `sudo apt-get upgrade` exited with code 100\n    │ E: Sub-process \
         /usr/bin/dpkg returned an error code (1)\n  Log         \
         ~/.local/state/kubaos/update/2026-10-23T09-14-03-update.log\n"
    );
    assert!(out.text().is_empty());
}

#[test]
fn a_skipped_phase_says_why_and_a_needed_reboot_is_the_next_step() {
    let (mut screen, out) = screen(Style::PLAIN, 100);
    let mut events = edge_update();
    let Some(UpdateEvent::Summary(summary)) = events.last_mut() else {
        panic!("the update ends on its summary");
    };
    summary.restart = Restart::Reboot;
    summary.restart_reason = Some("an updated package asks for it".into());
    events.insert(
        events.len() - 1,
        UpdateEvent::PhaseFinished {
            phase: Phase::Hyprland,
            outcome: Outcome::Skipped,
            elapsed_ms: 20,
            detail: Some("Hyprland comes from KubaOS's nixpkgs on this architecture".into()),
        },
    );

    play(&mut screen, &out, &events);

    let shown = terminal(&out.bytes());
    assert!(shown.contains(
        "  – Hyprland pin        0.0s  skipped: Hyprland comes from KubaOS's nixpkgs on this architecture"
    ));
    assert!(shown.contains("  Next        reboot: an updated package asks for it"));
}

#[test]
fn turning_a_group_off_ends_on_what_it_changed_in_apt() {
    let (mut quiet, quiet_out) = screen(Style::PLAIN, 100);
    let (mut screen, out) = screen(Style::PLAIN, 100);
    let Some(UpdateEvent::Summary(mut summary)) = edge_update().pop() else {
        panic!("the update ends on its summary");
    };
    summary.command = Command::Packages;
    summary.apt = vec![GroupChange {
        group: "u2f".into(),
        installed: Vec::new(),
        removed: vec!["libpam-u2f".into(), "pamu2fcfg".into()],
    }];
    let started = UpdateEvent::Started {
        command: Command::Packages,
        log: None,
    };
    let mut unchanged = summary.clone();
    unchanged.apt.clear();

    play(&mut screen, &out, &[started, UpdateEvent::Summary(summary)]);
    play(&mut quiet, &quiet_out, &[UpdateEvent::Summary(unchanged)]);

    assert_eq!(
        terminal(&out.bytes()),
        "KubaOS packages\n\n\n  Apt         u2f: removed libpam-u2f, pamu2fcfg\n  \
         Log         ~/.local/state/kubaos/update/2026-10-23T09-14-03-update.log"
    );
    assert!(terminal(&quiet_out.bytes()).contains("  Apt         nothing changed\n"));
}

#[test]
fn json_is_one_event_per_line() {
    let out = Shared::default();
    let mut json = Json::new(out.clone());

    for event in &edge_update() {
        json.event(event);
    }

    let text = out.text();
    let read: Vec<UpdateEvent> = text
        .lines()
        .map(|line| UpdateEvent::from_json_line(line).unwrap())
        .collect();
    assert_eq!(read, edge_update());
}
