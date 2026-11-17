//! The real machine: apt, dpkg, systemctl, loginctl, the group tools and curl, run as whoever
//! this process is, for the user it acts for.
//!
//! # Owns
//!
//! - [`SystemHost`].
//!
//! # Not here
//!
//! - The seam it implements: `host.rs`.

use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;

use super::AptPlan;
use super::AptSimulation;
use super::Compression;
use super::Downgrade;
use super::Host;
use super::HostError;
use super::UNIT_PROPERTIES;
use super::UnitState;
use super::untar_args;
use crate::apps::action::RUN_PATH;
use crate::apps::action::Scope;
use crate::apps::version::compare;
use crate::apt::Manifest;
use crate::apt::manifest::MANIFEST;
use crate::packages::Packages;
use crate::unfree::Setting;
use crate::unfree::Unfree;

/// Root's home, the `HOME` of what root runs.
const ROOT_HOME: &str = "/root";

/// The options every apt-get install and remove takes: unattended, keeping changed configuration.
const APT_OPTIONS: [&str; 4] = [
    "-o",
    "Dpkg::Options::=--force-confdef",
    "-o",
    "Dpkg::Options::=--force-confold",
];

/// The name and home of the user `uid`, as the password database (`/etc/passwd`) has them.
pub fn account(uid: u32) -> Option<(String, PathBuf)> {
    let passwd = fs::read_to_string("/etc/passwd").ok()?;
    passwd.lines().find_map(|line| {
        let fields: Vec<&str> = line.split(':').collect();
        (fields.len() >= 6 && fields[2].parse::<u32>().ok() == Some(uid))
            .then(|| (fields[0].to_owned(), PathBuf::from(fields[5])))
    })
}

/// The machine itself, for `user`.
#[derive(Debug)]
pub struct SystemHost {
    user: String,
    home: PathBuf,
    arch: String,
    /// The release's checkout, whose `system/apt-repo.sh` sets up a vendor repository.
    checkout: PathBuf,
    /// The User flake's `packages.toml`, on the user's side.
    packages: Option<Packages>,
    /// It is root's side: what it runs gets root's own environment, none of the user's.
    root: bool,
    /// The `PATH` of what it runs: [`RUN_PATH`], but for the root helper the tests run.
    path: OsString,
}

impl SystemHost {
    /// The machine, acting for `user` with the home `home`, the release's checkout `checkout`.
    #[must_use]
    pub fn new(user: impl Into<String>, home: impl Into<PathBuf>, checkout: impl Into<PathBuf>) -> Self {
        Self {
            user: user.into(),
            home: home.into(),
            arch: crate::apps::arch().to_owned(),
            checkout: checkout.into(),
            packages: None,
            root: false,
            path: OsString::from(RUN_PATH),
        }
    }

    /// The same, as root's side: every program it runs gets a clean environment, root's
    /// (`HOME=/root`), never the one of the user it acts for.
    #[must_use]
    pub fn as_root(mut self) -> Self {
        self.root = true;
        self
    }

    /// The same, finding programs on `path` instead of [`RUN_PATH`]: the root helper the tests run
    /// as themselves, whose `PATH` is their sandbox's.
    #[must_use]
    pub fn with_path(mut self, path: impl Into<OsString>) -> Self {
        self.path = path.into();
        self
    }

    /// The same, editing the User flake's `packages.toml` through `packages`.
    #[must_use]
    pub fn with_packages(mut self, packages: Packages) -> Self {
        self.packages = Some(packages);
        self
    }

    fn command(&self, program: impl AsRef<OsStr>) -> Command {
        let mut command = Command::new(program);
        if self.root {
            command
                .env_clear()
                .env("HOME", ROOT_HOME)
                .env("USER", "root")
                .env("LOGNAME", "root");
        }
        command
            .env("PATH", &self.path)
            .env("LANG", "C.UTF-8")
            .env("DEBIAN_FRONTEND", "noninteractive")
            .stdin(Stdio::null());
        command
    }

