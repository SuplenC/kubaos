//! Doing, checking and undoing one action on a [`Host`].
//!
//! A step first [`look`]s: whether the action is done already, what it will change and what it
//! finds there (its [`Claim`]s), and what the machine has that its undo needs (its [`Prior`]). The
//! journal records that before [`apply`] changes anything, and [`undo`] takes back only the owned
//! claims it is given, from the prior state recorded then, and leaves what was changed since.
//!
//! # Owns
//!
//! - Each kind's do, check and undo.
//!
//! # Not here
//!
//! - The journal and who owns what: `journal.rs`. The order of a run: `engine.rs`.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::fs::symlink;
use std::path::Path;
use std::path::PathBuf;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;

use super::Action;
use super::Kind;
use super::Scope;
use super::Unpack;
use super::dkms_source;
use super::parse_mode;
use super::settings;
use super::settings::Settings;
use super::settings::SettingsError;
use super::version_matches;
use crate::apps::host::AptPlan;
use crate::apps::host::AptSimulation;
use crate::apps::host::Compression;
use crate::apps::host::DkmsInstall;
use crate::apps::host::Downgrade;
use crate::apps::host::Host;
use crate::apps::host::HostError;
use crate::apps::host::Known;
use crate::apps::host::Removal;
use crate::apps::host::SecureBoot;
use crate::apps::host::UnitState;
use crate::apps::journal::Claim;
use crate::apps::journal::JournalError;
use crate::apps::journal::Prior;
use crate::apps::journal::write_new;
use crate::unfree::Unfree;

/// The mode a file gets without a `mode`.
const DEFAULT_FILE_MODE: u32 = 0o644;

/// The mode a single-file download gets: such a download is a program.
const DOWNLOAD_FILE_MODE: u32 = 0o755;

/// The mode a `run`'s script gets in the folder it runs from, which only its runner reads.
const SCRIPT_MODE: u32 = 0o700;

/// The permission bits no unpacked file keeps: no one but its owner may write it.
const WRITABLE_BY_OTHERS: u32 = 0o022;

/// An action the machine didn't take.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum PerformError {
    #[error(transparent)]
    Host(HostError),
    #[error("{}", path.display())]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("there is nothing to install for {arch}")]
    Unavailable { arch: String },
    #[error(
        "nixpkgs marks it unfree, and your nixpkgs refuses unfree packages: `kubaos package unfree on` lets them in"
    )]
    Unfree,
    #[error(transparent)]
    Settings(SettingsError),
    #[error("installing it would remove {}, which this app didn't install; nothing was changed", packages.join(", "))]
    AptRemoves { packages: Vec<String> },
    #[error(
        "installing it would move {} down to a lower version, which this app didn't ask for; nothing was changed",
        packages.join(", ")
    )]
    AptDowngrades { packages: Vec<String> },
    #[error("removing it would also remove {}, which this app didn't install; nothing was changed", packages.join(", "))]
    AptTakes { packages: Vec<String> },
    #[error("the journal's file isn't base64, so its bytes can't be written back")]
    NotBase64,
    #[error("recording what it is about to install")]
    Record(#[source] JournalError),
    #[error("no kernel has its headers to build {module} against: install linux-headers-generic")]
    NoKernelHeaders { module: String },
    #[error(
        "DKMS installed {module} for {} without signing it with the Machine Owner Key, so Secure Boot would refuse it: check mok_signing_key and mok_certificate in /etc/dkms/framework.conf",
        kernels.join(", ")
    )]
    Unsigned { module: String, kernels: Vec<String> },
    #[error("the source of {module} has no folder {folder}")]
    NoSourceFolder { module: String, folder: String },
}

impl PerformError {
    fn io(path: &Path) -> impl FnOnce(io::Error) -> Self + '_ {
        move |source| Self::Io {
            path: path.to_path_buf(),
            source,
        }
    }
}

/// What an undo knows beyond its step's own record.
#[derive(Clone, Copy, Debug, Default)]
pub struct Undoing<'a> {
    /// The packages the app's steps installed or pulled in, its `package:` and `dep:` claims as
    /// they name them (`libc6:i386`), but those handed to other apps: what an `architecture`
    /// step's undo may take with the architecture.
    pub brought: &'a [String],
    /// A changed action of the same thing replaces the step's, and stands: the module a `dkms`
    /// step loaded may be the new version's, which stays loaded.
    pub replacing: bool,
}

/// What a step finds before it changes anything.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Look {
    /// The action is done already.
    pub done: bool,
    /// What it changes (owned) and what it finds there (not owned).
    pub claims: Vec<Claim>,
    pub prior: Prior,
}

