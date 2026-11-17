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
pub mod settings;

use std::collections::BTreeMap;
use std::fs;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use serde::Deserialize;
use serde::Serialize;

use super::folder::is_app_id;
use crate::apt::manifest::is_package_name;
use crate::packages::validate_name;

/// The folders under which an action may write as root. Everything else on the system is the
/// base's or a package's.
pub const SYSTEM_PREFIXES: [&str; 3] = ["/etc/", "/opt/", "/usr/local/"];

/// The architectures a pinned download may be for, as `dpkg --print-architecture` names them.
pub const ARCHES: [&str; 2] = ["amd64", "arm64"];

/// The architectures dpkg may install packages of beside the machine's own, as `dpkg
/// --add-architecture` names them: 32-bit x86 beside amd64 (Steam's and Wine's libraries), 32-bit
/// ARM beside arm64, and each 64-bit one.
pub const FOREIGN_ARCHES: [&str; 4] = ["amd64", "arm64", "armhf", "i386"];

/// The packages an `apt` action installs on one architecture alone, beside its `packages`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArchPackages {
    pub packages: Vec<String>,
}

/// One pinned download: an https URL and the sha256 of what it serves.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Download {
    pub url: String,
    pub sha256: String,
}

/// The browser a `web_app` opens in: its `--app` windows are the app windows.
pub const WEB_APP_BROWSER: &str = "google-chrome";

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
    #[error("{name:?} is not an apt package name, with `:<architecture>` for another architecture's")]
    InvalidPackage { name: String },
    #[error("{name:?} is not a nixpkgs attribute name")]
    InvalidAttribute { name: String },
    #[error("{name:?} is not a repository name: lowercase letters, digits and dashes")]
    InvalidRepo { name: String },
    #[error("{value:?} is not a pattern of versions: a version, where `*` stands for any run of characters")]
    InvalidVersions { value: String },
    #[error("{value:?} is not an architecture dpkg names: amd64 or arm64")]
    InvalidArch { value: String },
    #[error("{value:?} is not an architecture dpkg installs beside the machine's: amd64, arm64, armhf or i386")]
    InvalidForeignArch { value: String },
    #[error("{value:?} is not a DKMS module's name: lowercase letters, digits, `-` and `_`")]
    InvalidModule { value: String },
    #[error("{value:?} is not a DKMS module's version: digits, letters and `.+~-`, starting with a digit")]
    InvalidModuleVersion { value: String },
    #[error("{value:?} is not a folder of the unpacked source: a relative path, no `.` or `..`")]
    InvalidSourceFolder { value: String },
    #[error("an apt action for {arch} alone lists no packages of another architecture's")]
    ArchPackagesWithArch { arch: String },
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
    #[error("a setting's file is a .json, .toml or .conf file in the home; {value:?} isn't")]
    InvalidSettingFile { value: String },
    #[error(
        "{value:?} is not a setting's key: dot-separated names, one bare name in a .conf, letters, digits, `_`, `.` and `-` under an INI file's section"
    )]
    InvalidKey { value: String },
    #[error("{value:?} is not an INI file's section: a name in a .conf of letters, digits, `_`, `.` and `-`")]
    InvalidSection { value: String },
    #[error(
        "an INI file's value is text Qt writes as it is: no quotes, `\\`, `,`, `;`, `=`, control characters, leading `@` or spaces around it"
    )]
    InvalidIniValue,
    #[error("a setting's value is a string, a number or a boolean")]
    InvalidValue,
    #[error("{path:?} isn't text, so it can't be a template")]
    BinaryTemplate { path: PathBuf },
    #[error("{value:?} is not a systemd unit name")]
    InvalidUnit { value: String },
    #[error("a unit action enables or disables, starts or stops: at least one of `enable` and `start`")]
    UnitDoesNothing,
    #[error("{value:?} is not a group name")]
    InvalidGroup { value: String },
    #[error("a run's {key} is an argv: a program on PATH or ./<script> in the app's folder, then its arguments")]
    InvalidArgv { key: &'static str },
    #[error("a run has both `do` and `check`, or neither, for one that only undoes")]
    RunHalf,
    #[error("a run that only undoes does nothing in a window, and installs no source")]
    RemovalOnlyExtra,
    #[error("{value:?} is not a web app's id: lowercase letters, digits and dashes")]
    InvalidWebApp { value: String },
    #[error(
        "{value:?} is not an https:// URL, or an http:// one on this machine (localhost, 127.0.0.1 or [::1]) with its port"
    )]
    InvalidUrl { value: String },
    #[error("a web app's title is one line, with no control characters")]
    InvalidTitle,
    #[error(
        "a web app's icon is an .svg file in the app's folder or an icon theme's name for an icon; {value:?} is neither"
    )]
    InvalidIcon { value: String },
    #[error("{key} is the journal's own; an app's folder doesn't set it")]
    JournalOnly { key: &'static str },
    #[error("{value:?} is not a stage: \"install\" or \"configure\"")]
    InvalidStage { value: String },
    #[error("it names @VERSION@, and the app has no download whose version it would be")]
    NoVersion,
    #[error("a download its updater rewrites in place names no version: {value:?} names its own without `@VERSION@`")]
    SelfUpdatesNamesVersion { value: String },
}

