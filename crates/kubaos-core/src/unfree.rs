//! Unfree packages in the user's nixpkgs: the setting that allows them, `nixpkgs.toml` in the
//! state directory, and telling unfree packages from free ones.
//!
//! Unfree packages are allowed until `nixpkgs.toml` says `allow_unfree = false`. KubaOS's home
//! module (`home/state.nix`) reads the same key into the home's `nixpkgs.config`, so the setting
//! holds for every package of the user's nixpkgs, `home.nix`'s included. Whether a package is
//! unfree is its `meta.unfree` in the user flake's nixpkgs, read by one `nix eval` for every name
//! at once.
//!
//! # Owns
//!
//! - Reading and writing `nixpkgs.toml` ([`Setting`]).
//! - The `nix eval` that tells unfree packages from free ones ([`LicenseCheck`]), and which
//!   packages the setting lets in ([`Unfree::admit`]).
//!
//! # Not here
//!
//! - Editing `packages.toml`: `packages.rs`.
//! - The Nix side: `home/state.nix`.

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fmt;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;

use serde::Deserialize;
use serde::Serialize;

use crate::packages::InvalidName;
use crate::packages::validate_name;
use crate::state::GitError;
use crate::state::NIXPKGS_FILE;
use crate::state::ReadError;
use crate::state::StateDir;
use crate::state::lock_file;
use crate::state::write_atomic;

/// The comment `nixpkgs.toml` starts with.
const HEADER: &str = "\
# Whether your nixpkgs takes unfree packages. The kubaos command rewrites this file: change it
# with `kubaos package unfree on|off`, or from the Command menu.
";

/// The function `nix eval` applies to the user's package set, before the list of names: each
/// name's `meta.unfree`, or `null` for a name that is no package there or whose meta fails to
/// evaluate (a removed package's alias throws).
const UNFREE_OF: &str = r#"pkgs: map (name:
  let
    found = builtins.tryEval (
      let package = pkgs.lib.attrByPath (pkgs.lib.splitString "." name) null pkgs;
      in if pkgs.lib.isDerivation package then package.meta.unfree or false else null);
  in if found.success then found.value else null)"#;

/// Whether the user's nixpkgs takes unfree packages.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unfree {
    /// Unfree packages install like any other.
    Allowed,
    /// Only free packages install; an unfree one fails the build.
    Refused,
}

/// Unfree packages are allowed until the user turns them off: KubaOS already ships unfree software
/// (Chrome, through apt), and `kubaos package add discord` should just work.
pub const DEFAULT: Unfree = Unfree::Allowed;

impl Default for Unfree {
    fn default() -> Self {
        DEFAULT
    }
}

/// `nixpkgs.toml` in one state directory, and the lock its edits take.
#[derive(Clone, Debug)]
pub struct Setting {
    state: StateDir,
    file: PathBuf,
    lock: PathBuf,
}