/// What a step's journal entry says its path holds, which a download's check can't read from
/// the path itself: a folder or a program unpacked from a tarball carries no sha256.
#[derive(Clone, Copy, Debug)]
pub enum Recorded<'a> {
    /// No entry of the app owns the path: the machine alone says whether the step is done.
    Nothing,
    /// The app's step put its path there, as this action did it.
    Done(&'a Action),
    /// The app's step was stopped while this action changed its path: what it holds is unknown.
    Unsettled(&'a Action),
    /// The app's `run` that only undoes, recorded as found: the app is there by other means, and
    /// the step owns nothing.
    Found,
}

/// Whether the download `had` put there is the one `want` pins for `arch`: the same bytes,
/// unpacked the same way.
fn same_download(had: &Action, want: &Action, arch: &str) -> bool {
    let pin = |action: &Action| match &action.kind {
        Kind::Download { unpack, strip, .. } => {
            Some((action.download(arch).map(|d| d.sha256.clone()), *unpack, *strip))
        }
        _ => None,
    };
    pin(had).is_some() && pin(had) == pin(want)
}

/// Whether `path`, a download's place in the folder named `version`, is there in a folder of a
/// newer version beside it: where a vendor's updater put the version it moved on to, after
/// removing the one the pin downloaded.
fn newer_version_there(host: &dyn Host, path: &Path, version: &str) -> bool {
    let Some(folder) = path
        .ancestors()
        .find(|dir| dir.file_name().is_some_and(|n| n == version))
    else {
        return false;
    };
    let (Some(versions), Ok(rest)) = (folder.parent(), path.strip_prefix(folder)) else {
        return false;
    };
    let Ok(entries) = fs::read_dir(host.place(versions)) else {
        return false;
    };
    entries.flatten().any(|entry| {
        entry
            .file_name()
            .to_str()
            .is_some_and(|name| newer_version(name, version))
            && host
                .place(&versions.join(entry.file_name()).join(rest))
                .symlink_metadata()
                .is_ok()
    })
}

/// Whether `a` is a version newer than `b`, both dot-separated numbers; a name that isn't one is
/// no version.
fn newer_version(a: &str, b: &str) -> bool {
    let parts = |v: &str| -> Option<Vec<u64>> { v.split('.').map(|part| part.parse().ok()).collect() };
    match (parts(a), parts(b)) {
        (Some(a), Some(b)) => a > b,
        _ => false,
    }
}

/// Where the path `named` (`~/…` or absolute) is, as the actions name it.
pub fn logical(host: &dyn Host, named: &str) -> PathBuf {
    match named.strip_prefix("~/") {
        Some(rest) => host.home().join(rest),
        None => PathBuf::from(named),
    }
}

/// A web app's desktop entry and copied icon, as an action names paths.
pub fn web_app_paths(id: &str) -> [String; 2] {
    [
        format!("~/.local/share/applications/kubaos-web-{id}.desktop"),
        format!("~/.local/share/icons/hicolor/scalable/apps/kubaos-web-{id}.svg"),
    ]
}

/// The browser's folder of a web app, its own `--user-data-dir`: its window's sign-ins, cookies
/// and cache, apart from the user's browser. The user's data, which the browser makes: no step
/// claims it, so a removal keeps it and a reinstall opens it again.
pub fn web_app_profile(id: &str) -> String {
    format!("~/.local/share/kubaos/web-apps/{id}")
}

/// The profile a web app's window opens in, in its own browser folder: it names the window.
fn web_app_profile_name(id: &str) -> String {
    format!("kubaos-web-{id}")
}

/// A web app's desktop entry: `url` in an app window of the browser, in a browser folder of its
/// own at `profile`, its window's app id what the browser names it, and `icon` its icon: a path, or
/// the icon theme's name for one.
///
/// Chrome names an `--app` window on Wayland after the URL and the profile, never `--class`,
/// which sets only X11's class: `chrome-<host>_<path>-<profile>`, as the file name of the
/// desktop entry it would write for the app. Its own `--user-data-dir` makes it a browser of its
/// own, so the flags reach it whether or not the user's Chrome is open, and its own
/// `--profile-directory` makes the name the web app's alone. `StartupWMClass` is that name, so a
/// launcher or a dock matches the window to the entry.
/// <https://github.com/chromium/chromium/blob/157.0.8103.78/chrome/browser/ui/views/frame/browser_native_widget_aura_linux.cc>
/// <https://github.com/chromium/chromium/blob/157.0.8103.78/chrome/browser/web_applications/os_integration/web_app_shortcut_linux.cc>
fn web_app_entry(id: &str, title: &str, url: &str, icon: &str, profile: &Path) -> String {
    let name = web_app_profile_name(id);
    let exec = [
        super::WEB_APP_BROWSER.to_owned(),
        format!("--user-data-dir={}", profile.display()),
        format!("--profile-directory={name}"),
        "--no-first-run".to_owned(),
        "--no-default-browser-check".to_owned(),
        format!("--app={url}"),
    ]
    .map(|arg| exec_arg(&arg))
    .join(" ");
    format!(
        "[Desktop Entry]\nType=Application\nName={title}\nExec={exec}\nIcon={icon}\nStartupWMClass={class}\n\
         Terminal=false\n",
        title = entry_value(title),
        icon = entry_value(icon),
        class = entry_value(&web_app_window(url, &name)),
    )
}

/// The app id Chrome gives the `--app` window of `url` in the profile `profile`: `chrome-`, the
/// URL's host, `_` and its path (`/` for none), `-` and the profile, with each character a file
/// name can't have (`"*/:<>?\|`, a control character) and each space made `_`.
pub fn web_app_window(url: &str, profile: &str) -> String {
    let rest = url.split_once("://").map_or(url, |(_, rest)| rest);
    let end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let (authority, tail) = rest.split_at(end);
    let host = authority.rsplit_once('@').map_or(authority, |(_, host)| host);
    let host = match host.strip_prefix('[') {
        Some(v6) => format!("[{}", v6.split_once(']').map_or(v6, |(address, _)| address)) + "]",
        None => host.split_once(':').map_or(host, |(name, _)| name).to_owned(),
    };
    let path = &tail[..tail.find(['?', '#']).unwrap_or(tail.len())];
    let path = if path.is_empty() { "/" } else { path };
    format!("chrome-{}_{path}-{profile}", host.to_ascii_lowercase())
        .chars()
        .map(|c| {
            if c.is_control() || c == ' ' || matches!(c, '"' | '*' | '/' | ':' | '<' | '>' | '?' | '\\' | '|') {
                '_'
            } else {
                c
            }
        })
        .collect()
}

/// The characters the Desktop Entry spec reserves in an `Exec` argument, which quote it.
///
/// <https://specifications.freedesktop.org/desktop-entry-spec/latest/exec-variables.html>
const EXEC_RESERVED: &[char] = &[
    ' ', '\t', '\n', '"', '\'', '\\', '>', '<', '~', '|', '&', ';', '$', '*', '?', '#', '(', ')', '`',
];

/// One argument of an `Exec` line, as the Desktop Entry spec has it: a `%` doubled, so no field
/// code expands; an argument with a reserved character in double quotes, its `"`, `` ` ``, `$`
/// and `\` escaped; then escaped once more as a string value, whose own escape for `\` is
/// applied before the quoting is read.
fn exec_arg(arg: &str) -> String {
    let arg = arg.replace('%', "%%");
    let quoted = if arg.contains(EXEC_RESERVED) {
        let mut out = String::from("\"");
        for c in arg.chars() {
            if matches!(c, '"' | '`' | '$' | '\\') {
                out.push('\\');
            }
            out.push(c);
        }
        out.push('"');
        out
    } else {
        arg
    };
    entry_value(&quoted)
}

/// A string value of a desktop entry: its `\` escaped; a value is one line.
fn entry_value(text: &str) -> String {
    text.replace('\\', "\\\\")
}

/// A web app's files, each with its path and its text: its desktop entry, and the icon an `.svg`
/// of its folder is copied to. A theme's icon, which the entry names, copies nothing.
fn web_app_files(host: &dyn Host, action: &Action) -> Vec<(PathBuf, String)> {
    let Kind::WebApp {
        id,
        title,
        url,
        icon,
        svg,
    } = &action.kind
    else {
        return Vec::new();
    };
    let [entry, copied] = web_app_paths(id).map(|p| logical(host, &p));
    let profile = logical(host, &web_app_profile(id));
    match svg {
        Some(svg) => {
            let text = web_app_entry(id, title, url, &copied.to_string_lossy(), &profile);
            vec![(entry, text), (copied, svg.clone())]
        }
        None => vec![(entry, web_app_entry(id, title, url, icon, &profile))],
    }
}

/// Whether a unit stands as `start` asks: started (running, or run and exited cleanly, as a
/// oneshot does), or not running.
fn started_as(state: UnitState, start: bool) -> bool {
    if start { state.started() } else { !state.active }
}

/// What a `nix` action finds: the names `packages.toml` lists. One for another architecture, or
/// an unfree one the user's nixpkgs refuses, is refused before anything changes: its name would
/// fail the switch after the run, for the whole home. One already listed is in a home that built.
fn look_nix(host: &mut dyn Host, packages: &[String], arch: Option<&str>, unfree: bool) -> Result<Look, PerformError> {
    if let Some(only) = arch
        && only != host.arch()
    {
        return Err(PerformError::Unavailable {
            arch: host.arch().to_owned(),
        });
    }
    let listed = host.nix_list().map_err(PerformError::Host)?;
    let look = parts("nix", packages, |p| listed.contains(p));
    if unfree && !look.done && host.unfree().map_err(PerformError::Host)? == Unfree::Refused {
        return Err(PerformError::Unfree);
    }
    Ok(look)
}

/// What a unit, a group or lingering finds.
fn look_machine(action: &Action, host: &mut dyn Host, user: &str) -> Result<Look, PerformError> {
    match &action.kind {
        Kind::Unit {
            unit,
            scope,
            enable,
            start,
        } => {
            let state = host.unit_state(*scope, unit).map_err(PerformError::Host)?;
            let done = enable.is_none_or(|e| state.enabled == e) && start.is_none_or(|s| started_as(state, s));
            Ok(Look {
                done,
                claims: vec![Claim::new(format!("unit:{}:{unit}", scope.word()), !done)],
                prior: Prior {
                    enabled: Some(state.enabled),
                    active: Some(state.started()),
                    ..Prior::default()
                },
            })
        }
        Kind::Group { group } => {
            let exists = host.group_exists(group).map_err(PerformError::Host)?;
            let member = exists && host.member(group).map_err(PerformError::Host)?;
            Ok(Look {
                done: member,
                claims: vec![
                    Claim::new(format!("group:{group}"), !exists),
                    Claim::new(format!("member:{group}:{user}"), !member),
                ],
                prior: Prior::default(),
            })
        }
        Kind::Linger {} => {
            let on = host.lingers().map_err(PerformError::Host)?;
            Ok(Look {
                done: on,
                claims: vec![Claim::new(format!("linger:{user}"), !on)],
                prior: Prior {
                    lingered: Some(on),
                    ..Prior::default()
                },
            })
        }
        _ => Ok(Look::default()),
    }
}

/// Whether what `action` puts on the machine is there, by `look`, its look: when it is done, and
/// for an `apt` action whose packages are all installed but whose repository isn't set up yet,
/// which [`apply`] then sets up without installing anything.
#[must_use]
pub fn there(action: &Action, look: &Look) -> bool {
    look.done || (matches!(action.kind, Kind::Apt { .. }) && look.claims.iter().all(|c| !c.owned))
}

/// What `action` finds on the machine, whose step's journal entry says `recorded` of its path.
/// A download is done when its path is there and holds what the action pins: a release that
/// changes the pin of a download at the same path downloads it again.
///
/// # Errors
///
/// [`PerformError`] when the machine can't be read, a setting's file isn't settings it can
/// change, or a `run`'s check can't be started.
pub fn look(action: &Action, host: &mut dyn Host, recorded: Recorded<'_>) -> Result<Look, PerformError> {
    let user = host.user().to_owned();
    match &action.kind {
        Kind::Apt {
            packages,
            arch,
            versions,
            repo,
            ..
        } => {
            if let Some(only) = arch
                && only != host.arch()
            {
                return Err(PerformError::Unavailable {
                    arch: host.arch().to_owned(),
                });
            }
            let ours = installed_as(host, packages, versions.as_deref())?;
            let mut look = parts("package", packages, |p| ours.contains(p));
            // Packages a vendor's own setup brought (a hand-installed Slack, its key trusted for
            // every repository) aren't the app set up as KubaOS sets it up until its repository is.
            if let Some(repo) = repo
                && look.done
                && !host.apt_repo_ready(repo).map_err(PerformError::Host)?
            {
                look.done = false;
            }
            Ok(look)
        }
        Kind::Deb {
            package,
            version,
            self_updates,
            ..
        } => {
            let have = host.version(package).map_err(PerformError::Host)?;
            let done = have.as_ref().is_some_and(|v| *self_updates || v == version);
            Ok(Look {
                done,
                claims: vec![Claim::new(format!("package:{package}"), have.is_none())],
                prior: Prior::default(),
            })
        }
        Kind::Nix { packages, arch, unfree } => look_nix(host, packages, arch.as_deref(), *unfree),
        Kind::Download {
            to,
            version,
            self_updates,
            in_place,
            ..
        } => {
            let path = logical(host, to);
            let there = host.place(&path).symlink_metadata().is_ok();
            let holds = match recorded {
                Recorded::Done(had) if had.path() == action.path() => same_download(had, action, host.arch()),
                Recorded::Unsettled(had) => had.path() != action.path(),
                Recorded::Done(_) | Recorded::Nothing | Recorded::Found => true,
            };
            let moved_on = *self_updates && !*in_place && !there && newer_version_there(host, &path, version);
            // A `to` naming no `@VERSION@` is the vendor's updater's to rewrite in place once it is
            // there: a new pin would put an older build back over its newer one.
            let updated_in_place = *in_place && there && !matches!(recorded, Recorded::Unsettled(_));
            Ok(at_path(host, &path, (there && holds) || moved_on || updated_in_place))
        }
        Kind::File { .. } => Ok(look_file(host, &action.kind)),
        Kind::WebApp { .. } => {
            let files = web_app_files(host, action);
            let done = files
                .iter()
                .all(|(path, text)| has_file(host, path, text.as_bytes(), DEFAULT_FILE_MODE));
            let mut look = Look {
                done,
                ..Look::default()
            };
            // The browser folder is left out: it holds the user's sign-ins, which outlast the app.
            for (path, _) in &files {
                for claim in at_path(host, path, done).claims {
                    if !look.claims.contains(&claim) {
                        look.claims.push(claim);
                    }
                }
            }
            Ok(look)
        }
        Kind::Symlink { to, target } => {
            let path = logical(host, to);
            let done = fs::read_link(host.place(&path)).is_ok_and(|t| t == Path::new(target));
            Ok(at_path(host, &path, done))
        }
        Kind::Output { path } => {
            let path = logical(host, path);
            let there = host.place(&path).symlink_metadata().is_ok();
            Ok(at_path(host, &path, there))
        }
        Kind::Setting {
            file,
            section,
            key,
            value,
        } => look_setting(host, &Place::new(host, file, section.as_deref()), key, value),
        Kind::Unit { .. } | Kind::Group { .. } | Kind::Linger {} => look_machine(action, host, &user),
        Kind::Architecture { architecture } => look_architecture(host, architecture),
        Kind::Dkms { .. } => look_dkms(action, host, recorded),
        Kind::Run { .. } => look_run(action, host, recorded),
    }
}

/// What an `architecture` step finds: dpkg taking packages of `arch`. The machine's own is dpkg's
/// always: there, and never the app's.
fn look_architecture(host: &mut dyn Host, arch: &str) -> Result<Look, PerformError> {
    let there = arch == host.arch() || host.foreign_arches().map_err(PerformError::Host)?.contains(arch);
    Ok(Look {
        done: there,
        claims: vec![Claim::new(format!("arch:{arch}"), !there)],
        ..Look::default()
    })
}

/// How far DKMS has a module at a version: in its tree, and installed, signed with the MOK when
/// there is one, for every kernel with its headers.
struct Built {
    added: bool,
    /// The kernels with their headers that lack it, or have it unsigned while DKMS has a MOK.
    missing: Vec<String>,
    /// No kernel has its headers.
    no_kernel: bool,
    /// The kernels with their headers.
    kernels: Vec<String>,
}

impl Built {
    fn whole(&self) -> bool {
        self.added && !self.no_kernel && self.missing.is_empty()
    }
}

/// What DKMS has of `module` at `version`, as [`Built`] says.
fn dkms_built(host: &mut dyn Host, module: &str, version: &str) -> Result<Built, PerformError> {
    let status = host.dkms_status(module, version).map_err(PerformError::Host)?;
    let kernels = host.kernels().map_err(PerformError::Host)?;
    let mut missing = Vec::new();
    for kernel in &kernels {
        let mok = host.mok(kernel).map_err(PerformError::Host)?;
        let signed = match &mok {
            Some(mok) => host
                .module_signer(module, kernel)
                .map_err(PerformError::Host)?
                .is_some_and(|key| same_key(&key, &mok.serial)),
            None => true,
        };
        if !status.installed.contains(kernel) || !signed {
            missing.push(kernel.clone());
        }
    }
    Ok(Built {
        added: status.added,
        missing,
        no_kernel: kernels.is_empty(),
        kernels: kernels.into_iter().collect(),
    })
}

/// Whether two key ids, as `modinfo` and `openssl` print a serial, are the same: case, `:` and a
/// leading zero byte aside, as a module's signature keeps the serial's sign byte.
fn same_key(a: &str, b: &str) -> bool {
    let bare = |key: &str| {
        let hex: String = key
            .chars()
            .filter(|c| *c != ':')
            .collect::<String>()
            .to_ascii_uppercase();
        hex.trim_start_matches('0').to_owned()
    };
    bare(a) == bare(b)
}

/// Whether the source a `dkms` step's journal entry `had` put in place is the one `want` pins:
/// the same download, unpacked the same way, with the same `dkms.conf`.
fn same_source(had: &Action, want: &Action) -> bool {
    let pin = |action: &Action| match &action.kind {
        Kind::Dkms {
            version,
            url,
            sha256,
            strip,
            folder,
            conf_text,
            ..
        } => Some((
            version.clone(),
            url.clone(),
            sha256.clone(),
            *strip,
            folder.clone(),
            conf_text.clone(),
        )),
        _ => None,
    };
    pin(had).is_some() && pin(had) == pin(want)
}

/// What a `dkms` step finds: its source in `/usr/src/<module>-<version>`, from its pin when the
/// journal says its step put it there, and the module in DKMS's tree and installed, signed with
/// the MOK, for every kernel with its headers.
fn look_dkms(action: &Action, host: &mut dyn Host, recorded: Recorded<'_>) -> Result<Look, PerformError> {
    let Kind::Dkms { module, version, .. } = &action.kind else {
        return Ok(Look::default());
    };
    let path = PathBuf::from(dkms_source(module, version));
    let there = host.place(&path).join("dkms.conf").is_file();
    let holds = match recorded {
        Recorded::Done(had) => same_source(had, action),
        Recorded::Unsettled(_) => false,
        Recorded::Nothing | Recorded::Found => true,
    };
    let built = dkms_built(host, module, version)?;
    let mut look = at_path(host, &path, there && holds);
    look.claims
        .push(Claim::new(format!("dkms:{module}/{version}"), !built.added));
    look.done = there && holds && built.whole();
    Ok(look)
}

/// What the user has to know of `action`, a step that stands, said whenever a run reaches it:
/// for a `dkms` step, what to do at the next boot while Secure Boot won't load what the MOK signs.
///
/// # Errors
///
/// [`PerformError`] when the machine can't be asked.
pub fn says(action: &Action, host: &mut dyn Host) -> Result<Option<String>, PerformError> {
    match &action.kind {
        Kind::Dkms { module, .. } => mok_note(host, module),
        _ => Ok(None),
    }
}

/// What the user must do for Secure Boot to load `module`, which DKMS signed with the MOK: none
/// while Secure Boot is off or takes the MOK. The boot screen that enrols it is the user's.
fn mok_note(host: &mut dyn Host, module: &str) -> Result<Option<String>, PerformError> {
    // The newest kernel's, which the machine boots next.
    let mok = match host.kernels().map_err(PerformError::Host)?.last() {
        Some(kernel) => host.mok(kernel).map_err(PerformError::Host)?,
        None => None,
    };
    let certificate = mok.as_ref().map(|m| m.certificate.display().to_string());
    let state = host
        .secure_boot(mok.as_ref().map(|m| m.certificate.as_path()))
        .map_err(PerformError::Host)?;
    Ok(match (state, certificate) {
        (SecureBoot::Off | SecureBoot::Enrolled, _) => None,
        (SecureBoot::Queued, _) => Some(format!(
            "Secure Boot is on: at the next boot, pick Enroll MOK on the blue MokManager screen and type the one-time password you chose, or {module} won't load."
        )),
        (SecureBoot::NotEnrolled, Some(certificate)) => Some(format!(
            "Secure Boot is on, and the key DKMS signed {module} with isn't enrolled, so it won't load: run `sudo mokutil --import {certificate}` and pick a one-time password, then at the next boot pick Enroll MOK on the blue MokManager screen and type it."
        )),
        (_, Some(certificate)) => Some(format!(
            "Secure Boot is on, and it couldn't tell whether it takes the key DKMS signed {module} with: `sudo mokutil --test-key {certificate}` says."
        )),
        (_, None) => Some(format!(
            "Secure Boot is on, and DKMS has no key to sign {module} with, so it won't load."
        )),
    })
}

/// Does a `dkms` step: its source put in place when `look` found it missing or another pin's, the
/// module added to DKMS's tree, then built and installed for each kernel with its headers that
/// lacks it, or has it unsigned while DKMS has a MOK, and for every one when the source is new, as
/// a kernel's build is of the source that was there. It fails when a kernel still lacks it signed,
/// but for one its `dkms.conf` leaves out, which the note names.
fn apply_dkms(action: &Action, host: &mut dyn Host, look: &Look) -> Result<Option<String>, PerformError> {
    let Kind::Dkms { module, version, .. } = &action.kind else {
        return Ok(None);
    };
    let placed = !owned_names(&look.claims, "path:").is_empty();
    if placed {
        place_dkms_source(action, host)?;
    }
    if !host.dkms_status(module, version).map_err(PerformError::Host)?.added {
        host.dkms_add(module, version).map_err(PerformError::Host)?;
    }
    let built = dkms_built(host, module, version)?;
    if built.no_kernel {
        return Err(PerformError::NoKernelHeaders { module: module.clone() });
    }
    let kernels = if placed { &built.kernels } else { &built.missing };
    let mut excluded = Vec::new();
    for kernel in kernels {
        if host.dkms_install(module, version, kernel).map_err(PerformError::Host)? == DkmsInstall::Excluded {
            excluded.push(kernel.clone());
        }
    }
    let built = dkms_built(host, module, version)?;
    let missing: Vec<String> = built.missing.into_iter().filter(|k| !excluded.contains(k)).collect();
    if !missing.is_empty() {
        return Err(PerformError::Unsigned {
            module: module.clone(),
            kernels: missing,
        });
    }
    Ok(match excluded.as_slice() {
        [] => None,
        [kernel] => Some(format!(
            "{module} isn't built for {kernel}: its dkms.conf leaves that kernel out."
        )),
        _ => Some(format!(
            "{module} isn't built for {}: its dkms.conf leaves those kernels out.",
            excluded.join(", ")
        )),
    })
}

/// Puts a `dkms` step's source in `/usr/src/<module>-<version>`: the pinned tarball downloaded
/// and checked, unpacked into a staging folder beside it, its `folder` taken, `dkms.conf` written
/// into it, no one but its owner able to write it, then moved into place whole, over one there.
fn place_dkms_source(action: &Action, host: &mut dyn Host) -> Result<(), PerformError> {
    let Kind::Dkms {
        module,
        version,
        url,
        sha256,
        strip,
        folder,
        conf_text,
        ..
    } = &action.kind
    else {
        return Ok(());
    };
    let path = host.place(&PathBuf::from(dkms_source(module, version)));
    let stage = staging(&path);
    let file = stage.with_extension("download");
    remove_path(&stage)?;
    make_parents(&path)?;
    let result = (|| {
        write_new(&file, b"").map_err(PerformError::io(&file))?;
        host.fetch(url, sha256, &file).map_err(PerformError::Host)?;
        fs::create_dir(&stage).map_err(PerformError::io(&stage))?;
        host.untar(&file, &stage, Compression::Gzip, *strip)
            .map_err(PerformError::Host)?;
        let tree = match folder {
            Some(folder) => stage.join(folder),
            None => stage.clone(),
        };
        if !tree.is_dir() {
            return Err(PerformError::NoSourceFolder {
                module: module.clone(),
                folder: folder.clone().unwrap_or_default(),
            });
        }
        write_file(
            &tree.join("dkms.conf"),
            conf_text.as_deref().unwrap_or_default(),
            DEFAULT_FILE_MODE,
        )?;
        only_owner_writes(&tree)?;
        replace_with(&tree, &path)
    })();
    let _ = fs::remove_file(&file);
    let _ = remove_path(&stage);
    result
}

/// Undoes a `dkms` step from its owned `claims`: with the module's in DKMS's tree, the module
/// unloaded when it is loaded and nothing uses it (nothing holds it and no device is bound to its
/// drivers), then taken out of DKMS's tree and every kernel; then its source and the folders it
/// made. A module in use stays loaded until the machine restarts, which the note says; one a
/// version bump is `replacing` is never unloaded, as the new version's build is the one it uses.
fn undo_dkms(
    host: &mut dyn Host,
    module: &str,
    version: &str,
    claims: &[Claim],
    replacing: bool,
) -> Result<Option<String>, PerformError> {
    let mut note = None;
    if !owned_names(claims, "dkms:").is_empty() {
        let users = if replacing {
            None
        } else {
            host.module_users(module).map_err(PerformError::Host)?
        };
        match users {
            Some(0) => host.unload(module).map_err(PerformError::Host)?,
            Some(_) => {
                note = Some(format!(
                    "{module} stays loaded until the machine restarts, as something uses it now."
                ));
            }
            None => {}
        }
        if host.dkms_status(module, version).map_err(PerformError::Host)?.added {
            host.dkms_remove(module, version).map_err(PerformError::Host)?;
        }
    }
    for path in owned_names(claims, "path:") {
        remove_path(&host.place(Path::new(&path)))?;
    }
    remove_dirs(host, owned_names(claims, "dir:"));
    Ok(note)
}

/// A `run`'s check: its `check` exits 0; for one with `undo` alone, its own settled entry. Recorded
/// done, the step is the app's; recorded found, the app is there by other means, and the step
/// owns nothing, as a found app's steps own nothing; not recorded, it is the app's once recorded.
fn look_run(action: &Action, host: &mut dyn Host, recorded: Recorded<'_>) -> Result<Look, PerformError> {
    let Kind::Run { check, .. } = &action.kind else {
        return Ok(Look::default());
    };
    if action.removal_only() {
        let found = matches!(recorded, Recorded::Found);
        return Ok(Look {
            done: found || matches!(recorded, Recorded::Done(_)),
            claims: vec![Claim::new("run", !found)],
            prior: Prior::default(),
        });
    }
    let done = ran(run_recorded(host, action, check)?)?;
    Ok(Look {
        done,
        claims: vec![Claim::new("run", !done)],
        prior: Prior::default(),
    })
}

/// Which of `packages` dpkg has, at a version of `versions` when it is given: a package of
/// another version is another build of the name, not the one the action installs.
fn installed_as(
    host: &mut dyn Host,
    packages: &[String],
    versions: Option<&str>,
) -> Result<BTreeSet<String>, PerformError> {
    let mut installed = host.installed(packages).map_err(PerformError::Host)?;
    if let Some(versions) = versions {
        let mut other = Vec::new();
        for package in &installed {
            let version = host.version(package).map_err(PerformError::Host)?;
            if !version.is_some_and(|v| version_matches(versions, &v)) {
                other.push(package.clone());
            }
        }
        installed.retain(|p| !other.contains(p));
    }
    Ok(installed)
}

/// Whether `path` is a file holding `contents` with `mode`.
fn has_file(host: &dyn Host, path: &Path, contents: &[u8], mode: u32) -> bool {
    let placed = host.place(path);
    placed.symlink_metadata().is_ok_and(|m| m.is_file())
        && fs::read(&placed).is_ok_and(|t| t == contents)
        && fs::metadata(&placed).is_ok_and(|m| m.permissions().mode() & 0o777 == mode)
}

/// Does `action`, which `look`, just taken, found not done, and returns what the user has to know
/// of what it did. Before apt installs anything,
/// `record` is given the claims of the packages it would newly install beyond the action's own,
/// `dep:<name>`, so the journal holds them before they are there.
///
/// # Errors
///
/// [`PerformError`] for the change the machine refused, one apt would make beyond the action's
/// own packages, or claims `record` couldn't keep.
pub fn apply(
    action: &Action,
    host: &mut dyn Host,
    look: &Look,
    record: &mut dyn FnMut(&[Claim]) -> Result<(), JournalError>,
) -> Result<Option<String>, PerformError> {
    let owned = |prefix: &str| owned_names(&look.claims, prefix);
    if let Kind::Dkms { .. } = &action.kind {
        return apply_dkms(action, host, look);
    }
    let applied = match &action.kind {
        Kind::Apt {
            repo,
            packages,
            versions,
            ..
        } => apply_apt(
            host,
            repo.as_deref(),
            packages,
            versions.as_deref(),
            &owned("package:"),
            record,
        ),
        Kind::Deb { package, .. } => apply_deb(action, host, package, record),
        Kind::Nix { .. } => host.nix_add(&owned("nix:")).map_err(PerformError::Host),
        Kind::Download { to, unpack, strip, .. } => {
            let download = action.download(host.arch()).ok_or_else(|| PerformError::Unavailable {
                arch: host.arch().to_owned(),
            })?;
            let path = host.place(&logical(host, to));
            make_parents(&path)?;
            download_to(host, &download.url, &download.sha256, &path, *unpack, *strip)
        }
        Kind::File {
            to, text, mode, binary, ..
        } => {
            let logical_path = logical(host, to);
            let path = host.place(&logical_path);
            let contents = file_contents(text.as_deref(), *binary).ok_or(PerformError::NotBase64)?;
            make_parents(&path)?;
            let mode = mode.as_deref().and_then(parse_mode).unwrap_or(DEFAULT_FILE_MODE);
            write_bytes(&path, &contents, mode)?;
            reload_unit_file(host, &logical_path)
        }
        Kind::WebApp { .. } => {
            for (path, text) in web_app_files(host, action) {
                let path = host.place(&path);
                make_parents(&path)?;
                write_file(&path, &text, DEFAULT_FILE_MODE)?;
            }
            Ok(())
        }
        Kind::Symlink { to, target } => {
            let path = host.place(&logical(host, to));
            make_parents(&path)?;
            if path.symlink_metadata().is_ok() {
                fs::remove_file(&path).map_err(PerformError::io(&path))?;
            }
            symlink(target, &path).map_err(PerformError::io(&path))
        }
        Kind::Output { path } => make_parents(&host.place(&logical(host, path))),
        Kind::Setting {
            file,
            section,
            key,
            value,
        } => apply_setting(&Place::new(host, file, section.as_deref()), key, value),
        Kind::Unit {
            unit,
            scope,
            enable,
            start,
        } => {
            host.daemon_reload(*scope).map_err(PerformError::Host)?;
            let state = host.unit_state(*scope, unit).map_err(PerformError::Host)?;
            if let Some(enable) = *enable
                && state.enabled != enable
            {
                host.set_enabled(*scope, unit, enable).map_err(PerformError::Host)?;
            }
            if let Some(start) = *start
                && !started_as(state, start)
            {
                host.set_active(*scope, unit, start).map_err(PerformError::Host)?;
            }
            Ok(())
        }
        Kind::Group { group } => {
            if !host.group_exists(group).map_err(PerformError::Host)? {
                host.group_create(group).map_err(PerformError::Host)?;
            }
            host.set_member(group, true).map_err(PerformError::Host)
        }
        Kind::Linger {} => host.set_linger(true).map_err(PerformError::Host),
        Kind::Architecture { architecture } => add_architecture(host, architecture),
        Kind::Dkms { .. } => Ok(()),
        Kind::Run { .. } if action.removal_only() => Ok(()),
        Kind::Run { run, .. } => run_recorded(host, action, run)?.map_err(PerformError::Host),
    };
    applied.map(|()| None)
}

/// Installs a `deb` step's pinned `.deb` of `package`, downloaded and checked, its dependencies
/// recorded first as `apt`'s are.
fn apply_deb(
    action: &Action,
    host: &mut dyn Host,
    package: &str,
    record: &mut dyn FnMut(&[Claim]) -> Result<(), JournalError>,
) -> Result<(), PerformError> {
    let download = action.download(host.arch()).ok_or_else(|| PerformError::Unavailable {
        arch: host.arch().to_owned(),
    })?;
    let dir = tempfile::Builder::new()
        .prefix("kubaos-apps-")
        .tempdir()
        .map_err(PerformError::io(Path::new("a temporary folder")))?;
    // apt reads a local .deb as its own _apt user.
    fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o755)).map_err(PerformError::io(dir.path()))?;
    let deb = dir.path().join("app.deb");
    host.fetch(&download.url, &download.sha256, &deb)
        .map_err(PerformError::Host)?;
    fs::set_permissions(&deb, fs::Permissions::from_mode(0o644)).map_err(PerformError::io(&deb))?;
    let simulated = host.apt_simulate(AptPlan::Deb(&deb)).map_err(PerformError::Host)?;
    record_dependencies(simulated, &[package.to_owned()], record)?;
    host.deb_install(&deb).map_err(PerformError::Host)
}