/// One step of an install: its name and what it does.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "toml::Table")]
pub struct Action {
    /// The step as the user reads it: "Start the service".
    pub name: String,
    /// The stage it is part of: the install, or the configure stage after it.
    #[serde(default, skip_serializing_if = "Stage::is_install")]
    pub stage: Stage,
    #[serde(flatten)]
    pub kind: Kind,
}

/// The part of an install an action is in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Stage {
    /// Putting the app on the machine.
    #[default]
    Install,
    /// Setting the app up the KubaOS way, its defaults in the user layer, after the install;
    /// `kubaos app configure` runs it alone again.
    Configure,
}

impl Stage {
    /// Whether it is the install stage, which `app.toml` and a journal leave unsaid.
    pub fn is_install(&self) -> bool {
        *self == Self::Install
    }
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
        /// The versions that are the app's, as a pattern where `*` stands for any run of
        /// characters: a package dpkg has at another version is another build of that name (a
        /// distribution's stand-in for a snap), which doesn't count and which the install
        /// replaces.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        versions: Option<String>,
        /// Its undo purges its packages and what it pulled in, rather than keeping their
        /// configuration: for a package whose conffiles are KubaOS's business, as Steam's apt
        /// source is.
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        purge: bool,
        /// The packages it installs on amd64 alone, beside `packages`: what one architecture
        /// needs that the other has no package of (a virtual machine's firmware). Resolving puts
        /// the machine's into `packages`, so a journal records one list.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        amd64: Option<ArchPackages>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        arm64: Option<ArchPackages>,
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
        /// The one architecture nixpkgs builds them for, when it builds them for one alone.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        arch: Option<String>,
        /// nixpkgs marks one of them unfree (`meta.unfree`): only a user whose nixpkgs takes unfree
        /// packages can have them, as an unfree name fails the home's switch for everyone else.
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        unfree: bool,
    },
    /// A pinned download, unpacked to a path.
    Download {
        to: String,
        version: String,
        unpack: Unpack,
        #[serde(default)]
        strip: u32,
        /// A vendor's updater moves it on: the same path in a newer version's folder stands for
        /// the pin, which the updater may have removed.
        #[serde(default)]
        self_updates: bool,
        /// A self-updating `to` naming no `@VERSION@`, which the vendor's updater rewrites in
        /// place, set as the action resolves from the folder's `to`: the filled one may hold the
        /// version's text in the home or the user's name. The journal's own; a folder never sets
        /// it.
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        in_place: bool,
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
        /// The vendor's own package writes the file too, as `key=value` lines: one there already
        /// that sets no key but the ones this text sets is the vendor's, not the user's. The step
        /// writes this text over it and owns nothing, so a removal leaves it, with these values.
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        vendor_keys: bool,
        /// A journal's: `text` is the base64 of the bytes of a `from` that isn't UTF-8 text, a
        /// picture's, so the journal can hold them and the step write them back.
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        binary: bool,
    },
    Symlink {
        to: String,
        target: String,
    },
    /// A key in an app's JSON, TOML or flat `key = value` `.conf` settings, or under `section` of
    /// an INI `.conf`, the file's other keys kept.
    Setting {
        file: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        section: Option<String>,
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
    /// dpkg taking packages of another architecture than the machine's (`dpkg --add-architecture`),
    /// apt's lists then read again, so a later `apt` step installs `<package>:<architecture>`.
    /// Shared by claim: an app that finds it there owns nothing of it, and the last app that owns
    /// it takes it away only once dpkg has no package of it left.
    Architecture {
        architecture: String,
    },
    /// A kernel module built from a pinned source by DKMS for every kernel with its headers, and
    /// again by DKMS for each kernel installed later, signed with the machine's Machine Owner Key
    /// as DKMS signs: the source unpacked to `/usr/src/<module>-<version>` with KubaOS's
    /// `dkms.conf`, registered (`dkms add`) and installed (`dkms install`).
    Dkms {
        /// DKMS's name for it, and the one kernel module it builds: `hid-xpadneo`.
        module: String,
        version: String,
        /// A gzipped tarball of the source, over https.
        url: String,
        sha256: String,
        /// The leading folders of the tarball's paths left out, as tar's `--strip-components`.
        #[serde(default)]
        strip: u32,
        /// The folder of the unpacked tarball that holds the module's source, when it isn't the
        /// tarball's top.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        folder: Option<String>,
        /// The `dkms.conf` the source gets, a file of the app's folder, its words filled in and
        /// `@VERSION@` the module's version.
        conf: String,
        /// That file's text, read from the folder as the action resolves: the journal's own.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        conf_text: Option<String>,
    },
    /// A web app in a window of its own: a desktop entry and an icon that open `url` as a Chrome
    /// app window, in a browser folder and a profile of its own, which give the window an app id
    /// of its own.
    WebApp {
        /// What names its desktop entry and its icon, `kubaos-web-<id>`, its browser folder, and
        /// its profile, `kubaos-web-<id>`, which ends the app id Chrome gives its window.
        id: String,
        /// The name its desktop entry shows.
        title: String,
        /// An `https://` URL, or an `http://` one of this machine with its port.
        url: String,
        /// An `.svg` in the app's folder, or the icon theme's name for an icon.
        icon: String,
        /// An `.svg` icon, read from the folder as the action resolves: the journal's own.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        svg: Option<String>,
    },
    /// A path a program of the app writes later, which the app owns all the same.
    Output {
        path: String,
    },
    /// A vendor tool's own command, with how to check it and undo it; or, with neither `do` nor
    /// `check`, an `undo` alone, which a removal runs and an install never does.
    Run {
        #[serde(rename = "do", default, skip_serializing_if = "Vec::is_empty")]
        run: Vec<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        check: Vec<String>,
        undo: Vec<String>,
        #[serde(default)]
        root: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        window: Option<String>,
        /// What it puts on the machine, for a `run` that installs the app itself: the source the
        /// catalog lists for it, as no other action of the install names one.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        source: Option<RunSource>,
        /// The `./<script>`s its argvs name, by name, read from the app's folder as it resolves:
        /// what a journal runs, so an undo runs the script that matches what was done even after
        /// the folder changed or left the catalog. A folder never sets it.
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        scripts: BTreeMap<String, String>,
    },
}

