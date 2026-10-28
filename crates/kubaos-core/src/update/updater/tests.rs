use std::collections::BTreeMap;
use std::fmt::Write;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
use std::time::UNIX_EPOCH;

use super::Confirm;
use super::UpdateError;
use super::UpdateOptions;
use super::Updater;
use crate::state::Paths;
use crate::update::event;
use crate::update::event::Outcome;
use crate::update::event::Phase;
use crate::update::event::Prompt;
use crate::update::event::Restart;
use crate::update::event::Summary;
use crate::update::event::UpdateEvent;
use crate::update::fixture;
use crate::update::fixture::Key;
use crate::update::machine::Cmd;
use crate::update::machine::Machine;
use crate::update::machine::MachineError;
use crate::update::policy::NO_KEYS_EDGE;
use crate::update::policy::PolicyError;
use crate::update::record::Channel;
use crate::update::record::Installed;
use crate::update::record::SystemPaths;
use crate::update::record::parse_history;

/// What a program the fake ran printed, and whether it succeeded.
struct Ran {
    ok: bool,
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

/// Runs git, bash and sh for real, as this user even for root commands; answers the rest from
/// `canned`, and records every command. A command whose line starts with one of `failing` fails.
/// sudo's password is stale until the first `authenticate` when `password_stale` is set.
struct Fake {
    ran: Vec<String>,
    canned: BTreeMap<&'static str, Result<String, ()>>,
    failing: Vec<&'static str>,
    switches: usize,
    switch_output: Vec<&'static str>,
    answer: bool,
    asked: usize,
    password_stale: bool,
    authenticated: usize,
    free: u64,
    /// When set, apt, dpkg and apt-mark answer from it and change it, as on a machine.
    dpkg: Option<FakeDpkg>,
    /// The live preview the home is on.
    preview: Option<String>,
    /// Whether switching back from the preview fails.
    leave_fails: bool,
}

/// A machine's packages, for the package manifest's tests.
#[derive(Default)]
struct FakeDpkg {
    /// Installed packages and whether each is held.
    installed: BTreeMap<String, bool>,
    /// Held packages that aren't installed.
    held_missing: Vec<String>,
    /// Packages apt has no install candidate for.
    unavailable: Vec<String>,
    /// What removing a package takes along with it.
    takes: BTreeMap<String, Vec<String>>,
}

impl FakeDpkg {
    fn with(installed: &[&str]) -> Self {
        Self {
            installed: installed.iter().map(|p| ((*p).to_owned(), false)).collect(),
            ..Self::default()
        }
    }

    /// The packages an apt-get or apt-mark command line names.
    fn names(cmd: &Cmd) -> Vec<String> {
        cmd.argv()[1..]
            .iter()
            .map(|a| a.to_string_lossy().into_owned())
            .filter(|a| !a.starts_with('-') && !a.contains("::"))
            .filter(|a| {
                !matches!(
                    a.as_str(),
                    "install" | "remove" | "hold" | "unhold" | "showhold" | "policy"
                )
            })
            .map(|a| a.split('=').next().unwrap_or_default().to_owned())
            .collect()
    }

    /// The answer to `cmd` when it is one this machine knows, changing it as apt would.
    fn answer(&mut self, cmd: &Cmd) -> Option<String> {
        let args: Vec<String> = cmd.argv().iter().map(|a| a.to_string_lossy().into_owned()).collect();
        let has = |word: &str| args.iter().any(|a| a == word);
        let names = Self::names(cmd);
        match cmd.program().as_str() {
            "dpkg-query" if has("${db:Status-Status} ${db:Status-Want} ${Package} ${Version}\n") => {
                let mut out = String::new();
                for (p, held) in &self.installed {
                    let _ = writeln!(out, "installed {} {p} 1.0", if *held { "hold" } else { "install" });
                }
                for p in &self.held_missing {
                    let _ = writeln!(out, "not-installed hold {p} ");
                }
                Some(out)
            }
            "apt-cache" => Some(names.iter().filter(|p| !self.unavailable.contains(p)).fold(
                String::new(),
                |mut out, p| {
                    let _ = write!(out, "{p}:\n  Installed: (none)\n  Candidate: 1.0\n");
                    out
                },
            )),
            "apt-get" if has("-s") && has("remove") => Some(
                names
                    .iter()
                    .flat_map(|p| std::iter::once(p.clone()).chain(self.takes.get(p).cloned().unwrap_or_default()))
                    .fold(String::new(), |mut out, p| {
                        let _ = writeln!(out, "Remv {p} [1.0]");
                        out
                    }),
            ),
            "apt-get" if has("install") => {
                for p in names {
                    self.held_missing.retain(|h| *h != p);
                    self.installed.entry(p).or_insert(false);
                }
                Some(String::new())
            }
            "apt-get" if has("remove") => {
                for p in names {
                    self.installed.remove(&p);
                }
                Some(String::new())
            }
            "apt-mark" if has("hold") => {
                for p in names {
                    if let Some(held) = self.installed.get_mut(&p) {
                        *held = true;
                    }
                }
                Some(String::new())
            }
            "apt-mark" if has("unhold") => {
                for p in names {
                    self.held_missing.retain(|h| *h != p);
                    if let Some(held) = self.installed.get_mut(&p) {
                        *held = false;
                    }
                }
                Some(String::new())
            }
            _ => None,
        }
    }
}

impl Fake {
    fn new() -> Self {
        Self {
            ran: Vec::new(),
            canned: BTreeMap::from([
                ("dpkg", Ok("amd64\n".to_owned())),
                ("apt-mark", Ok(String::new())),
                ("dpkg-query", Ok("installed hyprland\ninstalled firefox\n".to_owned())),
            ]),
            failing: Vec::new(),
            switches: 0,
            switch_output: Vec::new(),
            answer: true,
            asked: 0,
            password_stale: false,
            authenticated: 0,
            free: u64::MAX,
            dpkg: None,
            preview: None,
            leave_fails: false,
        }
    }