/// An `apt` action's install of `missing`, its packages `look` found not there, from `repo`
/// when it names one. A package of `missing` that dpkg has is, under `versions`, another build of
/// the name (Ubuntu's `1:1snap1` firefox), which may sort above the app's own: only then may apt
/// move it down.
fn apply_apt(
    host: &mut dyn Host,
    repo: Option<&str>,
    packages: &[String],
    versions: Option<&str>,
    missing: &[String],
    record: &mut dyn FnMut(&[Claim]) -> Result<(), JournalError>,
) -> Result<(), PerformError> {
    if let Some(repo) = repo {
        host.apt_repo(repo).map_err(PerformError::Host)?;
    }
    if missing.is_empty() {
        return Ok(());
    }
    let simulated = host
        .apt_simulate(AptPlan::Install(missing))
        .map_err(PerformError::Host)?;
    record_dependencies(simulated, packages, record)?;
    let replaces = versions.is_some() && !host.installed(missing).map_err(PerformError::Host)?.is_empty();
    let downgrade = if replaces { Downgrade::Allow } else { Downgrade::Refuse };
    host.apt_install(missing, downgrade).map_err(PerformError::Host)
}

/// Refuses a change apt would make by removing other packages, or by moving down a package other
/// than `own`, the action's packages (one of which an `apt` action's `versions` lets apt move down,
/// to replace another build of its name); else hands `record` the claims of what it would newly
/// install beyond `own`.
fn record_dependencies(
    simulated: AptSimulation,
    own: &[String],
    record: &mut dyn FnMut(&[Claim]) -> Result<(), JournalError>,
) -> Result<(), PerformError> {
    if !simulated.removes.is_empty() {
        return Err(PerformError::AptRemoves {
            packages: simulated.removes.into_iter().collect(),
        });
    }
    let downgrades: Vec<String> = simulated.downgrades.into_iter().filter(|p| !own.contains(p)).collect();
    if !downgrades.is_empty() {
        return Err(PerformError::AptDowngrades { packages: downgrades });
    }
    let dependencies: Vec<Claim> = simulated
        .installs
        .into_iter()
        .filter(|p| !own.contains(p))
        .map(|p| Claim::new(format!("dep:{p}"), true))
        .collect();
    if dependencies.is_empty() {
        return Ok(());
    }
    record(&dependencies).map_err(PerformError::Record)
}