/// Changing `nixpkgs.toml` failed. The file is as it was, unless the failure is in git.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SettingError {
    /// The edit lock could not be taken.
    #[error("locking {}", path.display())]
    Lock {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    /// The new file could not be written.
    #[error("writing {}", path.display())]
    Write {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    /// The file is written, but git's index could not be brought in step with it, so Nix may not
    /// see it.
    #[error("recording nixpkgs.toml in git")]
    Git(#[source] GitError),
}

#[derive(Deserialize, Serialize)]
struct NixpkgsFile {
    allow_unfree: Option<bool>,
}

impl Setting {
    /// `nixpkgs.toml` in `state`, with `lock` as the edit lock (`Paths::edit_lock`).
    #[must_use]
    pub fn new(state: &StateDir, lock: impl Into<PathBuf>) -> Self {
        Self {
            state: state.clone(),
            file: state.nixpkgs_file(),
            lock: lock.into(),
        }
    }

    /// The setting in the state directory at `state_dir`, [`DEFAULT`] without the file or the
    /// key.
    ///
    /// # Errors
    ///
    /// [`ReadError`] when the file exists but cannot be read, or is not TOML with a boolean
    /// `allow_unfree`.
    pub fn read(state_dir: &Path) -> Result<Unfree, ReadError> {
        let path = state_dir.join(NIXPKGS_FILE);
        let text = match fs::read_to_string(&path) {
            Ok(text) => text,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(DEFAULT),
            Err(e) => return Err(ReadError::read(&path)(e)),
        };
        let file: NixpkgsFile = toml::from_str(&text).map_err(ReadError::parse(&path))?;
        Ok(match file.allow_unfree {
            Some(true) => Unfree::Allowed,
            Some(false) => Unfree::Refused,
            None => DEFAULT,
        })
    }

    /// The setting, [`DEFAULT`] without the file or the key.
    ///
    /// # Errors
    ///
    /// [`ReadError`] when the file exists but cannot be read or parsed.
    pub fn get(&self) -> Result<Unfree, ReadError> {
        Self::read(self.state.root())
    }

    /// Writes `unfree` to `nixpkgs.toml`, creating the state directory when it is missing, and
    /// records the file in git. Returns whether the setting changed; when it didn't, nothing is
    /// written.
    ///
    /// # Errors
    ///
    /// [`SettingError`] when the lock, the write or git fails. A file that doesn't parse is
    /// replaced.
    pub fn set(&self, unfree: Unfree) -> Result<bool, SettingError> {
        let _lock = lock_file(&self.lock).map_err(|source| SettingError::Lock {
            path: self.lock.clone(),
            source,
        })?;
        if self.get().ok() == Some(unfree) {
            return Ok(false);
        }
        self.write(unfree)?;
        self.state.sync_git().map_err(SettingError::Git)?;
        Ok(true)
    }

    fn write(&self, unfree: Unfree) -> Result<(), SettingError> {
        let write_err = |source| SettingError::Write {
            path: self.file.clone(),
            source,
        };
        let body = toml::to_string(&NixpkgsFile {
            allow_unfree: Some(unfree == Unfree::Allowed),
        })
        .map_err(|e| write_err(io::Error::other(e)))?;
        if let Some(dir) = self.file.parent() {
            fs::create_dir_all(dir).map_err(write_err)?;
        }
        write_atomic(&self.file, &format!("{HEADER}{body}")).map_err(write_err)
    }
}

/// Whether one package is unfree.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum License {
    Free,
    Unfree,
    /// The name is no package in the user's nixpkgs, or its meta doesn't evaluate.
    Unknown,
}

/// What one [`LicenseCheck::run`] found, by name.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Licenses {
    by_name: BTreeMap<String, License>,
}

impl Licenses {
    /// The license of `name`, [`License::Unknown`] for a name the check wasn't asked about.
    pub fn get(&self, name: &str) -> License {
        self.by_name.get(name).copied().unwrap_or(License::Unknown)
    }

    /// Whether `name` is unfree.
    pub fn is_unfree(&self, name: &str) -> bool {
        self.get(name) == License::Unfree
    }

    /// The unfree packages, sorted.
    pub fn unfree(&self) -> Vec<&str> {
        self.by_name
            .iter()
            .filter(|(_, license)| **license == License::Unfree)
            .map(|(name, _)| name.as_str())
            .collect()
    }
}

/// The unfree packages the setting turns away. Its message names the setting and how to turn it
/// back on.
#[derive(Debug, thiserror::Error)]
#[error(
    "{} unfree, and unfree packages are turned off (allow_unfree in kubaos.d/nixpkgs.toml): turn \
     them back on with `kubaos package unfree on`, or \"Allow unfree packages\" in the Command menu",
    Names(names)
)]
pub struct Refused {
    names: Vec<String>,
}

impl Refused {
    /// The unfree packages, sorted.
    pub fn names(&self) -> &[String] {
        &self.names
    }
}

/// `a is`, `a and b are`, `a, b and c are`.
struct Names<'a>(&'a [String]);

impl fmt::Display for Names<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Some((last, rest)) = self.0.split_last() else {
            return Ok(());
        };
        if !rest.is_empty() {
            write!(f, "{} and ", rest.join(", "))?;
        }
        let verb = if rest.is_empty() { "is" } else { "are" };
        write!(f, "{last} {verb}")
    }
}

impl Unfree {
    /// Whether the setting lets in the packages `licenses` describes: every one while unfree
    /// packages are allowed, else only those not known to be unfree.
    ///
    /// # Errors
    ///
    /// [`Refused`] with the unfree ones, while unfree packages are refused.
    pub fn admit(self, licenses: &Licenses) -> Result<(), Refused> {
        let unfree = licenses.unfree();
        if self == Unfree::Allowed || unfree.is_empty() {
            return Ok(());
        }
        Err(Refused {
            names: unfree.into_iter().map(str::to_owned).collect(),
        })
    }
}