/// What a `run` that installs the app takes it from: the sources no other kind of action installs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum RunSource {
    /// Container images, each pinned by its digest: `container_images`.
    ContainerImages,
}

/// How a download becomes its path.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Unpack {
    /// The download is the file.
    File,
    /// A gzipped tarball, unpacked into the path as a folder: `tar-gz`.
    TarGz,
    /// A zstd-compressed tarball, unpacked into the path as a folder: `tar-zst`.
    TarZst,
    /// An xz-compressed tarball, unpacked into the path as a folder: `tar-xz`.
    TarXz,
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Side {
    User,
    Root,
}

/// The words an action's strings take: `@USER@`, `@HOME@`, `@ARCH@`, and `@VERSION@`, the version
/// of a download, or of the app's downloads in the app's other actions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Words {
    pub user: String,
    pub home: PathBuf,
    pub arch: String,
    /// The version of the app's downloads, which every download of an app shares; none for an
    /// app without one.
    pub version: Option<String>,
}

impl Words {
    /// `text` with its words filled in, `@VERSION@` with `version`, else the app's.
    pub fn fill(&self, text: &str, version: Option<&str>) -> String {
        let mut out = text
            .replace("@USER@", &self.user)
            .replace("@HOME@", &self.home.to_string_lossy())
            .replace("@ARCH@", &self.arch);
        if let Some(version) = version.or(self.version.as_deref()) {
            out = out.replace("@VERSION@", version);
        }
        out
    }

    /// The same, for an app whose downloads are of `version`.
    #[must_use]
    pub fn with_version(mut self, version: Option<&str>) -> Self {
        self.version = version.map(str::to_owned);
        self
    }

    /// Where `path`, as an action names it, is: `~/…` under the home, else itself.
    pub fn path(&self, path: &str) -> PathBuf {
        match path.strip_prefix("~/") {
            Some(rest) => self.home.join(rest),
            None => PathBuf::from(path),
        }
    }
}

impl Action {
    /// An action as an app's folder declares it, refusing what only a journal records.
    ///
    /// # Errors
    ///
    /// [`ActionError`] for an action that isn't valid.
    pub fn from_folder(table: toml::Table) -> Result<Self, ActionError> {
        for key in ["scripts", "svg", "in_place", "conf_text", "binary"] {
            if table.contains_key(key) {
                return Err(ActionError::JournalOnly { key });
            }
        }
        let action = Self::try_from(table)?;
        action.check_unresolved()?;
        Ok(action)
    }