/// Takes back what `claims`, the owned claims of a done step that pass to no other app, changed,
/// from `prior`. What was changed since, a key the user set again, a unit they turned back on,
/// stays as it is. Returns what the user has to know of what it left, and why.
///
/// # Errors
///
/// [`PerformError`] for the change the machine refused, or a removal apt would take further
/// than the action's own packages.
#[expect(clippy::too_many_lines, reason = "one arm for each kind of action")]
pub fn undo(
    action: &Action,
    host: &mut dyn Host,
    claims: &[Claim],
    prior: &Prior,
    undoing: Undoing<'_>,
) -> Result<Option<String>, PerformError> {
    let owned = |prefix: &str| owned_names(claims, prefix);
    let undone = match &action.kind {
        Kind::Apt {
            versions: Some(versions),
            ..
        } => {
            // A build of the name outside `versions` (one a refused install left in place) isn't
            // the one the step installed.
            let ours: Vec<String> = installed_as(host, &owned("package:"), Some(versions))?
                .into_iter()
                .collect();
            undo_packages(host, &ours, &owned("dep:"), removal(action))
        }
        Kind::Apt { .. } | Kind::Deb { .. } => undo_packages(host, &owned("package:"), &owned("dep:"), removal(action)),
        Kind::Nix { .. } => {
            let mine = owned("nix:");
            if mine.is_empty() {
                return Ok(None);
            }
            host.nix_remove(&mine).map_err(PerformError::Host)
        }
        Kind::Download { .. }
        | Kind::File { .. }
        | Kind::Symlink { .. }
        | Kind::Output { .. }
        | Kind::WebApp { .. } => {
            for path in owned("path:") {
                let logical_path = PathBuf::from(&path);
                remove_path(&host.place(&logical_path))?;
                if matches!(action.kind, Kind::File { .. }) {
                    reload_if_unit(host, &logical_path)?;
                }
            }
            remove_dirs(host, owned("dir:"));
            Ok(())
        }
        Kind::Setting {
            file,
            section,
            key,
            value,
        } => {
            if owned("key:").is_empty() {
                return Ok(None);
            }
            undo_setting(&Place::new(host, file, section.as_deref()), key, value, prior)?;
            remove_dirs(host, owned("dir:"));
            Ok(())
        }
        Kind::Unit {
            unit,
            scope,
            enable,
            start,
        } => {
            if owned("unit:").is_empty() {
                return Ok(None);
            }
            let state = host.unit_state(*scope, unit).map_err(PerformError::Host)?;
            let as_left = enable.is_none_or(|e| state.enabled == e) && start.is_none_or(|s| started_as(state, s));
            if !state.exists || !as_left {
                return Ok(None);
            }
            if let (Some(_), Some(was)) = (enable, prior.enabled)
                && state.enabled != was
            {
                host.set_enabled(*scope, unit, was).map_err(PerformError::Host)?;
            }
            if let (Some(_), Some(was)) = (start, prior.active)
                && !started_as(state, was)
            {
                host.set_active(*scope, unit, was).map_err(PerformError::Host)?;
            }
            Ok(())
        }
        Kind::Group { group } => {
            if !owned("member:").is_empty() && host.member(group).map_err(PerformError::Host)? {
                host.set_member(group, false).map_err(PerformError::Host)?;
            }
            if !owned("group:").is_empty() && host.group_exists(group).map_err(PerformError::Host)? {
                host.group_delete(group).map_err(PerformError::Host)?;
            }
            Ok(())
        }
        Kind::Linger {} => {
            if owned("linger:").is_empty() {
                return Ok(None);
            }
            host.set_linger(false).map_err(PerformError::Host)
        }
        Kind::Architecture { architecture } => {
            if owned("arch:").is_empty() {
                return Ok(None);
            }
            return undo_architecture(host, architecture, undoing.brought);
        }
        Kind::Dkms { module, version, .. } => {
            return undo_dkms(host, module, version, claims, undoing.replacing);
        }
        Kind::Run { undo, .. } => {
            if owned("run").is_empty() {
                return Ok(None);
            }
            run_recorded(host, action, undo)?.map_err(PerformError::Host)
        }
    };
    undone.map(|()| None)
}