    fn ran(&self, prefix: &str) -> usize {
        self.ran.iter().filter(|line| line.starts_with(prefix)).count()
    }

    fn exec(cmd: &Cmd) -> Result<Ran, MachineError> {
        let mut command = Command::new(&cmd.argv()[0]);
        command.args(&cmd.argv()[1..]);
        command.envs(cmd.vars().iter().map(|(k, v)| (k, v)));
        if let Some(dir) = cmd.dir() {
            command.current_dir(dir);
        }
        command.stdin(if cmd.input().is_some() {
            std::process::Stdio::piped()
        } else {
            std::process::Stdio::null()
        });
        command
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
        let mut child = command.spawn().map_err(|source| MachineError::Spawn {
            program: cmd.program(),
            source,
        })?;
        if let (Some(bytes), Some(mut stdin)) = (cmd.input(), child.stdin.take()) {
            io::Write::write_all(&mut stdin, bytes).unwrap();
        }
        let out = child.wait_with_output().unwrap();
        Ok(Ran {
            ok: out.status.success(),
            code: out.status.code(),
            stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        })
    }

    /// Runs or answers `cmd`.
    fn answer(&mut self, cmd: &Cmd) -> Result<Ran, MachineError> {
        self.ran.push(cmd.display());
        if self.failing.iter().any(|prefix| cmd.display().starts_with(prefix)) {
            return Ok(Ran {
                ok: false,
                code: Some(100),
                stdout: String::new(),
                stderr: "failing (test)".to_owned(),
            });
        }
        let canned = |stdout: String| Ran {
            ok: true,
            code: Some(0),
            stdout,
            stderr: String::new(),
        };
        if let Some(out) = self.dpkg.as_mut().and_then(|d| d.answer(cmd)) {
            return Ok(canned(out));
        }
        Ok(match cmd.program().as_str() {
            "git" | "bash" | "sh" => Self::exec(cmd)?,
            program => match self.canned.get(program) {
                Some(Ok(out)) => canned(out.clone()),
                Some(Err(())) => Ran {
                    ok: false,
                    code: Some(1),
                    stdout: String::new(),
                    stderr: "canned failure".to_owned(),
                },
                None if program == "apt-get" && cmd.argv().iter().any(|a| a == "-s") => {
                    canned("Inst vim [1] (2 Ubuntu)\nConf vim\n".to_owned())
                }
                None => canned(String::new()),
            },
        })
    }
}

impl Machine for Fake {
    fn output(&mut self, cmd: &Cmd) -> Result<String, MachineError> {
        let ran = self.answer(cmd)?;
        if ran.ok {
            Ok(ran.stdout)
        } else {
            Err(MachineError::exited(cmd, ran.code, &ran.stderr))
        }
    }

    fn run(&mut self, cmd: &Cmd, on_line: &mut dyn FnMut(&str)) -> Result<(), MachineError> {
        let ran = self.answer(cmd)?;
        for line in ran.stdout.lines().chain(ran.stderr.lines()) {
            on_line(line);
        }
        if ran.ok {
            Ok(())
        } else {
            Err(MachineError::exited(cmd, ran.code, ""))
        }
    }

    fn switch(&mut self, on_line: &mut dyn FnMut(&str)) -> Result<(), MachineError> {
        self.ran.push("switch".to_owned());
        self.switches += 1;
        for line in &self.switch_output {
            on_line(line);
        }
        Ok(())
    }

    fn preview(&mut self) -> Option<String> {
        self.preview.clone()
    }

    fn leave_preview(&mut self, on_line: &mut dyn FnMut(&str)) -> Result<(), MachineError> {
        self.ran.push("leave preview".to_owned());
        on_line("switching back");
        if self.leave_fails {
            return Err(MachineError::Switch("home-manager exited with code 1".to_owned()));
        }
        self.preview = None;
        Ok(())
    }

    fn confirm(&mut self, _question: &str) -> bool {
        self.asked += 1;
        self.answer
    }

    fn needs_password(&mut self) -> bool {
        self.password_stale
    }

    fn authenticate(&mut self) -> Result<(), MachineError> {
        self.authenticated += 1;
        self.password_stale = false;
        Ok(())
    }