    /// What only the folder's form of an action can be checked for: a journal records it with
    /// its words filled in.
    fn check_unresolved(&self) -> Result<(), ActionError> {
        match &self.kind {
            Kind::Run {
                run, check, undo, root, ..
            } => {
                for (key, argv) in [("do", run), ("check", check), ("undo", undo)] {
                    if argv.is_empty() && key != "undo" {
                        continue;
                    }
                    // HAZARD: root never runs a program from the home: the user could change it.
                    let from_home = argv.first().is_some_and(|p| p.starts_with("@HOME@/"));
                    if !is_argv(argv) || (*root && from_home) {
                        return Err(ActionError::InvalidArgv { key });
                    }
                }
                Ok(())
            }
            // A look tells a `to` its updater rewrites in place from one of its version's folder
            // by `@VERSION@`, so one naming no `@VERSION@` names no version either.
            Kind::Download {
                to,
                version,
                self_updates: true,
                ..
            } if !to.contains("@VERSION@") && to.contains(version.as_str()) => {
                Err(ActionError::SelfUpdatesNamesVersion { value: to.clone() })
            }
            _ => Ok(()),
        }
    }
}

impl TryFrom<toml::Table> for Action {
    type Error = ActionError;

    fn try_from(mut table: toml::Table) -> Result<Self, Self::Error> {
        let Some(toml::Value::String(name)) = table.remove("name") else {
            return Err(ActionError::NoName);
        };
        let stage = match table.remove("stage") {
            None => Stage::Install,
            Some(toml::Value::String(word)) if word == "install" => Stage::Install,
            Some(toml::Value::String(word)) if word == "configure" => Stage::Configure,
            Some(other) => {
                return Err(ActionError::InvalidStage {
                    value: other.to_string(),
                });
            }
        };
        let kind = Kind::deserialize(toml::Value::Table(table)).map_err(ActionError::Parse)?;
        let action = Self { name, stage, kind };
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
            Kind::Apt { .. }
            | Kind::Deb { .. }
            | Kind::Group { .. }
            | Kind::Architecture { .. }
            | Kind::Dkms { .. } => Side::Root,
            Kind::Nix { .. } | Kind::Setting { .. } | Kind::Linger {} | Kind::WebApp { .. } => Side::User,
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

    /// Whether it is a `run` with an `undo` alone: nothing to do or check, its step the app's from
    /// the run that records it, so a removal runs its `undo`.
    #[must_use]
    pub fn removal_only(&self) -> bool {
        matches!(&self.kind, Kind::Run { run, check, .. } if run.is_empty() && check.is_empty())
    }

    /// Whether `other` acts on the same thing, so that a release changing one into the other
    /// changes the step, which is then updated in place: the same kind on the same side and, for
    /// a kind whose thing has a name, the same name. That is a `deb`'s package, a `file`'s,
    /// `symlink`'s or `output`'s path, a `setting`'s file and key, a `unit`'s unit and scope and a
    /// `group`'s group; an `apt` or `nix` action (whose list may change), a `download` (whose path
    /// may carry its version), `linger` and a `run` are the same thing as any of their kind, and a
    /// `run` that only undoes as any other that only undoes.
    pub fn same_thing(&self, other: &Self) -> bool {
        if self.side() != other.side() {
            return false;
        }
        match (&self.kind, &other.kind) {
            (Kind::Apt { .. }, Kind::Apt { .. })
            | (Kind::Nix { .. }, Kind::Nix { .. })
            | (Kind::Download { .. }, Kind::Download { .. })
            | (Kind::Linger {}, Kind::Linger {}) => true,
            (Kind::Run { .. }, Kind::Run { .. }) => self.removal_only() == other.removal_only(),
            (Kind::Deb { package: a, .. }, Kind::Deb { package: b, .. })
            | (Kind::Group { group: a }, Kind::Group { group: b })
            | (Kind::Architecture { architecture: a }, Kind::Architecture { architecture: b })
            | (Kind::Dkms { module: a, .. }, Kind::Dkms { module: b, .. })
            | (Kind::WebApp { id: a, .. }, Kind::WebApp { id: b, .. })
            | (Kind::File { to: a, .. }, Kind::File { to: b, .. })
            | (Kind::Symlink { to: a, .. }, Kind::Symlink { to: b, .. })
            | (Kind::Output { path: a }, Kind::Output { path: b }) => a == b,
            (
                Kind::Setting {
                    file: file_a,
                    section: section_a,
                    key: key_a,
                    ..
                },
                Kind::Setting {
                    file: file_b,
                    section: section_b,
                    key: key_b,
                    ..
                },
            ) => file_a == file_b && section_a == section_b && key_a == key_b,
            (Kind::Unit { unit: a, scope: x, .. }, Kind::Unit { unit: b, scope: y, .. }) => a == b && x == y,
            _ => false,
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
            Kind::Apt { arch: Some(only), .. } | Kind::Nix { arch: Some(only), .. } => only == arch,
            Kind::Deb { .. } | Kind::Download { .. } => self.download(arch).is_some(),
            _ => true,
        }
    }

    /// Whether it installs a package nixpkgs marks unfree, which a user whose nixpkgs refuses
    /// unfree packages can't have.
    pub fn needs_unfree(&self) -> bool {
        matches!(self.kind, Kind::Nix { unfree: true, .. })
    }

    /// The action as it runs on this machine and as a journal records it: its words filled in,
    /// a file's `from` read from `folder` into its `text`, and a run's `./<script>`s into its
    /// `scripts`.
    ///
    /// # Errors
    ///
    /// [`ActionError::ReadFrom`] when a file's `from` or a run's script can't be read.
    pub fn resolve(&self, words: &Words, folder: &Path) -> Result<Self, ActionError> {
        let fill = |text: &str, version: Option<&str>| words.fill(text, version);
        let kind = match &self.kind {
            Kind::Apt { .. } => resolve_apt(&words.arch, &self.kind),
            Kind::Download { .. } => resolve_download(words, &self.kind),
            Kind::File { .. } => resolve_file(words, folder, &self.kind)?,
            Kind::Symlink { to, target } => {
                let target = fill(target, None);
                if target.contains("@VERSION@") {
                    return Err(ActionError::NoVersion);
                }
                Kind::Symlink {
                    to: fill(to, None),
                    target,
                }
            }
            Kind::Setting {
                file,
                section,
                key,
                value,
            } => Kind::Setting {
                file: fill(file, None),
                section: section.clone(),
                key: key.clone(),
                value: value.clone(),
            },
            Kind::Output { path } => Kind::Output { path: fill(path, None) },
            Kind::Dkms { conf, version, .. } => {
                let path = folder.join(conf);
                let text = fs::read_to_string(&path).map_err(|source| ActionError::ReadFrom { path, source })?;
                let mut kind = self.kind.clone();
                if let Kind::Dkms { conf_text, .. } = &mut kind {
                    *conf_text = Some(words.fill(&text, Some(version)));
                }
                kind
            }
            Kind::WebApp {
                id, title, url, icon, ..
            } => {
                let svg = if is_svg_icon(icon) {
                    let path = folder.join(icon);
                    Some(fs::read_to_string(&path).map_err(|source| ActionError::ReadFrom { path, source })?)
                } else {
                    None
                };
                Kind::WebApp {
                    id: id.clone(),
                    title: title.clone(),
                    url: fill(url, None),
                    icon: icon.clone(),
                    svg,
                }
            }
            Kind::Run {
                run,
                check,
                undo,
                root,
                window,
                source,
                ..
            } => {
                let argv = |argv: &[String]| argv.iter().map(|a| fill(a, None)).collect();
                let mut scripts = BTreeMap::new();
                for program in [run, check, undo].into_iter().filter_map(|argv| argv.first()) {
                    if let Some(name) = program.strip_prefix("./")
                        && !scripts.contains_key(name)
                    {
                        let path = folder.join(name);
                        let text =
                            fs::read_to_string(&path).map_err(|source| ActionError::ReadFrom { path, source })?;
                        scripts.insert(name.to_owned(), text);
                    }
                }
                Kind::Run {
                    run: argv(run),
                    check: argv(check),
                    undo: argv(undo),
                    root: *root,
                    window: window.clone(),
                    source: *source,
                    scripts,
                }
            }
            other => other.clone(),
        };
        Ok(Self {
            name: self.name.clone(),
            stage: self.stage,
            kind,
        })
    }

    fn check(&self) -> Result<(), ActionError> {
        if self.name.trim().is_empty() {
            return Err(ActionError::NoName);
        }
        match &self.kind {
            Kind::Apt {
                packages,
                repo,
                arch,
                versions,
                amd64,
                arm64,
                ..
            } => {
                check_apt(packages, repo.as_deref(), arch.as_deref(), versions.as_deref())?;
                for own in [amd64, arm64].into_iter().flatten() {
                    if let Some(arch) = arch {
                        return Err(ActionError::ArchPackagesWithArch { arch: arch.clone() });
                    }
                    check_apt(&own.packages, None, None, None)?;
                }
            }
            Kind::Deb { package, .. } => {
                if !is_package_name(package) {
                    return Err(ActionError::InvalidPackage { name: package.clone() });
                }
                self.check_downloads()?;
            }
            Kind::Nix { packages, arch, .. } => {
                non_empty("packages", packages)?;
                for name in packages {
                    validate_name(name).map_err(|_| ActionError::InvalidAttribute { name: name.clone() })?;
                }
                if let Some(arch) = arch {
                    check_arch(arch)?;
                }
            }
            Kind::Download { to, .. } => {
                check_path(to)?;
                self.check_downloads()?;
            }
            Kind::File {
                to, from, text, mode, ..
            } => check_file(to, from.as_deref(), text.is_some(), mode.as_deref())?,
            Kind::Symlink { to, target } => {
                check_path(to)?;
                if target.is_empty() {
                    return Err(ActionError::Empty { key: "target" });
                }
            }
            Kind::Setting {
                file,
                section,
                key,
                value,
            } => check_setting(file, section.as_deref(), key, value)?,
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
            Kind::Dkms { .. } => check_dkms(&self.kind)?,
            Kind::Architecture { architecture } if !FOREIGN_ARCHES.contains(&architecture.as_str()) => {
                return Err(ActionError::InvalidForeignArch {
                    value: architecture.clone(),
                });
            }
            Kind::Linger {} | Kind::Architecture { .. } => {}
            Kind::WebApp {
                id, title, url, icon, ..
            } => check_web_app(id, title, url, icon)?,
            Kind::Output { path } => check_path(path)?,
            Kind::Run {
                run,
                check,
                undo,
                window,
                source,
                ..
            } => check_run(run, check, undo, window.is_some() || source.is_some())?,
        }
        Ok(())
    }

    fn check_downloads(&self) -> Result<(), ActionError> {
        let found: Vec<&Download> = ARCHES.iter().filter_map(|arch| self.download(arch)).collect();
        if found.is_empty() {
            return Err(ActionError::NoDownload);
        }
        found.into_iter().try_for_each(check_pin)
    }
}

/// Whether a pinned download is an https URL with a sha256 of 64 lowercase hex digits.
fn check_pin(download: &Download) -> Result<(), ActionError> {
    let url = is_https(&download.url);
    let sha = download.sha256.len() == 64
        && download
            .sha256
            .chars()
            .all(|c| c.is_ascii_digit() || matches!(c, 'a'..='f'));
    if url && sha {
        Ok(())
    } else {
        Err(ActionError::InvalidDownload {
            value: download.url.clone(),
        })
    }
}

/// Whether a `dkms` step's module, version, pinned source, folder in it and `dkms.conf` are well
/// formed: names DKMS takes in a path, `/usr/src/<module>-<version>`, and a file of the folder.
fn check_dkms(kind: &Kind) -> Result<(), ActionError> {
    let Kind::Dkms {
        module,
        version,
        url,
        sha256,
        folder,
        conf,
        ..
    } = kind
    else {
        return Ok(());
    };
    let module_ok = !module.is_empty()
        && module.len() <= 64
        && module.chars().next().is_some_and(|c| c.is_ascii_alphanumeric())
        && module
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '-' | '_'));
    if !module_ok {
        return Err(ActionError::InvalidModule { value: module.clone() });
    }
    let version_ok = version.chars().next().is_some_and(|c| c.is_ascii_digit())
        && version.len() <= 64
        && version
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '+' | '~' | '-'));
    if !version_ok {
        return Err(ActionError::InvalidModuleVersion { value: version.clone() });
    }
    check_pin(&Download {
        url: url.clone(),
        sha256: sha256.clone(),
    })?;
    if let Some(folder) = folder
        && !is_folder_path(folder)
    {
        return Err(ActionError::InvalidSourceFolder { value: folder.clone() });
    }
    if !is_folder_file(conf) {
        return Err(ActionError::InvalidFrom { value: conf.clone() });
    }
    Ok(())
}

