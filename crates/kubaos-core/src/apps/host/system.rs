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

use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;

use super::Host;
use super::HostError;
use super::UnitState;
use crate::apps::action::RUN_PATH;
use crate::apps::action::Scope;
use crate::packages::Packages;

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
        }
    }

    /// The same, editing the User flake's `packages.toml` through `packages`.
    #[must_use]
    pub fn with_packages(mut self, packages: Packages) -> Self {
        self.packages = Some(packages);
        self
    }

    fn command(program: impl AsRef<OsStr>) -> Command {
        let mut command = Command::new(program);
        command
            .env("PATH", RUN_PATH)
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

    fn systemctl(scope: Scope) -> Command {
        let mut command = Self::command("systemctl");
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
        let output = Self::command("dpkg-query")
            .args(["-W", "-f=${db:Status-Abbrev}\t${Package}\n"])
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
            .map(|(_, name)| name.split(':').next().unwrap_or(name).to_owned())
            .collect())
    }

    fn version(&mut self, package: &str) -> Result<Option<String>, HostError> {
        if self.installed(&[package.to_owned()])?.is_empty() {
            return Ok(None);
        }
        Self::out(Self::command("dpkg-query").args(["-W", "-f=${Version}", package])).map(Some)
    }

    fn apt_install(&mut self, repo: Option<&str>, packages: &[String]) -> Result<(), HostError> {
        if let Some(repo) = repo {
            Self::out(Self::command(self.checkout.join("system/apt-repo.sh")).arg(repo))?;
        }
        if packages.is_empty() {
            return Ok(());
        }
        Self::out(
            Self::command("apt-get")
                .args(["install", "-y", "--no-install-recommends"])
                .args([
                    "-o",
                    "Dpkg::Options::=--force-confdef",
                    "-o",
                    "Dpkg::Options::=--force-confold",
                ])
                .args(packages),
        )
        .map(drop)
    }

    fn deb_install(&mut self, deb: &Path) -> Result<(), HostError> {
        Self::out(
            Self::command("apt-get")
                .args(["install", "-y", "--no-install-recommends"])
                .args([
                    "-o",
                    "Dpkg::Options::=--force-confdef",
                    "-o",
                    "Dpkg::Options::=--force-confold",
                ])
                .arg(deb),
        )
        .map(drop)
    }

    fn apt_remove(&mut self, packages: &[String]) -> Result<(), HostError> {
        Self::out(
            Self::command("apt-get")
                .args(["remove", "-y"])
                .args([
                    "-o",
                    "Dpkg::Options::=--force-confdef",
                    "-o",
                    "Dpkg::Options::=--force-confold",
                ])
                .args(packages),
        )
        .map(drop)
    }

    fn fetch(&mut self, url: &str, sha256: &str, to: &Path) -> Result<(), HostError> {
        Self::out(
            Self::command("curl")
                .args(["-fsSL", "--proto", "=https", "--tlsv1.2", "--retry", "2", "-o"])
                .arg(to)
                .arg(url),
        )?;
        let sum = Self::out(Self::command("sha256sum").arg(to))?;
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
        match Self::out(Self::command("getent").args(["group", group])) {
            Ok(_) => Ok(true),
            Err(HostError::Failed { code: Some(2), .. }) => Ok(false),
            Err(e) => Err(e),
        }
    }

    fn member(&mut self, group: &str) -> Result<bool, HostError> {
        let groups = Self::out(Self::command("id").args(["-nG", &self.user]))?;
        Ok(groups.split_whitespace().any(|g| g == group))
    }

    fn group_create(&mut self, group: &str) -> Result<(), HostError> {
        Self::out(Self::command("groupadd").arg(group)).map(drop)
    }

    fn group_delete(&mut self, group: &str) -> Result<(), HostError> {
        Self::out(Self::command("groupdel").arg(group)).map(drop)
    }

    fn set_member(&mut self, group: &str, member: bool) -> Result<(), HostError> {
        let mut command = if member {
            let mut c = Self::command("usermod");
            c.args(["-aG", group, &self.user]);
            c
        } else {
            let mut c = Self::command("gpasswd");
            c.args(["-d", &self.user, group]);
            c
        };
        Self::out(&mut command).map(drop)
    }

    fn unit_state(&mut self, scope: Scope, unit: &str) -> Result<UnitState, HostError> {
        let shown =
            Self::out(Self::systemctl(scope).args(["show", "--property=LoadState,UnitFileState,ActiveState", unit]))?;
        let value = |key: &str| {
            shown
                .lines()
                .find_map(|line| line.strip_prefix(key)?.strip_prefix('='))
                .unwrap_or_default()
                .to_owned()
        };
        Ok(UnitState {
            exists: value("LoadState") == "loaded",
            enabled: matches!(value("UnitFileState").as_str(), "enabled" | "enabled-runtime"),
            active: matches!(value("ActiveState").as_str(), "active" | "activating" | "reloading"),
        })
    }

    fn daemon_reload(&mut self, scope: Scope) -> Result<(), HostError> {
        Self::out(Self::systemctl(scope).arg("daemon-reload")).map(drop)
    }

    fn set_enabled(&mut self, scope: Scope, unit: &str, enabled: bool) -> Result<(), HostError> {
        let verb = if enabled { "enable" } else { "disable" };
        Self::out(Self::systemctl(scope).args([verb, unit])).map(drop)
    }

    fn set_active(&mut self, scope: Scope, unit: &str, active: bool) -> Result<(), HostError> {
        let verb = if active { "start" } else { "stop" };
        Self::out(Self::systemctl(scope).args([verb, unit])).map(drop)
    }

    fn lingers(&mut self) -> Result<bool, HostError> {
        let shown =
            Self::out(Self::command("loginctl").args(["show-user", &self.user, "--property=Linger", "--value"]))?;
        Ok(shown.trim() == "yes")
    }

    fn set_linger(&mut self, on: bool) -> Result<(), HostError> {
        let verb = if on { "enable-linger" } else { "disable-linger" };
        Self::out(Self::command("loginctl").args([verb, &self.user])).map(drop)
    }

    fn run(&mut self, argv: &[String], dir: &Path) -> Result<(), HostError> {
        let Some((program, rest)) = argv.split_first() else {
            return Ok(());
        };
        let program = match program.strip_prefix("./") {
            Some(script) => dir.join(script),
            None => PathBuf::from(program),
        };
        let mut command = Self::command(&program);
        command
            .args(rest)
            .current_dir(dir)
            .env("HOME", &self.home)
            .env("USER", &self.user);
        Self::out(&mut command).map(drop)
    }
}
