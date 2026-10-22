use std::collections::BTreeMap;
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

/// Runs git, bash and sh for real, as this user even for root commands; answers the rest from
/// `canned`, and records every command. A command whose line starts with one of `failing` fails.
struct Fake {
    ran: Vec<String>,
    canned: BTreeMap<&'static str, Result<String, ()>>,
    failing: Vec<&'static str>,
    switches: usize,
    answer: bool,
    asked: usize,
    free: u64,
    said: Vec<String>,
    warned: Vec<String>,
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
            answer: true,
            asked: 0,
            free: u64::MAX,
            said: Vec::new(),
            warned: Vec::new(),
        }
    }

    fn ran(&self, prefix: &str) -> usize {
        self.ran.iter().filter(|line| line.starts_with(prefix)).count()
    }

    fn run(cmd: &Cmd) -> Result<String, MachineError> {
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
        if out.status.success() {
            Ok(String::from_utf8_lossy(&out.stdout).into_owned())
        } else {
            Err(MachineError::exited(
                cmd,
                out.status.code(),
                &String::from_utf8_lossy(&out.stderr),
            ))
        }
    }
}

impl Machine for Fake {
    fn output(&mut self, cmd: &Cmd) -> Result<String, MachineError> {
        self.ran.push(cmd.display());
        if self.failing.iter().any(|prefix| cmd.display().starts_with(prefix)) {
            return Err(MachineError::exited(cmd, Some(100), "failing (test)"));
        }
        match cmd.program().as_str() {
            "git" | "bash" | "sh" => Self::run(cmd),
            program => match self.canned.get(program) {
                Some(Ok(out)) => Ok(out.clone()),
                Some(Err(())) => Err(MachineError::exited(cmd, Some(1), "canned failure")),
                None if program == "apt-get" && cmd.argv().iter().any(|a| a == "-s") => {
                    Ok("Inst vim [1] (2 Ubuntu)\nConf vim\n".to_owned())
                }
                None => Ok(String::new()),
            },
        }
    }

    fn stream(&mut self, cmd: &Cmd) -> Result<(), MachineError> {
        self.output(cmd).map(drop)
    }

    fn switch(&mut self) -> Result<(), MachineError> {
        self.ran.push("switch".to_owned());
        self.switches += 1;
        Ok(())
    }

    fn confirm(&mut self, _question: &str) -> bool {
        self.asked += 1;
        self.answer
    }

    fn free_bytes(&self, _path: &Path) -> io::Result<u64> {
        Ok(self.free)
    }

    fn say(&mut self, line: &str) {
        self.said.push(line.to_owned());
    }

    fn warn(&mut self, line: &str) {
        self.warned.push(line.to_owned());
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

    w.updater(&mut m).update(yes()).unwrap();

    assert_eq!(w.release(), head);
    assert_eq!(m.warned.iter().filter(|w| *w == NO_KEYS_EDGE).count(), 1);
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

    let err = w.updater(&mut m).update(yes()).unwrap_err();

    assert!(matches!(err, UpdateError::Migration { ref name, .. } if name == "1790000000-broken.sh"));
    assert!(!w.sys.markers().join("1790000000-broken.sh").exists());
    assert_eq!(w.ran_log(), "");
    assert_eq!(m.switches, 0);
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

    w.updater(&mut m).update(yes().packages_only()).unwrap();

    assert_eq!(w.release(), before);
    assert_eq!(w.ran_log(), "");
    assert_eq!(m.ran("sudo apt-get upgrade"), 1);
    assert_eq!(m.switches, 1);
}

#[test]
fn saying_no_changes_nothing() {
    let w = World::new(Channel::Edge, true);
    let before = w.release();
    w.push(&[("VERSION", "0.1.1\n")], true);
    let mut m = Fake::new();
    m.answer = false;

    let err = w.updater(&mut m).update(UpdateOptions::new()).unwrap_err();

    assert!(matches!(err, UpdateError::Declined));
    assert_eq!(m.asked, 1);
    assert_eq!(w.release(), before);
    assert_eq!(m.ran("sudo apt-get upgrade") + m.ran("sudo snapper"), 0);
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
    w.updater(&mut back).rollback(Confirm::AssumeYes).unwrap();

    assert_eq!(w.release(), before);
    assert_eq!(fs::read_to_string(w.user.flake_lock()).unwrap(), "lock before\n");
    assert_eq!(w.installed().commit, before);
    assert_eq!(w.installed().version.to_string(), "0.1.0");
    assert_eq!(fs::read_to_string(w.sys.history()).unwrap(), "");
    assert!(w.sys.markers().join("1790000000-fix.sh").is_file());
    assert_eq!(back.switches, 1);

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

    w.updater(&mut m).update(yes()).unwrap();

    assert_eq!(w.release(), signed);
    assert_eq!(w.installed().version.to_string(), "0.1.1");
    assert!(m.warned.iter().any(|w| w.contains("v0.1.2 is not signed")));
}

#[test]
fn stable_without_a_release_tag_says_so_and_updates_packages() {
    let w = World::new(Channel::Stable, true);
    let before = w.release();
    w.push(&[("VERSION", "0.1.1\n")], true);
    let mut m = Fake::new();

    w.updater(&mut m).update(yes()).unwrap();

    assert_eq!(w.release(), before);
    assert!(m.said.iter().any(|s| s.contains("newest release of its channel")));
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

    w.updater(&mut m).update(yes()).unwrap();

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
            "sudo apt-get install -y --no-install-recommends --allow-downgrades hyprland=0.50.1-1",
            "sudo apt-mark hold hyprland",
        ]
    );

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

    w.updater(&mut m).update(yes()).unwrap();

    assert!(!w.sys.apt_pin().exists());
    assert_eq!(m.ran("sudo apt-mark") + m.ran("sudo apt-get install"), 0);
}

#[test]
fn the_daily_check_writes_what_is_available_and_notifies_once() {
    let w = World::new(Channel::Edge, true);
    let head = w.push(&[("VERSION", "0.1.1\n")], true);
    let mut m = Fake::new();

    let available = w.updater(&mut m).check().unwrap();

    assert_eq!(available.update.as_ref().unwrap().commit, head);
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