/// Has dpkg take packages of `arch`, then apt read its sources again: it has no list of the
/// architecture's packages before.
fn add_architecture(host: &mut dyn Host, arch: &str) -> Result<(), PerformError> {
    host.set_foreign_arch(arch, true).map_err(PerformError::Host)?;
    host.apt_update().map_err(PerformError::Host)
}

/// Takes `arch` away from dpkg, which an app's step added. First what the app `brought` of it
/// goes, purged by dpkg: a package its removal left with only its configuration (`rc`, as
/// `libc6:i386` with its conffiles), and one it left installed, as apt marks it important
/// (`libgcc-s1:i386`, Protected) or something of the app's own still needed it then, while
/// removing them takes nothing else and the user hasn't installed one by name since. A package
/// dpkg still has of the architecture, installed or with its configuration, keeps it, and the
/// note says which. The machine's own architecture is dpkg's always, and stays.
fn undo_architecture(host: &mut dyn Host, arch: &str, brought: &[String]) -> Result<Option<String>, PerformError> {
    if arch == host.arch() {
        return Ok(None);
    }
    let known = host.arch_packages(arch).map_err(PerformError::Host)?;
    let ours: Vec<String> = known.keys().filter(|p| brought.contains(*p)).cloned().collect();
    let installed: Vec<String> = ours.iter().filter(|p| known[*p] == Known::Installed).cloned().collect();
    let theirs = host.manual(&installed).map_err(PerformError::Host)?;
    let ours: Vec<String> = ours.into_iter().filter(|p| !theirs.contains(p)).collect();
    let installed: Vec<String> = installed.into_iter().filter(|p| !theirs.contains(p)).collect();
    let free = installed.is_empty()
        || host
            .apt_simulate(AptPlan::Remove(&installed))
            .map_err(PerformError::Host)?
            .removes
            .iter()
            .all(|p| installed.contains(p));
    let purged: Vec<String> = if free {
        ours
    } else {
        ours.into_iter().filter(|p| !installed.contains(p)).collect()
    };
    if !purged.is_empty() {
        // HAZARD: dpkg, not apt: the dry run above isn't the purge, and a package the user
        // installs between the two that needs one of these would be one apt's `-y` takes too,
        // where dpkg refuses. Only an architecture's packages beside the machine's own may go
        // past the Essential and Protected marks: nothing of the machine's own needs them.
        host.dpkg_purge(&purged).map_err(PerformError::Host)?;
    }
    let left = host.arch_packages(arch).map_err(PerformError::Host)?;
    if left.is_empty() {
        host.set_foreign_arch(arch, false).map_err(PerformError::Host)?;
        return Ok(None);
    }
    Ok(Some(kept_note(arch, &left)))
}