/// Where DKMS takes a module's source from: `/usr/src/<module>-<version>`.
pub fn dkms_source(module: &str, version: &str) -> String {
    format!("/usr/src/{module}-{version}")
}

/// An `apt` action as it runs on a machine of `arch`: that architecture's packages after its
/// own, in one list.
fn resolve_apt(arch: &str, kind: &Kind) -> Kind {
    let mut kind = kind.clone();
    if let Kind::Apt {
        packages, amd64, arm64, ..
    } = &mut kind
    {
        let (amd64, arm64) = (amd64.take(), arm64.take());
        let own = match arch {
            "amd64" => amd64,
            "arm64" => arm64,
            _ => None,
        };
        packages.extend(own.into_iter().flat_map(|o| o.packages));
    }
    kind
}

/// A download as it runs: its `to` with its words and version filled in, and whether its
/// updater rewrites it in place, read before the filling.
fn resolve_download(words: &Words, kind: &Kind) -> Kind {
    let mut kind = kind.clone();
    if let Kind::Download {
        to,
        version,
        self_updates,
        in_place,
        ..
    } = &mut kind
    {
        *in_place = *self_updates && !to.contains("@VERSION@");
        *to = words.fill(to, Some(version));
    }
    kind
}

/// A `file` as its journal records it: its words filled in and its `from` read into its text.
fn resolve_file(words: &Words, folder: &Path, kind: &Kind) -> Result<Kind, ActionError> {
    let Kind::File {
        to,
        from,
        text,
        template,
        mode,
        vendor_keys,
        ..
    } = kind
    else {
        unreachable!("resolve_file resolves a file step");
    };
    let (text, binary) = file_text(words, folder, from.as_deref(), text.as_deref(), *template)?;
    Ok(Kind::File {
        to: words.fill(to, None),
        from: None,
        text: Some(text),
        template: false,
        mode: mode.clone(),
        vendor_keys: *vendor_keys,
        binary,
    })
}