    /// Runs `command`; its stdout, when it exits 0.
    fn out(command: &mut Command) -> Result<String, HostError> {
        let program = command.get_program().to_string_lossy().into_owned();
        let shown = std::iter::once(program.clone())
            .chain(command.get_args().map(|a| a.to_string_lossy().into_owned()))
            .collect::<Vec<_>>()
            .join(" ");
        let output = command
            .output()
            .map_err(|source| HostError::Spawn { program, source })?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).into_owned())
        } else {
            Err(HostError::Failed {
                command: shown,
                code: output.status.code(),
                stderr: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
            })
        }
    }

    fn systemctl(&self, scope: Scope) -> Command {
        let mut command = self.command("systemctl");
        if scope == Scope::User {
            command.arg("--user");
        }
        command
    }

    fn nix(&self) -> Result<&Packages, HostError> {
        self.packages
            .as_ref()
            .ok_or_else(|| HostError::Nix("this side doesn't edit the User flake".into()))
    }
}

impl Host for SystemHost {
    fn user(&self) -> &str {
        &self.user
    }

    fn home(&self) -> &Path {
        &self.home
    }

    fn arch(&self) -> &str {
        &self.arch
    }

    fn place(&self, path: &Path) -> PathBuf {
        path.to_path_buf()
    }

    fn installed(&mut self, packages: &[String]) -> Result<BTreeSet<String>, HostError> {
        if packages.is_empty() {
            return Ok(BTreeSet::new());
        }
        // dpkg-query exits 1 when it knows none of them, which is an answer, not a failure.
        let output = self
            .command("dpkg-query")
            .args(["-W", "-f=${db:Status-Abbrev}\t${Package}\t${Architecture}\n"])
            .args(packages)
            .output()
            .map_err(|source| HostError::Spawn {
                program: "dpkg-query".to_owned(),
                source,
            })?;
        Ok(String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter_map(|line| line.split_once('\t'))
            .filter(|(status, _)| status.starts_with("ii"))
            .filter_map(|(_, rest)| rest.split_once('\t'))
            .map(|(name, arch)| package_name(name, arch, &self.arch))
            .collect())
    }

    fn manual(&mut self, packages: &[String]) -> Result<BTreeSet<String>, HostError> {
        if packages.is_empty() {
            return Ok(BTreeSet::new());
        }
        let shown = Self::out(self.command("apt-mark").arg("showmanual").args(packages))?;
        Ok(shown
            .lines()
            .map(|line| qualified(line.trim(), &self.arch))
            .filter(|name| packages.contains(name))
            .collect())
    }

    fn version(&mut self, package: &str) -> Result<Option<String>, HostError> {
        if self.installed(&[package.to_owned()])?.is_empty() {
            return Ok(None);
        }
        Self::out(self.command("dpkg-query").args(["-W", "-f=${Version}", package])).map(Some)
    }

    fn candidate(&mut self, package: &str) -> Result<Option<String>, HostError> {
        let shown = Self::out(self.command("apt-cache").args(["policy", "--", package]))?;
        Ok(candidate(&shown))
    }

    fn apt_repo(&mut self, repo: &str) -> Result<(), HostError> {
        Self::out(self.command(self.checkout.join("system/apt-repo.sh")).arg(repo)).map(drop)
    }

    fn apt_repo_ready(&mut self, repo: &str) -> Result<bool, HostError> {
        let path = self.checkout.join(MANIFEST);
        let manifest = Manifest::read(&path).map_err(HostError::Manifest)?.unwrap_or_default();
        Ok(manifest
            .repos()
            .get(repo)
            .is_some_and(|r| r.in_place(repo, Path::new("/"))))
    }

    fn apt_keys_due(&mut self, repos: &[String]) -> Result<BTreeSet<String>, HostError> {
        if repos.is_empty() {
            return Ok(BTreeSet::new());
        }
        let shown = Self::out(
            self.command(self.checkout.join("system/apt-repo.sh"))
                .arg("--keys-due")
                .args(repos),
        )?;
        Ok(shown.lines().map(str::to_owned).collect())
    }

    fn apt_simulate(&mut self, plan: AptPlan<'_>) -> Result<AptSimulation, HostError> {
        let mut command = self.command("apt-get");
        command.arg("-s");
        match plan {
            AptPlan::Install(packages) => command.args(["install", "--no-install-recommends"]).args(packages),
            AptPlan::Deb(deb) => command.args(["install", "--no-install-recommends"]).arg(deb),
            AptPlan::Remove(packages) => command.arg("remove").args(packages),
        };
        Ok(simulation(&Self::out(&mut command)?, &self.arch))
    }