/// What a note says of `left`, the packages of `arch` that keep dpkg taking it, by whether each is
/// installed or has only its configuration left.
fn kept_note(arch: &str, left: &BTreeMap<String, Known>) -> String {
    let with = |known: Known| -> Vec<&str> {
        left.iter()
            .filter(|(_, k)| **k == known)
            .map(|(p, _)| p.as_str())
            .collect()
    };
    let (installed, configured) = (with(Known::Installed), with(Known::ConfigFiles));
    match (installed.as_slice(), configured.as_slice()) {
        ([one], []) => format!("dpkg keeps taking {arch} packages, as {one} is still installed."),
        (many, []) => format!(
            "dpkg keeps taking {arch} packages, as {} of them are still installed: {}.",
            many.len(),
            many.join(", ")
        ),
        ([], [one]) => format!("dpkg keeps taking {arch} packages, as {one}'s configuration is still there."),
        ([], many) => format!(
            "dpkg keeps taking {arch} packages, as {} of them still have their configuration there: {}.",
            many.len(),
            many.join(", ")
        ),
        (installed, configured) => format!(
            "dpkg keeps taking {arch} packages, as some of them are still installed ({}) and others still have their configuration there ({}).",
            installed.join(", "),
            configured.join(", ")
        ),
    }
}

/// How an `apt` or `deb` step's undo leaves its packages' configuration: purged with `purge`.
fn removal(action: &Action) -> Removal {
    match action.kind {
        Kind::Apt { purge: true, .. } => Removal::Purge,
        _ => Removal::KeepConfig,
    }
}

/// Removes `mine`, the packages a step installed, and of `dependencies`, those it pulled in, the
/// ones nothing else on the machine needs, the user hasn't installed by name since, and apt will
/// remove: one it marks important (Essential or Protected, as `libgcc-s1:i386` is) stays, as apt
/// refuses its removal though its simulation shows it. apt simulates each removal first, and never
/// takes a package that isn't the step's; `removal` keeps or purges their configuration.
fn undo_packages(
    host: &mut dyn Host,
    mine: &[String],
    dependencies: &[String],
    removal: Removal,
) -> Result<(), PerformError> {
    let all: Vec<String> = mine.iter().chain(dependencies).cloned().collect();
    let installed = host.installed(&all).map_err(PerformError::Host)?;
    let there: Vec<String> = mine.iter().filter(|p| installed.contains(*p)).cloned().collect();
    let pulled: Vec<String> = dependencies
        .iter()
        .filter(|p| installed.contains(*p))
        .cloned()
        .collect();
    if !there.is_empty() {
        let taken = host.apt_simulate(AptPlan::Remove(&there)).map_err(PerformError::Host)?;
        let others: Vec<String> = taken.removes.into_iter().filter(|p| !there.contains(p)).collect();
        if !others.is_empty() {
            return Err(PerformError::AptTakes { packages: others });
        }
    }
    // One the user has since installed by name is theirs now, and stays.
    let theirs = host.manual(&pulled).map_err(PerformError::Host)?;
    let important = host.important(&pulled).map_err(PerformError::Host)?;
    let pulled: Vec<String> = pulled
        .into_iter()
        .filter(|p| !theirs.contains(p) && !important.contains(p))
        .collect();
    let mut gone = there;
    gone.extend(unneeded(host, &gone, pulled)?);
    if gone.is_empty() {
        return Ok(());
    }
    // The removal is apt's, whose hooks take the snapshots around it (snapper's), so it isn't
    // dpkg's, which would refuse rather than cascade. The last dry run is of exactly what goes,
    // right before it: a package installed earlier that needs one of them refuses the removal,
    // and one apt's `-y` could still take is one installed in the gap between the two apt runs.
    let taken = host.apt_simulate(AptPlan::Remove(&gone)).map_err(PerformError::Host)?;
    let others: Vec<String> = taken.removes.into_iter().filter(|p| !gone.contains(p)).collect();
    if !others.is_empty() {
        return Err(PerformError::AptTakes { packages: others });
    }
    host.apt_remove(&gone, removal).map_err(PerformError::Host)
}

/// The packages of `dependencies` that go with `packages` taking nothing else with them. One that
/// a package outside them depends on stays, and so does what it needs in turn.
fn unneeded(
    host: &mut dyn Host,
    packages: &[String],
    mut dependencies: Vec<String>,
) -> Result<Vec<String>, PerformError> {
    let outside = |taken: AptSimulation, kept: &[String]| -> bool {
        taken.removes.iter().any(|p| !packages.contains(p) && !kept.contains(p))
    };
    while !dependencies.is_empty() {
        let all: Vec<String> = packages.iter().chain(&dependencies).cloned().collect();
        let taken = host.apt_simulate(AptPlan::Remove(&all)).map_err(PerformError::Host)?;
        if !outside(taken, &dependencies) {
            return Ok(dependencies);
        }
        let mut free = Vec::new();
        for dependency in &dependencies {
            let mut one = packages.to_vec();
            one.push(dependency.clone());
            let taken = host.apt_simulate(AptPlan::Remove(&one)).map_err(PerformError::Host)?;
            if !outside(taken, &dependencies) {
                free.push(dependency.clone());
            }
        }
        if free.len() == dependencies.len() {
            return Ok(Vec::new());
        }
        dependencies = free;
    }
    Ok(dependencies)
}