/// A `file`'s text as it is written: its `from` read, filled in for a template, which may name
/// the app's version only in an app that has one; and whether it is a `from` that isn't UTF-8
/// text, a picture's bytes, which the text holds as base64.
fn file_text(
    words: &Words,
    folder: &Path,
    from: Option<&str>,
    text: Option<&str>,
    template: bool,
) -> Result<(String, bool), ActionError> {
    let raw = match (from, text) {
        (Some(from), _) => {
            let path = folder.join(from);
            let bytes = fs::read(&path).map_err(|source| ActionError::ReadFrom {
                path: path.clone(),
                source,
            })?;
            match String::from_utf8(bytes) {
                Ok(text) => text,
                Err(_) if template => return Err(ActionError::BinaryTemplate { path }),
                Err(not_text) => return Ok((BASE64.encode(not_text.as_bytes()), true)),
            }
        }
        (None, Some(text)) => text.to_owned(),
        (None, None) => return Err(ActionError::FileSource),
    };
    let text = if template { words.fill(&raw, None) } else { raw };
    if template && text.contains("@VERSION@") {
        return Err(ActionError::NoVersion);
    }
    Ok((text, false))
}

/// Whether a `file` step's path, source and mode are well formed: one source, `from` a file of
/// the folder or `text`.
fn check_file(to: &str, from: Option<&str>, text: bool, mode: Option<&str>) -> Result<(), ActionError> {
    check_path(to)?;
    match (from, text) {
        (Some(from), false) if is_folder_file(from) => {}
        (Some(from), false) => return Err(ActionError::InvalidFrom { value: from.to_owned() }),
        (None, true) => {}
        _ => return Err(ActionError::FileSource),
    }
    if let Some(mode) = mode
        && parse_mode(mode).is_none()
    {
        return Err(ActionError::InvalidMode { value: mode.to_owned() });
    }
    Ok(())
}