    fn free_bytes(&self, _path: &Path) -> io::Result<u64> {
        Ok(self.free)
    }
}

/// The phases `events` started, in order.
fn phases(events: &[UpdateEvent]) -> Vec<Phase> {
    events
        .iter()
        .filter_map(|e| match e {
            UpdateEvent::PhaseStarted { phase } => Some(*phase),
            _ => None,
        })
        .collect()
}

fn warnings(events: &[UpdateEvent]) -> Vec<&str> {
    events
        .iter()
        .filter_map(|e| match e {
            UpdateEvent::Warning { text } => Some(text.as_str()),
            _ => None,
        })
        .collect()
}

fn notes(events: &[UpdateEvent]) -> Vec<&str> {
    events
        .iter()
        .filter_map(|e| match e {
            UpdateEvent::Note { text } => Some(text.as_str()),
            _ => None,
        })
        .collect()
}

fn summary(events: &[UpdateEvent]) -> &Summary {
    match events.last() {
        Some(UpdateEvent::Summary(summary)) => summary,
        last => panic!("the last event is {last:?}, not a summary"),
    }
}

/// A machine in a temporary folder: an origin, the checkout cloned from it, a user flake, and
/// the release record of the clone.
struct World {
    tmp: tempfile::TempDir,
    sys: SystemPaths,
    user: Paths,
    work: PathBuf,
    key: Key,
}

impl World {
    fn new(channel: Channel, signed: bool) -> Self {
        let tmp = tempfile::tempdir().unwrap();
        let key = fixture::key(&tmp.path().join("keys"));
        let origin = fixture::origin(tmp.path(), "0.1.0\n", &[("migrations/system/.keep", "")]);
        let sys = SystemPaths::under(tmp.path().join("root"));
        fixture::checkout(&origin, &sys.checkout());
        if signed {
            fs::create_dir_all(sys.allowed_signers().parent().unwrap()).unwrap();
            fs::copy(&key.allowed_signers, sys.allowed_signers()).unwrap();
        }
        let home = tmp.path().join("home");
        let user = Paths::from_vars(Some(home.clone().into()), None, None).unwrap();
        fs::create_dir_all(user.flake()).unwrap();
        fs::write(user.flake().join("flake.nix"), "{ }\n").unwrap();
        fs::write(user.flake_lock(), "lock before\n").unwrap();
        let world = Self {
            work: tmp.path().join("work"),
            tmp,
            sys,
            user,
            key,
        };
        let commit = world.release();
        let installed = Installed {
            version: crate::update::version::Version::parse("0.1.0").unwrap(),
            codename: Some("shinjuku".into()),
            commit,
            channel,
            date: String::new(),
        };
        fs::create_dir_all(world.sys.release().parent().unwrap()).unwrap();
        fs::write(world.sys.release(), installed.to_text()).unwrap();
        world
    }

    fn updater<'a>(&self, m: &'a mut Fake) -> Updater<'a, Fake> {
        let now = UNIX_EPOCH + Duration::from_secs(1_790_000_000);
        Updater::new(m, self.sys.clone(), self.user.clone(), now)
    }

    /// [`World::updater`], its events recorded in `events`.
    fn reporting<'a>(&self, m: &'a mut Fake, events: &'a mut Vec<UpdateEvent>) -> Updater<'a, Fake> {
        self.updater(m).reporting(events)
    }

    /// Commits `files` on origin's main, signed when `signed`, and returns the commit.
    fn push(&self, files: &[(&str, &str)], signed: bool) -> String {
        let commit = fixture::commit(&self.work, "change", files, signed.then_some(&self.key));
        fixture::git(&self.work, &["push", "-q", "origin", "main", "--tags"]);
        commit
    }

    fn release(&self) -> String {
        fixture::git(&self.sys.checkout(), &["rev-parse", "refs/heads/release"])
    }

    fn installed(&self) -> Installed {
        Installed::parse(&fs::read_to_string(self.sys.release()).unwrap()).unwrap()
    }

    fn log(&self) -> PathBuf {
        self.tmp.path().join("ran.log")
    }

    fn ran_log(&self) -> String {
        fs::read_to_string(self.log()).unwrap_or_default()
    }

    /// A migration that appends its name to the run log.
    fn migration(&self, name: &str) -> String {
        format!("echo {name} >> {}\n", self.log().display())
    }
}

fn yes() -> UpdateOptions {
    UpdateOptions::new().confirm(Confirm::AssumeYes)
}

#[test]
fn an_edge_update_moves_release_to_main_and_runs_each_migration_once() {
    let w = World::new(Channel::Edge, true);
    let before = w.release();
    let head = w.push(
        &[
            ("VERSION", "0.1.1\n"),
            ("migrations/system/1790000000-fix-apt.sh", &w.migration("system")),
            ("migrations/user/1790000001-move-config.sh", &w.migration("user")),
        ],
        true,
    );
    let mut m = Fake::new();

    w.updater(&mut m).update(yes()).unwrap();

    assert_eq!(w.release(), head);
    assert_eq!(w.ran_log(), "system\nuser\n");
    assert!(w.sys.markers().join("1790000000-fix-apt.sh").is_file());
    assert!(w.user.migration_markers().join("1790000001-move-config.sh").is_file());
    let installed = w.installed();
    assert_eq!(
        (installed.commit.as_str(), installed.version.to_string()),
        (head.as_str(), "0.1.1".into())
    );
    let history = parse_history(&fs::read_to_string(w.sys.history()).unwrap());
    assert_eq!(history.len(), 1);
    assert_eq!(
        (history[0].from_commit.as_str(), history[0].to_commit.as_str()),
        (before.as_str(), head.as_str())
    );
    let saved = history[0].lock.as_ref().unwrap();
    assert_eq!(fs::read_to_string(saved).unwrap(), "lock before\n");
    assert_eq!(m.switches, 1);
    assert_eq!(m.asked, 0);
    let order: Vec<String> = m
        .ran
        .iter()
        .filter(|line| {
            line.starts_with("sudo apt-get upgrade")
                || line.contains("checkout --quiet -B release")
                || line.starts_with("sudo bash")
                || line.starts_with("nix flake update")
                || *line == "switch"
                || line.starts_with("bash")
        })
        .map(|line| line.split(' ').take(2).collect::<Vec<_>>().join(" "))
        .collect();
    assert_eq!(
        order,
        [
            "sudo apt-get",
            "sudo git",
            "sudo bash",
            "nix flake",
            "switch",
            "bash -euo"
        ]
    );

    let mut again = Fake::new();
    w.updater(&mut again).update(yes()).unwrap();

    assert_eq!(w.ran_log(), "system\nuser\n");
    assert_eq!(w.release(), head);
    assert_eq!(again.ran("sudo bash") + again.ran("bash"), 0);
}