/// Runs `argv` of the `run` `action` in a fresh folder holding the scripts it recorded, so what
/// runs is what the journal says, whatever the app's folder holds now. The outer error is the
/// folder that couldn't be made; the inner one is the run's.
fn run_recorded(host: &mut dyn Host, action: &Action, argv: &[String]) -> Result<Result<(), HostError>, PerformError> {
    let Kind::Run { scripts, .. } = &action.kind else {
        return Ok(Ok(()));
    };
    let dir = tempfile::Builder::new()
        .prefix("kubaos-run-")
        .tempdir()
        .map_err(PerformError::io(Path::new("a temporary folder")))?;
    for (name, text) in scripts {
        let path = dir.path().join(name);
        make_parents(&path)?;
        write_file(&path, text, SCRIPT_MODE)?;
    }
    Ok(host.run(argv, dir.path()))
}

/// The folders `dirs` an action created, deepest first, each only while it is empty: a folder
/// something else has put a file in since stays.
fn remove_dirs(host: &dyn Host, mut dirs: Vec<String>) {
    dirs.sort_by_key(|d| std::cmp::Reverse(d.len()));
    for dir in dirs {
        let _ = fs::remove_dir(host.place(Path::new(&dir)));
    }
}

/// A look at a set of names, each claimed as `<prefix>:<name>`, owned when it isn't `there`.
fn parts(prefix: &str, names: &[String], there: impl Fn(&String) -> bool) -> Look {
    let claims: Vec<Claim> = names
        .iter()
        .map(|name| Claim::new(format!("{prefix}:{name}"), !there(name)))
        .collect();
    Look {
        done: claims.iter().all(|c| !c.owned),
        claims,
        prior: Prior::default(),
    }
}

/// A `file` step: done when the file holds `text` with its mode. A file there already that the
/// vendor's package wrote (`vendor_keys`) is claimed but not owned, so the step writes over it and
/// a removal leaves it.
fn look_file(host: &dyn Host, kind: &Kind) -> Look {
    let Kind::File {
        to,
        text,
        mode,
        vendor_keys,
        binary,
        ..
    } = kind
    else {
        unreachable!("look_file looks at a file step");
    };
    let path = logical(host, to);
    let text = text.as_deref().unwrap_or_default();
    let want_mode = mode.as_deref().and_then(parse_mode).unwrap_or(DEFAULT_FILE_MODE);
    // A journal whose bytes don't decode can't be what is on disk: the step writes them again,
    // and the write refuses them.
    let done = file_contents(Some(text), *binary).is_some_and(|contents| has_file(host, &path, &contents, want_mode));
    if *vendor_keys && !done && vendor_wrote(host, &path, text) {
        return Look {
            done,
            claims: vec![Claim::new(format!("path:{}", path.display()), false)],
            prior: Prior::default(),
        };
    }
    at_path(host, &path, done)
}

/// Whether the file at `path` is the vendor's: a regular file whose every line but a blank one or a
/// comment sets, as `key=value`, a key `ours` sets too.
fn vendor_wrote(host: &dyn Host, path: &Path, ours: &str) -> bool {
    let place = host.place(path);
    if !place.symlink_metadata().is_ok_and(|m| m.is_file()) {
        return false;
    }
    let Ok(theirs) = fs::read_to_string(&place) else {
        return false;
    };
    let keys: Vec<&str> = ours.lines().filter_map(shell_key).collect();
    theirs.lines().all(|line| {
        let line = line.trim();
        line.is_empty() || line.starts_with('#') || shell_key(line).is_some_and(|k| keys.contains(&k))
    })
}

/// The key a `key=value` line sets, as a shell reads it, when its value is one plain word: bare
/// (letters, digits and `._-/:+,@%=`), or quoted whole in `"` with no `$`, backtick, `\` or `"`
/// inside, or in `'` with no `'` inside. A line that would run or expand anything sets more than
/// a value, so it isn't one.
fn shell_key(line: &str) -> Option<&str> {
    let (key, value) = line.trim().split_once('=')?;
    let mut chars = key.chars();
    let first = chars.next()?;
    let key_ok = (first.is_ascii_alphabetic() || first == '_') && chars.all(|c| c.is_ascii_alphanumeric() || c == '_');
    (key_ok && plain_word(value)).then_some(key)
}

/// Whether `value` is one word a shell reads as itself, as [`shell_key`] says.
fn plain_word(value: &str) -> bool {
    let quoted = |quote: char, refused: &[char]| {
        value
            .strip_prefix(quote)
            .and_then(|v| v.strip_suffix(quote))
            .is_some_and(|inner| !inner.contains(refused) && !inner.contains(|c: char| c.is_control()))
    };
    value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || "._-/:+,@%=".contains(c))
        || (value.len() >= 2 && (quoted('"', &['$', '`', '\\', '"']) || quoted('\'', &['\''])))
}

/// A look at a path: the folders above it the action would create, then the path itself.
fn at_path(host: &dyn Host, path: &Path, done: bool) -> Look {
    let mut claims = Vec::new();
    if !done {
        let mut missing: Vec<&Path> = path
            .ancestors()
            .skip(1)
            .take_while(|dir| *dir != host.home() && dir.parent().is_some() && !host.place(dir).exists())
            .collect();
        missing.reverse();
        claims.extend(
            missing
                .into_iter()
                .map(|d| Claim::new(format!("dir:{}", d.display()), true)),
        );
    }
    claims.push(Claim::new(format!("path:{}", path.display()), !done));
    Look {
        done,
        claims,
        prior: Prior::default(),
    }
}

/// The names of `claims` owned under `prefix` (`package:`), without it.
fn owned_names(claims: &[Claim], prefix: &str) -> Vec<String> {
    claims
        .iter()
        .filter(|c| c.owned)
        .filter_map(|c| c.claim.strip_prefix(prefix))
        .map(str::to_owned)
        .collect()
}

/// `Ok(true)` for a command that exited 0, `Ok(false)` for one that exited otherwise.
fn ran(result: Result<(), HostError>) -> Result<bool, PerformError> {
    match result {
        Ok(()) => Ok(true),
        Err(HostError::Failed { .. }) => Ok(false),
        Err(e) => Err(PerformError::Host(e)),
    }
}

fn make_parents(path: &Path) -> Result<(), PerformError> {
    match path.parent() {
        Some(dir) => fs::create_dir_all(dir).map_err(PerformError::io(dir)),
        None => Ok(()),
    }
}

/// The bytes a `file` step writes: its text, or for a `binary` one the bytes its base64 text
/// holds; none when that text isn't base64.
fn file_contents(text: Option<&str>, binary: bool) -> Option<Vec<u8>> {
    let text = text.unwrap_or_default();
    if binary {
        BASE64.decode(text).ok()
    } else {
        Some(text.as_bytes().to_vec())
    }
}

/// Writes `text` to `path`, as `write_bytes` does.
fn write_file(path: &Path, text: &str, mode: u32) -> Result<(), PerformError> {
    write_bytes(path, text.as_bytes(), mode)
}

/// Writes `contents` to `path` through a new file renamed into place, so a reader never sees
/// half of it and a link left at the staging path is never followed.
fn write_bytes(path: &Path, contents: &[u8], mode: u32) -> Result<(), PerformError> {
    let tmp = staging(path);
    write_new(&tmp, contents).map_err(PerformError::io(&tmp))?;
    fs::set_permissions(&tmp, fs::Permissions::from_mode(mode)).map_err(PerformError::io(&tmp))?;
    fs::rename(&tmp, path).map_err(PerformError::io(path))
}

/// The path beside `path` a change is staged at before it is renamed into place.
fn staging(path: &Path) -> PathBuf {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    path.with_file_name(format!(".{name}.kubaos-new"))
}

