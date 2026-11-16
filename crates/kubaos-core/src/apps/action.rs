//! The action vocabulary of an app's folder: the small, fixed set of things an install does, each
//! knowing how to do itself, check whether it is done, and undo itself.
//!
//! An [`Action`] is what `app.toml` declares, and, once [`Action::resolve`] has filled in its
//! words and read the files it names, what a journal records as done. Whether it runs as root or
//! as the user is the kind's, by its paths and scope, never the author's, except `run`'s.
//!
//! # Owns
//!
//! - [`Action`], [`Kind`] and what a valid action is; [`Words`]; an action's [`Side`] and the paths
//!   and keys it writes.
//!
//! # Not here
//!
//! - Doing, checking and undoing an action on a machine: `action/perform.rs`. The journal:
//!   `journal.rs`. The order of a run: `engine.rs`.

pub mod perform;

use std::fs;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

use super::manifest::ARCHES;
use super::manifest::Download;
use super::manifest::is_app_id;
use crate::apt::manifest::is_package_name;
use crate::packages::validate_name;

/// The folders under which an action may write as root. Everything else on the system is the
/// base's or a package's.
pub const SYSTEM_PREFIXES: [&str; 3] = ["/etc/", "/opt/", "/usr/local/"];

/// The `PATH` a `run` runs with, as the user and as root alike.
pub const RUN_PATH: &str = "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin";

/// An action that isn't one.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ActionError {
    #[error("the action is not valid TOML of its shape")]
    Parse(#[source] toml::de::Error),
    #[error("an action needs a name, the step the user sees")]
    NoName,
    #[error("{key} is empty")]
    Empty { key: &'static str },
    #[error("{name:?} is not an apt package name")]
    InvalidPackage { name: String },
    #[error("{name:?} is not a nixpkgs attribute name")]
    InvalidAttribute { name: String },
    #[error("{name:?} is not a repository name: lowercase letters, digits and dashes")]
    InvalidRepo { name: String },
    #[error("{value:?} is not an architecture dpkg names: amd64 or arm64")]
    InvalidArch { value: String },
    #[error("{value:?} is not a path an action writes: `~/…` in the home, or under /etc/, /opt/ or /usr/local/")]
    InvalidPath { value: String },
    #[error("{value:?} is not an https:// URL with a sha256 of 64 lowercase hex digits")]
    InvalidDownload { value: String },
    #[error("a download needs a download for amd64 or arm64")]
    NoDownload,
    #[error("a file is `from` a file in the app's folder or `text`, one of them")]
    FileSource,
    #[error("{value:?} is not a file in the app's folder")]
    InvalidFrom { value: String },
    #[error("reading {}", path.display())]
    ReadFrom {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("{value:?} is not a file mode: three octal digits")]
    InvalidMode { value: String },
    #[error("a setting's file is a .json or .toml file in the home; {value:?} isn't")]
    InvalidSettingFile { value: String },
    #[error("{value:?} is not a setting's key: dot-separated names")]
    InvalidKey { value: String },
    #[error("a setting's value is a string, a number or a boolean")]
    InvalidValue,
    #[error("{value:?} is not a systemd unit name")]
    InvalidUnit { value: String },
    #[error("a unit action enables or disables, starts or stops: at least one of `enable` and `start`")]
    UnitDoesNothing,
    #[error("{value:?} is not a group name")]
    InvalidGroup { value: String },
    #[error("a run's {key} is an argv: a program on PATH or ./<script> in the app's folder, then its arguments")]
    InvalidArgv { key: &'static str },
}

/// One step of an install: its name and what it does.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "toml::Table")]
pub struct Action {
    /// The step as the user reads it: "Start the service".
    pub name: String,
    #[serde(flatten)]
    pub kind: Kind,
}

/// What an action does.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
#[non_exhaustive]
pub enum Kind {
    /// apt packages, from Ubuntu's archive or a vendor repository of the package manifest.
    Apt {
        packages: Vec<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        repo: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        arch: Option<String>,
    },
    /// A vendor's `.deb`, pinned to its sha256 for each architecture.
    Deb {
        package: String,
        version: String,
        #[serde(default)]
        self_updates: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        amd64: Option<Download>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        arm64: Option<Download>,
    },
    /// Names of the user's nixpkgs, in the User flake's `packages.toml`.
    Nix {
        packages: Vec<String>,
    },
    /// A pinned download, unpacked to a path.
    Download {
        to: String,
        version: String,
        unpack: Unpack,
        #[serde(default)]
        strip: u32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        amd64: Option<Download>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        arm64: Option<Download>,
    },
    /// A file, from one in the app's folder or from `text`.
    File {
        to: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        from: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        text: Option<String>,
        #[serde(default)]
        template: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        mode: Option<String>,
    },
    Symlink {
        to: String,
        target: String,
    },
    /// A key in an app's JSON or TOML settings, the file's other keys kept.
    Setting {
        file: String,
        key: String,
        value: toml::Value,
    },
    /// A systemd unit enabled or disabled, started or stopped.
    Unit {
        unit: String,
        scope: Scope,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        enable: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        start: Option<bool>,
    },
    /// The user who asked, in a group.
    Group {
        group: String,
    },
    /// The user's services kept running without a session.
    Linger {},
    /// A path a program of the app writes later, which the app owns all the same.
    Output {
        path: String,
    },
    /// A vendor tool's own command, with how to check it and undo it.
    Run {
        #[serde(rename = "do")]
        run: Vec<String>,
        check: Vec<String>,
        undo: Vec<String>,
        #[serde(default)]
        root: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        window: Option<String>,
    },
}

/// How a download becomes its path.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Unpack {
    /// The download is the file.
    File,
    /// A gzipped tarball, unpacked into the path as a folder.
    TarGz,
}

/// A systemd manager.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    User,
    System,
}