#[test]
fn an_update_reports_each_phase_that_runs_in_order_and_ends_with_its_summary() {
    let w = World::new(Channel::Edge, true);
    let before = w.release();
    let head = w.push(
        &[
            ("VERSION", "0.1.1\n"),
            ("migrations/system/1790000000-fix-apt.sh", &w.migration("system")),
            ("migrations/user/1790000001-move-config.sh", &w.migration("user")),
        ],
        true,
    );
    let mut m = Fake::new();
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events).update(yes()).unwrap();

    assert!(matches!(&events[0], UpdateEvent::Started { log: Some(_), .. }));
    assert_eq!(
        phases(&events),
        [
            Phase::Checking,
            Phase::Fetching,
            Phase::Verifying,
            Phase::Reviewing,
            Phase::Snapshot,
            Phase::SystemPackages,
            Phase::Release,
            Phase::SystemMigrations,
            Phase::Inputs,
            Phase::Home,
            Phase::UserMigrations,
        ]
    );
    assert!(events.iter().all(|e| !matches!(
        e,
        UpdateEvent::PhaseFinished {
            outcome: Outcome::Failed | Outcome::Skipped,
            ..
        }
    )));
    let preview = events.iter().find_map(|e| match e {
        UpdateEvent::Preview(preview) => Some(preview),
        _ => None,
    });
    let preview = preview.expect("a preview");
    assert_eq!(preview.to.as_ref().map(|to| to.commit.as_str()), Some(head.as_str()));
    assert_eq!(preview.migrations, ["1790000000-fix-apt.sh"]);
    assert_eq!(preview.packages, 1);
    let summary = summary(&events);
    assert_eq!(
        (summary.from.commit.as_str(), summary.to.commit.as_str()),
        (before.as_str(), head.as_str())
    );
    assert_eq!(summary.to.version, "0.1.1");
    assert_eq!(
        summary.migrations,
        ["1790000000-fix-apt.sh", "1790000001-move-config.sh"]
    );
    assert_eq!(summary.changes.len(), 1);
    assert_eq!(summary.restart, Restart::None);
    assert_eq!(summary.undo.as_deref(), Some("kubaos rollback"));
    let log = fs::read_to_string(summary.log.as_ref().unwrap()).unwrap();
    assert!(log.contains("== System migrations"), "the log:\n{log}");
    assert!(
        log.contains("$ sudo bash -euo pipefail ") && log.contains("1790000000-fix-apt.sh"),
        "the migration's command is not in the log:\n{log}"
    );
}

#[test]
fn a_stale_sudo_password_is_asked_once_before_the_first_root_command() {
    let w = World::new(Channel::Edge, true);
    w.push(&[("VERSION", "0.1.1\n")], true);
    let mut m = Fake::new();
    m.password_stale = true;
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events).update(yes()).unwrap();

    assert_eq!(m.authenticated, 1);
    let asked = events
        .iter()
        .position(|e| {
            matches!(
                e,
                UpdateEvent::Prompt {
                    prompt: Prompt::Password
                }
            )
        })
        .expect("a password prompt");
    assert!(matches!(
        events[asked - 2],
        UpdateEvent::PhaseStarted { phase: Phase::Fetching }
    ));
    assert_eq!(events[asked + 1], UpdateEvent::Answered { yes: true });
}

#[test]
fn the_summary_names_what_the_session_reloaded_and_the_inputs_that_moved() {
    let w = World::new(Channel::Edge, true);
    w.push(&[("VERSION", "0.1.1\n")], true);
    let mut m = Fake::new();
    m.canned.insert("nix", Ok("• Updated input 'nixpkgs':\n".to_owned()));
    m.switch_output = vec!["Activating reloadSession", "dot-session: reloaded Hyprland"];
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events).update(yes()).unwrap();

    let summary = summary(&events);
    assert_eq!(summary.reloaded, ["reloaded Hyprland"]);
    assert_eq!(summary.inputs, ["nixpkgs"]);
    assert!(events.contains(&UpdateEvent::Output {
        line: "Activating reloadSession".into()
    }));
    assert!(events.iter().any(|e| matches!(
        e,
        UpdateEvent::PhaseFinished { phase: Phase::Home, detail: Some(d), .. } if d == "reloaded Hyprland"
    )));
}

#[test]
fn an_edge_head_not_signed_by_a_kubaos_key_is_refused_before_anything_changes() {
    let w = World::new(Channel::Edge, true);
    let before = w.release();
    let head = w.push(&[("VERSION", "0.1.1\n")], false);
    let mut m = Fake::new();

    let err = w.updater(&mut m).update(yes()).unwrap_err();

    assert!(matches!(err, UpdateError::Policy(PolicyError::Unsigned { ref commit }) if *commit == head));
    assert_eq!(w.release(), before);
    assert_eq!(m.ran("sudo apt-get upgrade"), 0);
    assert!(!w.sys.history().exists());
}

#[test]
fn without_allowed_signers_edge_warns_once_and_updates() {
    let w = World::new(Channel::Edge, false);
    let head = w.push(&[("VERSION", "0.1.1\n")], false);
    let mut m = Fake::new();
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events).update(yes()).unwrap();

    assert_eq!(w.release(), head);
    assert_eq!(warnings(&events).iter().filter(|w| **w == NO_KEYS_EDGE).count(), 1);
}