    fn apt_install(&mut self, packages: &[String], downgrade: Downgrade) -> Result<(), HostError> {
        if packages.is_empty() {
            return Ok(());
        }
        let mut command = self.command("apt-get");
        command.args(["install", "-y", "--no-install-recommends"]);
        if downgrade == Downgrade::Allow {
            command.arg("--allow-downgrades");
        }
        Self::out(command.args(APT_OPTIONS).args(packages)).map(drop)
    }

    fn untar(&mut self, archive: &Path, into: &Path, compression: Compression, strip: u32) -> Result<(), HostError> {
        Self::out(self.command("tar").args(untar_args(archive, into, compression, strip))).map(drop)
    }

    fn deb_install(&mut self, deb: &Path) -> Result<(), HostError> {
        Self::out(
            self.command("apt-get")
                .args(["install", "-y", "--no-install-recommends"])
                .args(APT_OPTIONS)
                .arg(deb),
        )
        .map(drop)
    }

    fn apt_remove(&mut self, packages: &[String]) -> Result<(), HostError> {
        Self::out(
            self.command("apt-get")
                .args(["remove", "-y"])
                .args(APT_OPTIONS)
                .args(packages),
        )
        .map(drop)
    }

    fn fetch(&mut self, url: &str, sha256: &str, to: &Path) -> Result<(), HostError> {
        Self::out(
            self.command("curl")
                .args(["-fsSL", "--proto", "=https", "--tlsv1.2", "--retry", "2", "-o"])
                .arg(to)
                .arg(url),
        )?;
        let sum = Self::out(self.command("sha256sum").arg(to))?;
        if sum.split_whitespace().next() == Some(sha256) {
            Ok(())
        } else {
            let _ = fs::remove_file(to);
            Err(HostError::Sha256 {
                url: url.to_owned(),
                sha256: sha256.to_owned(),
            })
        }
    }

    fn nix_list(&mut self) -> Result<BTreeSet<String>, HostError> {
        let names = self.nix()?.list().map_err(|e| HostError::Nix(Box::new(e)))?;
        Ok(names.into_iter().collect())
    }

    fn unfree(&mut self) -> Result<Unfree, HostError> {
        Setting::read(self.nix()?.state().root()).map_err(|e| HostError::Nix(Box::new(e)))
    }

    fn nix_add(&mut self, names: &[String]) -> Result<(), HostError> {
        self.nix()?
            .add(names)
            .map(drop)
            .map_err(|e| HostError::Nix(Box::new(e)))
    }

    fn nix_remove(&mut self, names: &[String]) -> Result<(), HostError> {
        self.nix()?
            .remove(names)
            .map(drop)
            .map_err(|e| HostError::Nix(Box::new(e)))
    }

    fn group_exists(&mut self, group: &str) -> Result<bool, HostError> {
        match Self::out(self.command("getent").args(["group", group])) {
            Ok(_) => Ok(true),
            Err(HostError::Failed { code: Some(2), .. }) => Ok(false),
            Err(e) => Err(e),
        }
    }

    fn member(&mut self, group: &str) -> Result<bool, HostError> {
        let groups = Self::out(self.command("id").args(["-nG", &self.user]))?;
        Ok(groups.split_whitespace().any(|g| g == group))
    }

    fn group_create(&mut self, group: &str) -> Result<(), HostError> {
        Self::out(self.command("groupadd").arg(group)).map(drop)
    }

    fn group_delete(&mut self, group: &str) -> Result<(), HostError> {
        Self::out(self.command("groupdel").arg(group)).map(drop)
    }

    fn set_member(&mut self, group: &str, member: bool) -> Result<(), HostError> {
        let mut command = if member {
            let mut c = self.command("usermod");
            c.args(["-aG", group, &self.user]);
            c
        } else {
            let mut c = self.command("gpasswd");
            c.args(["-d", &self.user, group]);
            c
        };
        Self::out(&mut command).map(drop)
    }

    fn unit_state(&mut self, scope: Scope, unit: &str) -> Result<UnitState, HostError> {
        let property = format!("--property={UNIT_PROPERTIES}");
        let shown = Self::out(self.systemctl(scope).args(["show", &property, unit]))?;
        Ok(UnitState::from_show(&shown))
    }