impl Scope {
    pub fn word(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::System => "system",
        }
    }
}

/// Who runs an action.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Side {
    User,
    Root,
}

/// The words an action's strings take: `@USER@`, `@HOME@`, `@ARCH@`, and `@VERSION@` in an action
/// with a version.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Words {
    pub user: String,
    pub home: PathBuf,
    pub arch: String,
}

impl Words {
    fn fill(&self, text: &str, version: Option<&str>) -> String {
        let mut out = text
            .replace("@USER@", &self.user)
            .replace("@HOME@", &self.home.to_string_lossy())
            .replace("@ARCH@", &self.arch);
        if let Some(version) = version {
            out = out.replace("@VERSION@", version);
        }
        out
    }

    /// Where `path`, as an action names it, is: `~/…` under the home, else itself.
    pub fn path(&self, path: &str) -> PathBuf {
        match path.strip_prefix("~/") {
            Some(rest) => self.home.join(rest),
            None => PathBuf::from(path),
        }
    }
}

impl TryFrom<toml::Table> for Action {
    type Error = ActionError;

    fn try_from(mut table: toml::Table) -> Result<Self, Self::Error> {
        let Some(toml::Value::String(name)) = table.remove("name") else {
            return Err(ActionError::NoName);
        };
        let kind = Kind::deserialize(toml::Value::Table(table)).map_err(ActionError::Parse)?;
        let action = Self { name, kind };
        action.check()?;
        Ok(action)
    }
}

impl Action {
    /// Who runs it: by its kind, its paths and its scope, and for `run`, by `root`.
    pub fn side(&self) -> Side {
        let by_path = |path: &str| {
            if path.starts_with("~/") { Side::User } else { Side::Root }
        };
        match &self.kind {
            Kind::Apt { .. } | Kind::Deb { .. } | Kind::Group { .. } => Side::Root,
            Kind::Nix { .. } | Kind::Setting { .. } | Kind::Linger {} => Side::User,
            Kind::Download { to, .. } | Kind::File { to, .. } | Kind::Symlink { to, .. } => by_path(to),
            Kind::Output { path } => by_path(path),
            Kind::Unit { scope, .. } => match scope {
                Scope::User => Side::User,
                Scope::System => Side::Root,
            },
            Kind::Run { root, .. } => {
                if *root {
                    Side::Root
                } else {
                    Side::User
                }
            }
        }
    }

    /// The path it writes, as it names it, when it writes one.
    pub fn path(&self) -> Option<&str> {
        match &self.kind {
            Kind::Download { to, .. } | Kind::File { to, .. } | Kind::Symlink { to, .. } => Some(to),
            Kind::Output { path } => Some(path),
            _ => None,
        }
    }

    /// The pinned download for `arch`, of a `.deb` or a download.
    pub fn download(&self, arch: &str) -> Option<&Download> {
        let (Kind::Deb { amd64, arm64, .. } | Kind::Download { amd64, arm64, .. }) = &self.kind else {
            return None;
        };
        match arch {
            "amd64" => amd64.as_ref(),
            "arm64" => arm64.as_ref(),
            _ => None,
        }
    }

    /// Whether it has something for a machine of `arch`.
    pub fn available(&self, arch: &str) -> bool {
        match &self.kind {
            Kind::Apt { arch: Some(only), .. } => only == arch,
            Kind::Deb { .. } | Kind::Download { .. } => self.download(arch).is_some(),
            _ => true,
        }
    }