#[test]
fn a_failed_system_migration_stops_the_update_and_runs_again_next_time() {
    let w = World::new(Channel::Edge, true);
    w.push(
        &[
            ("migrations/system/1790000000-broken.sh", "echo broken >&2; exit 3\n"),
            ("migrations/user/1790000001-later.sh", &w.migration("user")),
        ],
        true,
    );
    let mut m = Fake::new();
    let mut events = Vec::new();

    let err = w.reporting(&mut m, &mut events).update(yes()).unwrap_err();

    assert!(matches!(err, UpdateError::Migration { ref name, .. } if name == "1790000000-broken.sh"));
    assert!(!w.sys.markers().join("1790000000-broken.sh").exists());
    assert_eq!(w.ran_log(), "");
    assert_eq!(m.switches, 0);
    let Some(UpdateEvent::Failed {
        phase,
        error,
        output,
        log,
        ..
    }) = events.last()
    else {
        panic!("the last event is {:?}, not a failure", events.last());
    };
    assert_eq!(*phase, Some(Phase::SystemMigrations));
    assert_eq!(*error, event::chain(&err));
    assert_eq!(output, &["broken"]);
    assert!(log.as_ref().is_some_and(|log| log.is_file()));
}

#[test]
fn packages_only_leaves_release_and_the_migrations_alone() {
    let w = World::new(Channel::Edge, true);
    let before = w.release();
    w.push(
        &[("migrations/system/1790000000-later.sh", &w.migration("system"))],
        true,
    );
    let mut m = Fake::new();
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events).update(yes().packages_only()).unwrap();

    assert_eq!(w.release(), before);
    assert_eq!(w.ran_log(), "");
    assert_eq!(m.ran("sudo apt-get upgrade"), 1);
    assert_eq!(m.switches, 1);
    let ran = phases(&events);
    assert!(!ran.contains(&Phase::Release) && !ran.contains(&Phase::SystemMigrations));
    assert_eq!(summary(&events).to.commit, before);
}

#[test]
fn saying_no_changes_nothing() {
    let w = World::new(Channel::Edge, true);
    let before = w.release();
    w.push(&[("VERSION", "0.1.1\n")], true);
    let mut m = Fake::new();
    m.answer = false;
    let mut events = Vec::new();

    let err = w
        .reporting(&mut m, &mut events)
        .update(UpdateOptions::new())
        .unwrap_err();

    assert!(matches!(err, UpdateError::Declined));
    assert_eq!(m.asked, 1);
    assert_eq!(w.release(), before);
    assert_eq!(m.ran("sudo apt-get upgrade") + m.ran("sudo snapper"), 0);
    let tail: Vec<&UpdateEvent> = events.iter().rev().take(4).rev().collect();
    assert!(matches!(tail[0], UpdateEvent::Preview(_)));
    assert!(matches!(
        tail[1],
        UpdateEvent::Prompt {
            prompt: Prompt::Confirm { .. }
        }
    ));
    assert_eq!(
        tail[2..],
        [&UpdateEvent::Answered { yes: false }, &UpdateEvent::Declined]
    );
}

#[test]
fn a_full_disk_stops_the_update_before_the_network() {
    let w = World::new(Channel::Edge, true);
    let mut m = Fake::new();
    m.free = 1024;

    let err = w.updater(&mut m).update(yes()).unwrap_err();

    assert!(matches!(err, UpdateError::LowSpace { .. }));
    assert!(m.ran.is_empty());
}

#[test]
fn a_second_update_while_one_runs_is_busy() {
    let w = World::new(Channel::Edge, true);
    let mut m = Fake::new();
    let updater = w.updater(&mut m);
    let _held = updater.lock().unwrap();

    let mut other = Fake::new();
    let err = w.updater(&mut other).update(yes()).unwrap_err();

    assert!(matches!(err, UpdateError::Busy));
}

#[test]
fn a_check_while_an_update_runs_leaves_update_available_alone() {
    let w = World::new(Channel::Edge, true);
    let mut holder = Fake::new();
    let updater = w.updater(&mut holder);
    let _held = updater.lock().unwrap();
    let mut m = Fake::new();

    let available = w.updater(&mut m).check().unwrap();

    assert!(available.is_empty());
    assert!(!w.user.update_available().exists());
    assert_eq!(m.ran("notify-send"), 0);
}

#[test]
fn a_partly_failed_refresh_keeps_the_last_count() {
    let w = World::new(Channel::Edge, true);
    fs::create_dir_all(w.user.update_available().parent().unwrap()).unwrap();
    fs::write(
        w.user.update_available(),
        "apt-security=1\napt-security-packages=openssl=2\n",
    )
    .unwrap();
    let mut m = Fake::new();
    m.canned.insert(
        "apt-get",
        Ok("Inst vim [1] (2 Ubuntu:24.04/noble-security [amd64])\n".to_owned()),
    );
    m.failing.push("apt-get -o Dir::State=");

    let available = w.updater(&mut m).check().unwrap();

    assert_eq!(available.apt_security, ["openssl=2"]);
    assert_eq!(m.ran("apt-cache"), 0);
}

#[test]
fn a_candidate_from_updates_alone_is_checked_against_the_security_pocket() {
    let w = World::new(Channel::Edge, true);
    let mut m = Fake::new();
    m.canned.insert(
        "apt-cache",
        Ok("vim:\n  Version table:\n     2 500\n        500 http://x noble-updates/main amd64 Packages\n     \
            1.5 500\n        500 http://x noble-security/main amd64 Packages\n *** 1 100\n        100 /var/lib/dpkg/status\n"
            .to_owned()),
    );

    let available = w.updater(&mut m).check().unwrap();

    assert_eq!(available.apt_security, ["vim=2"]);
    assert_eq!(m.ran("apt-cache"), 1);
}