/// The `nix eval` that reads whether packages of the user flake's nixpkgs are unfree.
#[derive(Clone, Debug)]
pub struct LicenseCheck {
    program: OsString,
    flake: PathBuf,
    system: String,
}

/// Telling which packages are unfree failed.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum CheckError {
    /// One of the names is not a package name.
    #[error("invalid package name")]
    InvalidName(#[source] InvalidName),
    /// `nix` could not be started.
    #[error("running {program}")]
    Spawn {
        program: String,
        #[source]
        source: io::Error,
    },
    /// `nix eval` failed; `message` is the last line it printed on stderr.
    #[error("nix eval failed ({status}): {message}")]
    Failed { status: String, message: String },
    /// `nix eval` printed something other than one license per name.
    #[error("reading what nix eval printed")]
    Answer(#[source] serde_json::Error),
}

impl LicenseCheck {
    /// The check against the nixpkgs input of the user flake at `flake`, on this machine's
    /// system, run with the `nix` on `PATH`.
    #[must_use]
    pub fn new(flake: impl Into<PathBuf>) -> Self {
        Self {
            program: "nix".into(),
            flake: flake.into(),
            system: format!("{}-linux", std::env::consts::ARCH),
        }
    }

    /// The same check, run with `program` in place of `nix`.
    #[must_use]
    pub fn with_program(mut self, program: impl Into<OsString>) -> Self {
        self.program = program.into();
        self
    }

    /// The command that prints, as a JSON list, `true`, `false` or `null` for each of `names`.
    ///
    /// # Errors
    ///
    /// [`InvalidName`] when a name is not a package name, which also keeps every name safe as a
    /// Nix string.
    pub fn command(&self, names: &[impl AsRef<str>]) -> Result<Vec<OsString>, InvalidName> {
        let mut quoted = Vec::with_capacity(names.len());
        for name in names {
            let name = name.as_ref();
            validate_name(name)?;
            quoted.push(format!("\"{name}\""));
        }
        let list = format!("[ {} ]", quoted.join(" "));
        Ok(vec![
            self.program.clone(),
            "eval".into(),
            "--json".into(),
            "--inputs-from".into(),
            self.flake.as_os_str().to_owned(),
            format!("nixpkgs#legacyPackages.{}", self.system).into(),
            "--apply".into(),
            format!("{UNFREE_OF} {list}").into(),
        ])
    }

    /// Whether each of `names` is unfree, from one `nix eval`. No names runs nothing.
    ///
    /// # Errors
    ///
    /// [`CheckError`] when a name is invalid, or `nix eval` can't start, fails or answers
    /// something else.
    pub fn run(&self, names: &[impl AsRef<str>]) -> Result<Licenses, CheckError> {
        if names.is_empty() {
            return Ok(Licenses::default());
        }
        let argv = self.command(names).map_err(CheckError::InvalidName)?;
        let program = self.program.to_string_lossy().into_owned();
        let out = Command::new(&argv[0])
            .args(&argv[1..])
            .stdin(Stdio::null())
            .output()
            .map_err(|source| CheckError::Spawn { program, source })?;
        if !out.status.success() {
            let stderr = String::from_utf8_lossy(&out.stderr);
            return Err(CheckError::Failed {
                status: out.status.to_string(),
                message: stderr
                    .lines()
                    .rev()
                    .find(|l| !l.trim().is_empty())
                    .unwrap_or("")
                    .trim()
                    .to_owned(),
            });
        }
        let answers: Vec<Option<bool>> = serde_json::from_slice(&out.stdout).map_err(CheckError::Answer)?;
        if answers.len() != names.len() {
            return Err(CheckError::Answer(serde::de::Error::invalid_length(
                answers.len(),
                &"one value per package",
            )));
        }
        let by_name = names
            .iter()
            .zip(answers)
            .map(|(name, unfree)| {
                let license = match unfree {
                    Some(true) => License::Unfree,
                    Some(false) => License::Free,
                    None => License::Unknown,
                };
                (name.as_ref().to_owned(), license)
            })
            .collect();
        Ok(Licenses { by_name })
    }
}

/// Licenses from names and their licenses.
impl FromIterator<(String, License)> for Licenses {
    fn from_iter<I: IntoIterator<Item = (String, License)>>(iter: I) -> Self {
        Self {
            by_name: iter.into_iter().collect(),
        }
    }
}

#[cfg(test)]
mod tests;