/// Downloads into a staging path beside `path`, checks it and moves it into place, so `path` is
/// there whole or not at all; one already there (an older pin's) is replaced. What a tarball
/// holds is unpacked as the one who unpacks it owns it, and no one else may write it, whatever
/// the archive or the umask says.
fn download_to(
    host: &mut dyn Host,
    url: &str,
    sha256: &str,
    path: &Path,
    unpack: Unpack,
    strip: u32,
) -> Result<(), PerformError> {
    let stage = staging(path);
    let file = stage.with_extension("download");
    remove_path(&stage)?;
    let result = (|| {
        write_new(&file, b"").map_err(PerformError::io(&file))?;
        host.fetch(url, sha256, &file).map_err(PerformError::Host)?;
        match unpack {
            Unpack::File => {
                fs::set_permissions(&file, fs::Permissions::from_mode(DOWNLOAD_FILE_MODE))
                    .map_err(PerformError::io(&file))?;
                // An older pin may have unpacked a folder here, which a file can't be renamed over.
                if path.symlink_metadata().is_ok_and(|m| m.is_dir()) {
                    replace_with(&file, path)
                } else {
                    fs::rename(&file, path).map_err(PerformError::io(path))
                }
            }
            Unpack::TarGz | Unpack::TarZst | Unpack::TarXz => {
                fs::create_dir(&stage).map_err(PerformError::io(&stage))?;
                let compression = match unpack {
                    Unpack::TarZst => Compression::Zstd,
                    Unpack::TarXz => Compression::Xz,
                    Unpack::TarGz | Unpack::File => Compression::Gzip,
                };
                host.untar(&file, &stage, compression, strip)
                    .map_err(PerformError::Host)?;
                only_owner_writes(&stage)?;
                replace_with(&stage, path)
            }
        }
    })();
    let _ = fs::remove_file(&file);
    if result.is_err() {
        let _ = remove_path(&stage);
    }
    result
}

/// Moves `stage` to `path`. A folder can't be renamed over another, so one at `path` is moved
/// aside first, and back if `stage` can't take its place, then removed. One a stopped run left
/// aside goes too.
fn replace_with(stage: &Path, path: &Path) -> Result<(), PerformError> {
    let aside = staging(path).with_extension("old");
    if path.symlink_metadata().is_err() {
        fs::rename(stage, path).map_err(PerformError::io(path))?;
        return remove_path(&aside);
    }
    remove_path(&aside)?;
    fs::rename(path, &aside).map_err(PerformError::io(path))?;
    if let Err(e) = fs::rename(stage, path) {
        let _ = fs::rename(&aside, path);
        return Err(PerformError::io(path)(e));
    }
    remove_path(&aside)
}

/// Takes the write bits for the group and others off `path` and everything in it; links stay.
fn only_owner_writes(path: &Path) -> Result<(), PerformError> {
    let meta = path.symlink_metadata().map_err(PerformError::io(path))?;
    if meta.file_type().is_symlink() {
        return Ok(());
    }
    let mode = meta.permissions().mode();
    if mode & WRITABLE_BY_OTHERS != 0 {
        fs::set_permissions(path, fs::Permissions::from_mode(mode & !WRITABLE_BY_OTHERS & 0o7777))
            .map_err(PerformError::io(path))?;
    }
    if meta.is_dir() {
        for entry in fs::read_dir(path).map_err(PerformError::io(path))? {
            only_owner_writes(&entry.map_err(PerformError::io(path))?.path())?;
        }
    }
    Ok(())
}

/// `path` gone, a file, a link or a folder with what is in it; nothing when it isn't there.
fn remove_path(path: &Path) -> Result<(), PerformError> {
    let removed = match path.symlink_metadata() {
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(e) => Err(e),
        Ok(meta) if meta.is_dir() => fs::remove_dir_all(path),
        Ok(_) => fs::remove_file(path),
    };
    removed.map_err(PerformError::io(path))
}

/// `daemon-reload` of the manager whose unit folder `path` is in, as a unit file changed.
fn reload_if_unit(host: &mut dyn Host, path: &Path) -> Result<(), PerformError> {
    let Some(scope) = unit_scope(host, path) else {
        return Ok(());
    };
    host.daemon_reload(scope).map_err(PerformError::Host)
}

/// The manager whose unit folder `path` is in.
fn unit_scope(host: &dyn Host, path: &Path) -> Option<Scope> {
    if path.starts_with("/etc/systemd/system") {
        Some(Scope::System)
    } else if path.starts_with(host.home().join(".config/systemd/user")) {
        Some(Scope::User)
    } else {
        None
    }
}

/// A unit file `path` a step just wrote: its manager reloaded, and its unit, when it runs,
/// restarted, so it runs what the file now says (a program in a new version's folder, the old
/// one's gone). A drop-in, in a folder of the unit's, only reloads.
fn reload_unit_file(host: &mut dyn Host, path: &Path) -> Result<(), PerformError> {
    let Some(scope) = unit_scope(host, path) else {
        return Ok(());
    };
    host.daemon_reload(scope).map_err(PerformError::Host)?;
    let in_unit_folder = path
        .parent()
        .is_some_and(|dir| dir == Path::new("/etc/systemd/system") || dir == host.home().join(".config/systemd/user"));
    let Some(unit) = path.file_name().map(|n| n.to_string_lossy().into_owned()) else {
        return Ok(());
    };
    if !in_unit_folder || !super::is_unit_name(&unit) {
        return Ok(());
    }
    if host.unit_state(scope, &unit).map_err(PerformError::Host)?.active {
        host.restart(scope, &unit).map_err(PerformError::Host)?;
    }
    Ok(())
}

/// Where a setting is: its file, as the action names it and as the host has it, and the INI
/// section its key is under, if any.
struct Place<'a> {
    path: PathBuf,
    placed: PathBuf,
    section: Option<&'a str>,
}

impl<'a> Place<'a> {
    fn new(host: &dyn Host, file: &str, section: Option<&'a str>) -> Self {
        let path = logical(host, file);
        let placed = host.place(&path);
        Self { path, placed, section }
    }

    /// The file's settings and its text; none when it isn't there.
    fn read(&self) -> Result<Option<(Settings, String)>, PerformError> {
        let text = match fs::read_to_string(&self.placed) {
            Ok(text) => text,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(PerformError::io(&self.placed)(e)),
        };
        let settings = self.parse(&text).map_err(PerformError::Settings)?;
        Ok(Some((settings, text)))
    }

    fn parse(&self, text: &str) -> Result<Settings, SettingsError> {
        Settings::parse(&self.path, self.section, text)
    }

    fn empty(&self) -> Settings {
        Settings::empty(&self.path, self.section)
    }

    /// The claim on the key, its section named so two sections' keys of one name are two keys.
    fn claim(&self, key: &str) -> String {
        match self.section {
            Some(section) => format!("key:{}#[{section}]{key}", self.path.display()),
            None => format!("key:{}#{key}", self.path.display()),
        }
    }

    fn write(&self, settings: &Settings) -> Result<(), PerformError> {
        make_parents(&self.placed)?;
        write_file(&self.placed, &settings.render(), self.mode())
    }

    fn mode(&self) -> u32 {
        fs::metadata(&self.placed).map_or(DEFAULT_FILE_MODE, |m| m.permissions().mode() & 0o777)
    }
}

fn look_setting(host: &dyn Host, place: &Place<'_>, key: &str, value: &toml::Value) -> Result<Look, PerformError> {
    let path = &place.path;
    let found = place.read()?;
    let exists = found.is_some();
    let (settings, text) = found.unwrap_or_else(|| (place.empty(), String::new()));
    let done = settings.has(path, key, value).map_err(PerformError::Settings)?;
    let old = settings.get(path, key).map_err(PerformError::Settings)?;
    let tables = settings.tables(path, key).map_err(PerformError::Settings)?;
    let mut claims = Vec::new();
    if !exists {
        claims = at_path(host, path, false).claims;
        claims.retain(|c| c.claim.starts_with("dir:"));
    }
    claims.push(Claim::new(place.claim(key), !done));
    Ok(Look {
        done,
        claims,
        prior: Prior {
            value: old,
            text: exists.then_some(text),
            file: Some(exists),
            tables: Some(tables),
            ..Prior::default()
        },
    })
}

fn apply_setting(place: &Place<'_>, key: &str, value: &toml::Value) -> Result<(), PerformError> {
    let (mut settings, _) = place.read()?.unwrap_or_else(|| (place.empty(), String::new()));
    let written = settings::written(&settings, value);
    settings
        .set(&place.path, key, Some(&written), usize::MAX)
        .map_err(PerformError::Settings)?;
    place.write(&settings)
}

/// Puts a setting back. While the file is exactly as the step left it, it gets its old text back
/// whole, so its layout and key order are the user's again; once something else changed it, only
/// the key goes back. A key changed since, or a file that is no longer settings this action can
/// read, stays as it is.
fn undo_setting(place: &Place<'_>, key: &str, value: &toml::Value, prior: &Prior) -> Result<(), PerformError> {
    let path = &place.path;
    let Ok(Some((mut settings, text))) = place.read() else {
        return Ok(());
    };
    if !settings.has(path, key, value).unwrap_or(false) {
        return Ok(());
    }
    let written = settings::written(&settings, value);
    if let Some(old) = &prior.text
        && let Ok(mut left) = place.parse(old)
        && left.set(path, key, Some(&written), usize::MAX).is_ok()
        && left.render() == text
    {
        return write_file(&place.placed, old, place.mode());
    }
    let keep = prior.tables.unwrap_or(usize::MAX);
    settings
        .set(path, key, prior.value.as_deref(), keep)
        .map_err(PerformError::Settings)?;
    if prior.file == Some(false) && settings.is_empty() {
        return remove_path(&place.placed);
    }
    place.write(&settings)
}