    /// The action as it runs on this machine and as a journal records it: its words filled in,
    /// and a file's `from` read from `folder` into its `text`.
    ///
    /// # Errors
    ///
    /// [`ActionError::ReadFrom`] when a file's `from` can't be read.
    pub fn resolve(&self, words: &Words, folder: &Path) -> Result<Self, ActionError> {
        let fill = |text: &str, version: Option<&str>| words.fill(text, version);
        let kind = match &self.kind {
            Kind::Download {
                to,
                version,
                unpack,
                strip,
                amd64,
                arm64,
            } => Kind::Download {
                to: fill(to, Some(version)),
                version: version.clone(),
                unpack: *unpack,
                strip: *strip,
                amd64: amd64.clone(),
                arm64: arm64.clone(),
            },
            Kind::File {
                to,
                from,
                text,
                template,
                mode,
            } => {
                let raw = match (from, text) {
                    (Some(from), _) => {
                        let path = folder.join(from);
                        fs::read_to_string(&path).map_err(|source| ActionError::ReadFrom { path, source })?
                    }
                    (None, Some(text)) => text.clone(),
                    (None, None) => return Err(ActionError::FileSource),
                };
                let text = if *template { fill(&raw, None) } else { raw };
                Kind::File {
                    to: fill(to, None),
                    from: None,
                    text: Some(text),
                    template: false,
                    mode: mode.clone(),
                }
            }
            Kind::Symlink { to, target } => Kind::Symlink {
                to: fill(to, None),
                target: fill(target, None),
            },
            Kind::Setting { file, key, value } => Kind::Setting {
                file: fill(file, None),
                key: key.clone(),
                value: value.clone(),
            },
            Kind::Output { path } => Kind::Output { path: fill(path, None) },
            Kind::Run {
                run,
                check,
                undo,
                root,
                window,
            } => {
                let argv = |argv: &[String]| argv.iter().map(|a| fill(a, None)).collect();
                Kind::Run {
                    run: argv(run),
                    check: argv(check),
                    undo: argv(undo),
                    root: *root,
                    window: window.clone(),
                }
            }
            other => other.clone(),
        };
        Ok(Self {
            name: self.name.clone(),
            kind,
        })
    }

    fn check(&self) -> Result<(), ActionError> {
        if self.name.trim().is_empty() {
            return Err(ActionError::NoName);
        }
        match &self.kind {
            Kind::Apt { packages, repo, arch } => {
                non_empty("packages", packages)?;
                if let Some(name) = packages.iter().find(|p| !is_package_name(p)) {
                    return Err(ActionError::InvalidPackage { name: name.clone() });
                }
                if let Some(repo) = repo
                    && !is_app_id(repo)
                {
                    return Err(ActionError::InvalidRepo { name: repo.clone() });
                }
                if let Some(arch) = arch {
                    check_arch(arch)?;
                }
            }
            Kind::Deb { package, .. } => {
                if !is_package_name(package) {
                    return Err(ActionError::InvalidPackage { name: package.clone() });
                }
                self.check_downloads()?;
            }
            Kind::Nix { packages } => {
                non_empty("packages", packages)?;
                for name in packages {
                    validate_name(name).map_err(|_| ActionError::InvalidAttribute { name: name.clone() })?;
                }
            }
            Kind::Download { to, .. } => {
                check_path(to)?;
                self.check_downloads()?;
            }
            Kind::File {
                to, from, text, mode, ..
            } => {
                check_path(to)?;
                match (from, text) {
                    (Some(from), None) if is_folder_file(from) => {}
                    (Some(from), None) => return Err(ActionError::InvalidFrom { value: from.clone() }),
                    (None, Some(_)) => {}
                    _ => return Err(ActionError::FileSource),
                }
                if let Some(mode) = mode
                    && parse_mode(mode).is_none()
                {
                    return Err(ActionError::InvalidMode { value: mode.clone() });
                }
            }
            Kind::Symlink { to, target } => {
                check_path(to)?;
                if target.is_empty() {
                    return Err(ActionError::Empty { key: "target" });
                }
            }
            Kind::Setting { file, key, value } => {
                let home = file.starts_with("~/") && check_path(file).is_ok();
                if !home || !Path::new(file).extension().is_some_and(|e| e == "json" || e == "toml") {
                    return Err(ActionError::InvalidSettingFile { value: file.clone() });
                }
                if key.split('.').any(str::is_empty) {
                    return Err(ActionError::InvalidKey { value: key.clone() });
                }
                if !matches!(
                    value,
                    toml::Value::String(_) | toml::Value::Integer(_) | toml::Value::Float(_) | toml::Value::Boolean(_)
                ) {
                    return Err(ActionError::InvalidValue);
                }
            }
            Kind::Unit {
                unit, enable, start, ..
            } => {
                if !is_unit_name(unit) {
                    return Err(ActionError::InvalidUnit { value: unit.clone() });
                }
                if enable.is_none() && start.is_none() {
                    return Err(ActionError::UnitDoesNothing);
                }
            }
            Kind::Group { group } => {
                if !is_group_name(group) {
                    return Err(ActionError::InvalidGroup { value: group.clone() });
                }
            }
            Kind::Linger {} => {}
            Kind::Output { path } => check_path(path)?,
            Kind::Run { run, check, undo, .. } => {
                for (key, argv) in [("do", run), ("check", check), ("undo", undo)] {
                    if !is_argv(argv) {
                        return Err(ActionError::InvalidArgv { key });
                    }
                }
            }
        }
        Ok(())
    }