    fn daemon_reload(&mut self, scope: Scope) -> Result<(), HostError> {
        Self::out(self.systemctl(scope).arg("daemon-reload")).map(drop)
    }

    fn set_enabled(&mut self, scope: Scope, unit: &str, enabled: bool) -> Result<(), HostError> {
        let verb = if enabled { "enable" } else { "disable" };
        Self::out(self.systemctl(scope).args([verb, unit])).map(drop)
    }

    fn set_active(&mut self, scope: Scope, unit: &str, active: bool) -> Result<(), HostError> {
        let verb = if active { "start" } else { "stop" };
        Self::out(self.systemctl(scope).args([verb, unit])).map(drop)
    }

    fn restart(&mut self, scope: Scope, unit: &str) -> Result<(), HostError> {
        Self::out(self.systemctl(scope).args(["try-restart", unit])).map(drop)
    }

    fn lingers(&mut self) -> Result<bool, HostError> {
        let shown =
            Self::out(
                self.command("loginctl")
                    .args(["show-user", &self.user, "--property=Linger", "--value"]),
            )?;
        Ok(shown.trim() == "yes")
    }

    fn set_linger(&mut self, on: bool) -> Result<(), HostError> {
        let verb = if on { "enable-linger" } else { "disable-linger" };
        Self::out(self.command("loginctl").args([verb, &self.user])).map(drop)
    }

    fn run(&mut self, argv: &[String], dir: &Path) -> Result<(), HostError> {
        let Some((program, rest)) = argv.split_first() else {
            return Ok(());
        };
        let program = match program.strip_prefix("./") {
            Some(script) => dir.join(script),
            None => PathBuf::from(program),
        };
        let mut command = self.command(&program);
        command.args(rest).current_dir(dir);
        if !self.root {
            command.env("HOME", &self.home).env("USER", &self.user);
        }
        Self::out(&mut command).map(drop)
    }
}

/// What `apt-get -s` printed: `Remv <package> [<version>]` for each package it would remove, and
/// `Inst <package> (<version> …)` for each it would newly install; a package that is there already
/// is `Inst <package> [<old version>] (<new version> …)`, an upgrade or, when the new version sorts
/// below the old one, a downgrade. A package of another architecture than `arch`, the machine's,
/// keeps its `:<arch>`, as it is another package.
fn simulation(shown: &str, arch: &str) -> AptSimulation {
    let mut simulation = AptSimulation::default();
    for line in shown.lines() {
        let mut words = line.split_whitespace();
        let (Some(verb), Some(package)) = (words.next(), words.next()) else {
            continue;
        };
        let package = qualified(package, arch);
        match (verb, words.next()) {
            ("Remv", _) => {
                simulation.removes.insert(package);
            }
            ("Inst", Some(old)) if old.starts_with('[') => {
                let old = old.trim_start_matches('[').trim_end_matches(']');
                let new = words.next().map(|w| w.trim_start_matches('('));
                if new.is_some_and(|new| compare(new, old) == Ordering::Less) {
                    simulation.downgrades.insert(package);
                }
            }
            ("Inst", _) => {
                simulation.installs.insert(package);
            }
            _ => {}
        }
    }
    simulation
}

/// `word`, a package apt named, as the engine names it: `<package>` for one of `arch`, the
/// machine's, or of every architecture, `<package>:<arch>` for one of another.
fn qualified(word: &str, arch: &str) -> String {
    match word.split_once(':') {
        Some((name, of)) => package_name(name, of, arch),
        None => word.to_owned(),
    }
}

/// Package `name` of architecture `of` as the engine names it, on a machine of `arch`.
fn package_name(name: &str, of: &str, arch: &str) -> String {
    if of == arch || of == "all" {
        name.to_owned()
    } else {
        format!("{name}:{of}")
    }
}

/// The candidate `apt-cache policy` shows, none when it shows `(none)` or no package.
fn candidate(policy: &str) -> Option<String> {
    policy
        .lines()
        .find_map(|line| line.trim().strip_prefix("Candidate:"))
        .map(str::trim)
        .filter(|version| !version.is_empty() && *version != "(none)")
        .map(str::to_owned)
}

#[cfg(test)]
mod tests;