#[test]
fn notified_forgets_apt_security_packages_once_they_are_installed() {
    let w = World::new(Channel::Edge, true);
    fs::create_dir_all(w.user.notified().parent().unwrap()).unwrap();
    fs::write(w.user.notified(), "apt-security gone=1\nupdate abc\n").unwrap();

    w.updater(&mut Fake::new()).check().unwrap();

    let text = fs::read_to_string(w.user.notified()).unwrap();
    assert!(text.contains("update abc") && !text.contains("apt-security"));
}

#[test]
fn a_rollback_puts_release_and_the_flake_lock_back_and_keeps_the_markers() {
    let w = World::new(Channel::Edge, true);
    let before = w.release();
    w.push(
        &[
            ("VERSION", "0.1.1\n"),
            ("migrations/system/1790000000-fix.sh", &w.migration("system")),
        ],
        true,
    );
    let mut m = Fake::new();
    w.updater(&mut m).update(yes()).unwrap();
    fs::write(w.user.flake_lock(), "lock after\n").unwrap();

    let mut back = Fake::new();
    let mut events = Vec::new();
    w.reporting(&mut back, &mut events)
        .rollback(Confirm::AssumeYes)
        .unwrap();

    assert_eq!(w.release(), before);
    assert_eq!(fs::read_to_string(w.user.flake_lock()).unwrap(), "lock before\n");
    assert_eq!(w.installed().commit, before);
    assert_eq!(w.installed().version.to_string(), "0.1.0");
    assert_eq!(fs::read_to_string(w.sys.history()).unwrap(), "");
    assert!(w.sys.markers().join("1790000000-fix.sh").is_file());
    assert_eq!(back.switches, 1);
    assert_eq!(
        phases(&events),
        [Phase::Checking, Phase::Snapshot, Phase::Release, Phase::Home]
    );
    let summary = summary(&events);
    assert_eq!(
        (summary.to.commit.as_str(), summary.to.version.as_str()),
        (before.as_str(), "0.1.0")
    );
    assert_eq!(summary.undo, None);

    let err = w.updater(&mut Fake::new()).rollback(Confirm::AssumeYes).unwrap_err();
    assert!(matches!(err, UpdateError::NothingToRollBack { .. }));
}

#[test]
fn stable_moves_to_the_newest_signed_patch_and_skips_an_unsigned_one() {
    let w = World::new(Channel::Stable, true);
    let signed = w.push(&[("VERSION", "0.1.1\n")], true);
    fixture::tag(&w.work, "v0.1.1", Some(&w.key));
    w.push(&[("VERSION", "0.1.2\n")], true);
    fixture::tag(&w.work, "v0.1.2", None);
    w.push(&[("VERSION", "0.2.0-dev\n")], true);
    fixture::git(&w.work, &["push", "-q", "origin", "main", "--tags"]);
    let mut m = Fake::new();
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events).update(yes()).unwrap();

    assert_eq!(w.release(), signed);
    assert_eq!(w.installed().version.to_string(), "0.1.1");
    assert!(warnings(&events).iter().any(|w| w.contains("v0.1.2 is not signed")));
}

#[test]
fn stable_without_a_release_tag_says_so_and_updates_packages() {
    let w = World::new(Channel::Stable, true);
    let before = w.release();
    w.push(&[("VERSION", "0.1.1\n")], true);
    let mut m = Fake::new();
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events).update(yes()).unwrap();

    assert_eq!(w.release(), before);
    assert!(
        notes(&events)
            .iter()
            .any(|s| s.contains("newest release of its channel"))
    );
    assert_eq!(m.ran("sudo apt-get upgrade"), 1);
}

#[test]
fn an_upgrade_is_refused_on_edge_and_while_a_patch_is_pending() {
    let w = World::new(Channel::Edge, true);
    let err = w.updater(&mut Fake::new()).upgrade().unwrap_err();
    assert!(matches!(err, UpdateError::EdgeUpgrade));

    let w = World::new(Channel::Stable, true);
    w.push(&[("VERSION", "0.1.1\n")], true);
    fixture::tag(&w.work, "v0.1.1", Some(&w.key));
    w.push(&[("VERSION", "0.2.0\n"), ("CODENAME", "akihabara\n")], true);
    fixture::tag(&w.work, "v0.2.0", Some(&w.key));
    fixture::git(&w.work, &["push", "-q", "origin", "main", "--tags"]);

    let err = w.updater(&mut Fake::new()).upgrade().unwrap_err();
    assert!(matches!(err, UpdateError::PatchFirst { .. }));

    w.updater(&mut Fake::new()).update(yes()).unwrap();
    let mut m = Fake::new();
    w.updater(&mut m).upgrade().unwrap();
    assert_eq!(m.asked, 1);
    assert_eq!(w.installed().version.to_string(), "0.2.0");
    assert_eq!(w.installed().codename.as_deref(), Some("akihabara"));
}