/// Whether a `setting` step's file, section, key and value are well formed.
fn check_setting(file: &str, section: Option<&str>, key: &str, value: &toml::Value) -> Result<(), ActionError> {
    let home = file.starts_with("~/") && check_path(file).is_ok();
    let extension = Path::new(file).extension().and_then(|e| e.to_str());
    if !home || !matches!(extension, Some("json" | "toml" | "conf")) {
        return Err(ActionError::InvalidSettingFile { value: file.to_owned() });
    }
    if let Some(section) = section
        && (extension != Some("conf") || !settings::is_ini_name(section))
    {
        return Err(ActionError::InvalidSection {
            value: section.to_owned(),
        });
    }
    let fits = match (section, extension) {
        (Some(_), _) => settings::is_ini_name(key),
        // A flat `.conf`'s keys have no tables above them: a dot would read as one, and `=`, `#`
        // or a line break would end the key.
        (None, Some("conf")) => !key.is_empty() && !key.contains(['.', '=', '#', '\n', '\r']) && key.trim() == key,
        (None, _) => !key.split('.').any(str::is_empty),
    };
    if !fits {
        return Err(ActionError::InvalidKey { value: key.to_owned() });
    }
    if !matches!(
        value,
        toml::Value::String(_) | toml::Value::Integer(_) | toml::Value::Float(_) | toml::Value::Boolean(_)
    ) {
        return Err(ActionError::InvalidValue);
    }
    if section.is_some() && !settings::is_ini_value(value) {
        return Err(ActionError::InvalidIniValue);
    }
    Ok(())
}

/// Whether an `apt` step's packages, repository, architecture and versions are well formed.
fn check_apt(
    packages: &[String],
    repo: Option<&str>,
    arch: Option<&str>,
    versions: Option<&str>,
) -> Result<(), ActionError> {
    non_empty("packages", packages)?;
    if let Some(versions) = versions
        && (versions.is_empty() || versions.contains(char::is_whitespace))
    {
        return Err(ActionError::InvalidVersions {
            value: versions.to_owned(),
        });
    }
    if let Some(name) = packages.iter().find(|p| !is_apt_package(p)) {
        return Err(ActionError::InvalidPackage { name: name.clone() });
    }
    if let Some(repo) = repo
        && !is_app_id(repo)
    {
        return Err(ActionError::InvalidRepo { name: repo.to_owned() });
    }
    if let Some(arch) = arch {
        check_arch(arch)?;
    }
    Ok(())
}

/// Whether `name` is a package an `apt` step installs: a package name, or one with the
/// architecture dpkg installs it for when that isn't the machine's (`libgl1:i386`).
fn is_apt_package(name: &str) -> bool {
    match name.split_once(':') {
        Some((package, arch)) => is_package_name(package) && FOREIGN_ARCHES.contains(&arch),
        None => is_package_name(name),
    }
}

/// The architecture an `apt` step's package names, `libgl1:i386`'s `i386`; none for one of the
/// machine's own.
pub fn package_arch(name: &str) -> Option<&str> {
    name.split_once(':').map(|(_, arch)| arch)
}