    fn check_downloads(&self) -> Result<(), ActionError> {
        let found: Vec<&Download> = ARCHES.iter().filter_map(|arch| self.download(arch)).collect();
        if found.is_empty() {
            return Err(ActionError::NoDownload);
        }
        for download in found {
            let url = download
                .url
                .strip_prefix("https://")
                .is_some_and(|rest| !rest.is_empty() && !rest.chars().any(|c| c.is_whitespace() || c.is_control()));
            let sha = download.sha256.len() == 64
                && download
                    .sha256
                    .chars()
                    .all(|c| c.is_ascii_digit() || matches!(c, 'a'..='f'));
            if !url || !sha {
                return Err(ActionError::InvalidDownload {
                    value: download.url.clone(),
                });
            }
        }
        Ok(())
    }
}

/// The mode a file's `mode` names, three octal digits.
pub fn parse_mode(mode: &str) -> Option<u32> {
    (mode.len() == 3 && mode.chars().all(|c| matches!(c, '0'..='7')))
        .then(|| u32::from_str_radix(mode, 8).ok())
        .flatten()
}

fn non_empty(key: &'static str, values: &[String]) -> Result<(), ActionError> {
    if values.is_empty() {
        Err(ActionError::Empty { key })
    } else {
        Ok(())
    }
}

fn check_arch(arch: &str) -> Result<(), ActionError> {
    if ARCHES.contains(&arch) {
        Ok(())
    } else {
        Err(ActionError::InvalidArch { value: arch.to_owned() })
    }
}

/// Whether `path` can be one an action writes: `~/…` with no `.` or `..` part, or an absolute path
/// under [`SYSTEM_PREFIXES`] with none either. The words in it are filled in later, so `@VERSION@`
/// is a name like any other here.
fn check_path(path: &str) -> Result<(), ActionError> {
    let invalid = || ActionError::InvalidPath { value: path.to_owned() };
    let rest = if let Some(rest) = path.strip_prefix("~/") {
        rest
    } else if SYSTEM_PREFIXES.iter().any(|prefix| path.starts_with(prefix)) {
        &path[1..]
    } else {
        return Err(invalid());
    };
    let clean = !rest.is_empty()
        && Path::new(rest).components().all(|c| matches!(c, Component::Normal(_)))
        && !rest
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..");
    if clean { Ok(()) } else { Err(invalid()) }
}

/// Whether `value` names a file in the app's folder: a name, no folder above it.
fn is_folder_file(value: &str) -> bool {
    !value.is_empty() && Path::new(value).components().all(|c| matches!(c, Component::Normal(_)))
}

/// Whether `argv` is a program on `PATH` (a bare name) or `./<script>` in the app's folder, or a
/// program under the home (`@HOME@/…`), then its arguments.
pub(crate) fn is_argv(argv: &[String]) -> bool {
    let Some(program) = argv.first() else {
        return false;
    };
    let bare = !program.is_empty()
        && program
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'));
    let script = program.strip_prefix("./").is_some_and(is_folder_file);
    let home = program.strip_prefix("@HOME@/").is_some_and(is_folder_path);
    bare || script || home
}

fn is_folder_path(value: &str) -> bool {
    !value.is_empty()
        && !value
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
}

fn is_unit_name(value: &str) -> bool {
    let types = [".service", ".socket", ".path", ".timer", ".target", ".mount"];
    value.len() <= 255
        && types.iter().any(|t| value.ends_with(t))
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '@' | ':'))
}

fn is_group_name(value: &str) -> bool {
    value.len() <= 32
        && value.chars().next().is_some_and(|c| c.is_ascii_lowercase() || c == '_')
        && value
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '-' | '_'))
}

#[cfg(test)]
mod tests;