#[test]
fn a_changed_hyprland_pin_is_installed_exactly_and_the_stack_held_again() {
    let w = World::new(Channel::Edge, true);
    let pin = "Package: hyprland\nPin: version 0.50.1-1\nPin-Priority: 1001\n";
    w.push(&[("system/files/apt/preferences.d/kubaos-hyprland", pin)], true);
    let mut m = Fake::new();
    m.canned.insert("apt-mark", Ok("hyprland\nfirefox\n".to_owned()));
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events).update(yes()).unwrap();

    assert_eq!(fs::read_to_string(w.sys.apt_pin()).unwrap(), pin);
    let hypr: Vec<&String> = m
        .ran
        .iter()
        .filter(|line| line.starts_with("sudo apt-mark") || line.starts_with("sudo apt-get install"))
        .collect();
    assert_eq!(
        hypr,
        [
            "sudo apt-mark unhold hyprland",
            "sudo apt-get install -y --no-install-recommends --allow-downgrades \
             -o Dpkg::Options::=--force-confdef -o Dpkg::Options::=--force-confold hyprland=0.50.1-1",
            "sudo apt-mark hold hyprland",
        ]
    );
    assert!(phases(&events).contains(&Phase::Hyprland));
    assert_eq!(summary(&events).restart, Restart::Relogin);

    let mut again = Fake::new();
    w.updater(&mut again).update(yes()).unwrap();
    assert_eq!(again.ran("sudo apt-mark"), 0);
}

#[test]
fn a_failed_hyprland_install_holds_the_stack_again_and_is_tried_next_time() {
    let w = World::new(Channel::Edge, true);
    let pin = "Package: hyprland\nPin: version 0.50.1-1\nPin-Priority: 1001\n";
    w.push(&[("system/files/apt/preferences.d/kubaos-hyprland", pin)], true);
    let mut m = Fake::new();
    m.canned.insert("apt-mark", Ok("hyprland\n".to_owned()));
    m.failing.push("sudo apt-get install");

    let err = w.updater(&mut m).update(yes()).unwrap_err();

    assert!(matches!(err, UpdateError::Step { ref what, .. } if what == "installing the pinned Hyprland"));
    assert_eq!(m.ran("sudo apt-mark hold hyprland"), 1);
    assert!(!w.sys.apt_pin().exists());
    assert_eq!(m.switches, 0);
}

#[test]
fn the_hyprland_pin_is_left_alone_off_amd64() {
    let w = World::new(Channel::Edge, true);
    w.push(
        &[("system/files/apt/preferences.d/kubaos-hyprland", "Package: hyprland\n")],
        true,
    );
    let mut m = Fake::new();
    m.canned.insert("dpkg", Ok("arm64\n".to_owned()));
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events).update(yes()).unwrap();

    assert!(!w.sys.apt_pin().exists());
    assert_eq!(m.ran("sudo apt-mark") + m.ran("sudo apt-get install"), 0);
    assert!(events.iter().any(|e| matches!(
        e,
        UpdateEvent::PhaseFinished {
            phase: Phase::Hyprland,
            outcome: Outcome::Skipped,
            ..
        }
    )));
    assert_eq!(summary(&events).restart, Restart::None);
}

#[test]
fn the_daily_check_writes_what_is_available_and_notifies_once() {
    let w = World::new(Channel::Edge, true);
    let head = w.push(&[("VERSION", "0.1.1\n")], true);
    let mut m = Fake::new();
    let mut events = Vec::new();

    let available = w.reporting(&mut m, &mut events).check().unwrap();

    assert_eq!(available.update.as_ref().unwrap().commit, head);
    assert_eq!(phases(&events), [Phase::Checking, Phase::Fetching, Phase::Verifying]);
    assert!(
        matches!(events.last(), Some(UpdateEvent::Available { update: Some(u), upgrade: None, .. }) if u.contains(&head[..7]))
    );
    assert!(!w.user.update_logs().exists(), "a check keeps no log");
    let text = fs::read_to_string(w.user.update_available()).unwrap();
    assert!(text.contains(&format!("update-commit={head}")));
    assert_eq!(m.ran("notify-send"), 1);

    let mut again = Fake::new();
    w.updater(&mut again).check().unwrap();
    assert_eq!(again.ran("notify-send"), 0);

    w.updater(&mut Fake::new()).update(yes()).unwrap();
    assert_eq!(fs::read_to_string(w.user.update_available()).unwrap(), "");
    let mut after = Fake::new();
    let available = w.updater(&mut after).check().unwrap();
    assert!(available.is_empty());
    assert_eq!(fs::read_to_string(w.user.update_available()).unwrap(), "");
    assert!(after.ran.iter().all(|line| !line.starts_with("sudo")));
}

#[test]
fn the_daily_check_counts_apt_security_updates_as_the_user_on_lists_of_its_own() {
    let w = World::new(Channel::Edge, true);
    let mut m = Fake::new();
    m.canned.insert(
        "apt-get",
        Ok("Inst openssl [1] (2 Ubuntu:24.04/noble-updates, Ubuntu:24.04/noble-security [amd64])\n".to_owned()),
    );

    let available = w.updater(&mut m).check().unwrap();

    assert_eq!(available.apt_security, ["openssl=2"]);
    let text = fs::read_to_string(w.user.update_available()).unwrap();
    assert!(text.contains("apt-security=1\napt-security-packages=openssl=2\n"));
    let lists = w.user.apt_lists();
    assert!(m.ran.iter().any(|line| line.starts_with("apt-get -o Dir::State=")
        && line.contains(&*lists.to_string_lossy())
        && line.ends_with(" update -q --error-on=any")));
    assert!(m.ran.iter().all(|line| !line.starts_with("sudo")));
    assert_eq!(m.ran("notify-send"), 1);

    w.updater(&mut Fake::new()).update(yes()).unwrap();
    assert_eq!(fs::read_to_string(w.user.update_available()).unwrap(), "");
}

#[test]
fn a_check_apt_cannot_answer_keeps_the_last_count_and_warns() {
    let w = World::new(Channel::Edge, true);
    fs::create_dir_all(w.user.update_available().parent().unwrap()).unwrap();
    fs::write(
        w.user.update_available(),
        "apt-security=1\napt-security-packages=openssl=2\n",
    )
    .unwrap();
    let mut m = Fake::new();
    m.failing.push("apt-get");
    let mut events = Vec::new();

    let available = w.reporting(&mut m, &mut events).check().unwrap();

    assert_eq!(available.apt_security, ["openssl=2"]);
    assert!(
        warnings(&events)
            .iter()
            .any(|text| text.contains("apt security updates"))
    );
}