/// A `run`'s argvs as a journal records them: `do`, `check` and `undo`, or `undo` alone, which
/// takes neither a window nor a source (`extra`).
fn check_run(run: &[String], check: &[String], undo: &[String], extra: bool) -> Result<(), ActionError> {
    if run.is_empty() != check.is_empty() {
        return Err(ActionError::RunHalf);
    }
    if run.is_empty() && extra {
        return Err(ActionError::RemovalOnlyExtra);
    }
    for (key, argv) in [("do", run), ("check", check), ("undo", undo)] {
        let absent = argv.is_empty() && key != "undo";
        if !absent && !is_resolved_argv(argv) {
            return Err(ActionError::InvalidArgv { key });
        }
    }
    Ok(())
}

fn check_web_app(id: &str, title: &str, url: &str, icon: &str) -> Result<(), ActionError> {
    if !is_app_id(id) {
        return Err(ActionError::InvalidWebApp { value: id.to_owned() });
    }
    if title.trim().is_empty() {
        return Err(ActionError::Empty { key: "title" });
    }
    if title.chars().any(char::is_control) {
        return Err(ActionError::InvalidTitle);
    }
    if !is_https(url) && !is_loopback_http(url) {
        return Err(ActionError::InvalidUrl { value: url.to_owned() });
    }
    if !is_svg_icon(icon) && !is_theme_icon(icon) {
        return Err(ActionError::InvalidIcon { value: icon.to_owned() });
    }
    Ok(())
}

/// Whether an `icon` names an `.svg` in the app's folder, which the install copies.
pub fn is_svg_icon(icon: &str) -> bool {
    is_folder_file(icon) && Path::new(icon).extension().is_some_and(|e| e == "svg")
}

/// Whether an `icon` is the icon theme's name for an icon, which a desktop entry or a menu names
/// as it is: an icon name with no extension the theme's lookup would read as a file's.
pub fn is_theme_icon(icon: &str) -> bool {
    is_icon_name(icon)
        && Path::new(icon)
            .extension()
            .is_none_or(|e| !matches!(e.to_str(), Some("svg" | "png" | "xpm" | "jpg" | "jpeg")))
}

/// Whether `value` can be an icon theme's name for an icon: no path, as the theme names them.
/// <https://specifications.freedesktop.org/icon-naming-spec/latest/>
pub fn is_icon_name(value: &str) -> bool {
    value.len() <= 128
        && value.chars().next().is_some_and(|c| c.is_ascii_alphanumeric())
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
}

/// Whether `url` is an https:// URL with no space or control character in it.
pub fn is_https(url: &str) -> bool {
    url.strip_prefix("https://")
        .is_some_and(|rest| !rest.is_empty() && !rest.chars().any(|c| c.is_whitespace() || c.is_control()))
}

/// Whether `url` is an http:// URL of this machine: `localhost`, `127.0.0.1` or `[::1]` with an
/// explicit port, as a service the user runs here (Penpot's local stack) listens on. Nothing but
/// this machine answers it, so it needs no TLS.
pub fn is_loopback_http(url: &str) -> bool {
    let Some(rest) = url.strip_prefix("http://") else {
        return false;
    };
    if rest.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return false;
    }
    let end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let authority = &rest[..end];
    let Some((host, port)) = authority.rsplit_once(':') else {
        return false;
    };
    let port = !port.is_empty() && port.chars().all(|c| c.is_ascii_digit()) && port.parse::<u16>().is_ok_and(|p| p > 0);
    port && matches!(host, "localhost" | "127.0.0.1" | "[::1]")
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

/// Whether `version` is one of `pattern`'s, where `*` stands for any run of characters and every
/// other character for itself.
pub fn version_matches(pattern: &str, version: &str) -> bool {
    let mut parts = pattern.split('*');
    let first = parts.next().unwrap_or_default();
    let Some(mut rest) = version.strip_prefix(first) else {
        return false;
    };
    let mut parts: Vec<&str> = parts.collect();
    let Some(last) = parts.pop() else {
        return rest.is_empty();
    };
    for part in parts {
        let Some(at) = rest.find(part) else {
            return false;
        };
        rest = &rest[at + part.len()..];
    }
    rest.len() >= last.len() && rest.ends_with(last)
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

/// Whether `argv` is a `run`'s argv as a journal records it: as [`is_argv`] says, but for a
/// program under the home, which is there by its path.
fn is_resolved_argv(argv: &[String]) -> bool {
    is_argv(argv)
        || argv
            .first()
            .and_then(|p| p.strip_prefix('/'))
            .is_some_and(is_folder_path)
}

fn is_folder_path(value: &str) -> bool {
    !value.is_empty()
        && !value
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
}

pub(crate) fn is_unit_name(value: &str) -> bool {
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