#[test]
fn a_machine_without_a_record_runs_the_release_branch_on_edge() {
    let w = World::new(Channel::Stable, false);
    fs::remove_file(w.sys.release()).unwrap();
    let mut m = Fake::new();

    let installed = w.updater(&mut m).installed().unwrap();

    assert_eq!(installed.commit, w.release());
    assert_eq!(installed.version.to_string(), "0.1.0");
    assert_eq!(installed.codename.as_deref(), Some("shinjuku"));
    assert_eq!(installed.channel, Channel::Edge);
}

#[test]
fn the_channel_file_overrides_the_record() {
    let w = World::new(Channel::Stable, true);
    let mut m = Fake::new();

    w.updater(&mut m).set_channel(Channel::Edge).unwrap();

    assert_eq!(fs::read_to_string(w.sys.channel()).unwrap(), "edge\n");
    let updater = w.updater(&mut m);
    assert_eq!(updater.channel(&w.installed()), Channel::Edge);
}

/// A machine whose home is a live preview of `feature/x`.
fn previewing() -> Fake {
    Fake {
        preview: Some("feature/x (at 1a2b3c4, /h/Projects/kubaos)".to_owned()),
        ..Fake::new()
    }
}

/// Where each of `steps` first ran in `m`, in `steps`' order.
fn first_runs(m: &Fake, steps: &[&str]) -> Vec<usize> {
    steps
        .iter()
        .map(|step| m.ran.iter().position(|line| line.starts_with(step)).unwrap())
        .collect()
}

#[test]
fn an_update_leaves_a_live_preview_before_it_changes_anything() {
    let w = World::new(Channel::Edge, true);
    w.push(&[("VERSION", "0.1.1\n")], true);
    let mut m = previewing();
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events).update(yes()).unwrap();

    assert_eq!(m.preview, None);
    let at = first_runs(&m, &["leave preview", "sudo apt-get upgrade", "switch"]);
    assert!(at.is_sorted(), "leaving the preview comes first: {:?}", m.ran);
    let phases = phases(&events);
    assert_eq!(
        phases[..4],
        [Phase::Checking, Phase::Fetching, Phase::Verifying, Phase::Reviewing]
    );
    assert_eq!(phases[4..6], [Phase::LivePreview, Phase::Snapshot]);
}

#[test]
fn an_upgrade_leaves_a_live_preview_first() {
    let w = World::new(Channel::Stable, true);
    w.push(&[("VERSION", "0.2.0\n"), ("CODENAME", "akihabara\n")], true);
    fixture::tag(&w.work, "v0.2.0", Some(&w.key));
    fixture::git(&w.work, &["push", "-q", "origin", "main", "--tags"]);
    let mut m = previewing();

    w.updater(&mut m).upgrade().unwrap();

    assert_eq!(w.installed().version.to_string(), "0.2.0");
    let at = first_runs(&m, &["leave preview", "sudo apt-get upgrade", "switch"]);
    assert!(at.is_sorted(), "{:?}", m.ran);
    assert_eq!(m.preview, None);
}

#[test]
fn a_rollback_leaves_a_live_preview_first() {
    let w = World::new(Channel::Edge, true);
    w.push(&[("VERSION", "0.1.1\n")], true);
    w.updater(&mut Fake::new()).update(yes()).unwrap();
    let mut m = previewing();
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events).rollback(Confirm::AssumeYes).unwrap();

    let at = first_runs(&m, &["leave preview", "switch"]);
    assert!(at.is_sorted(), "{:?}", m.ran);
    assert_eq!(m.preview, None);
    assert_eq!(
        phases(&events),
        [
            Phase::Checking,
            Phase::LivePreview,
            Phase::Snapshot,
            Phase::Release,
            Phase::Home
        ]
    );
}

#[test]
fn a_declined_update_keeps_the_live_preview() {
    let w = World::new(Channel::Edge, true);
    w.push(&[("VERSION", "0.1.1\n")], true);
    let mut m = previewing();
    m.answer = false;

    let err = w.updater(&mut m).update(UpdateOptions::new()).unwrap_err();

    assert!(matches!(err, UpdateError::Declined));
    assert!(m.preview.is_some());
    assert!(!m.ran.iter().any(|line| line == "leave preview"));
}

#[test]
fn an_update_that_cannot_leave_the_live_preview_stops_before_anything_changes() {
    let w = World::new(Channel::Edge, true);
    let before = w.release();
    w.push(&[("VERSION", "0.1.1\n")], true);
    let mut m = previewing();
    m.leave_fails = true;

    let err = w.updater(&mut m).update(yes()).unwrap_err();

    assert!(err.to_string().contains("leaving the live preview"), "{err}");
    assert_eq!(w.release(), before);
    assert_eq!(m.ran("sudo apt-get upgrade") + m.ran("sudo snapper") + m.switches, 0);
}

#[test]
fn an_update_without_a_live_preview_has_no_such_phase() {
    let w = World::new(Channel::Edge, true);
    w.push(&[("VERSION", "0.1.1\n")], true);
    let mut m = Fake::new();
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events).update(yes()).unwrap();

    assert!(!phases(&events).contains(&Phase::LivePreview));
    assert!(!m.ran.iter().any(|line| line == "leave preview"));
}

mod manifest;
