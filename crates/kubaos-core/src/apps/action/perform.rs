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

use std::fs;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::fs::symlink;
use std::path::Path;
use std::path::PathBuf;

use super::Action;
use super::Kind;
use super::Scope;
use super::Unpack;
use super::parse_mode;
use super::settings;
use super::settings::Settings;
use super::settings::SettingsError;
use crate::apps::host::AptPlan;
use crate::apps::host::AptSimulation;
use crate::apps::host::Host;
use crate::apps::host::HostError;
use crate::apps::host::UnitState;
use crate::apps::journal::Claim;
use crate::apps::journal::JournalError;
use crate::apps::journal::Prior;
use crate::apps::journal::write_new;

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
    #[error(transparent)]
    Settings(SettingsError),
    #[error("installing it would remove {}, which this app didn't install; nothing was changed", packages.join(", "))]
    AptRemoves { packages: Vec<String> },
    #[error("removing it would also remove {}, which this app didn't install; nothing was changed", packages.join(", "))]
    AptTakes { packages: Vec<String> },
    #[error("recording what it is about to install")]
    Record(#[source] JournalError),
}

impl PerformError {
    fn io(path: &Path) -> impl FnOnce(io::Error) -> Self + '_ {
        move |source| Self::Io {
            path: path.to_path_buf(),
            source,
        }
    }
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
    let Some(folder) = path.ancestors().find(|dir| dir.file_name().is_some_and(|n| n == version)) else {
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
            && host.place(&versions.join(entry.file_name()).join(rest)).symlink_metadata().is_ok()
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

/// A web app's desktop entry and icon, as an action names paths.
pub fn web_app_paths(id: &str) -> [String; 2] {
    [
        format!("~/.local/share/applications/kubaos-web-{id}.desktop"),
        format!("~/.local/share/icons/hicolor/scalable/apps/kubaos-web-{id}.svg"),
    ]
}

/// A web app's desktop entry: `url` in an app window of the browser, with the window class
/// `kubaos-web-<id>`, and the icon at `icon`.
fn web_app_entry(id: &str, title: &str, url: &str, icon: &Path) -> String {
    let class = format!("kubaos-web-{id}");
    let exec = [
        super::WEB_APP_BROWSER.to_owned(),
        format!("--app={url}"),
        format!("--class={class}"),
    ]
    .map(|arg| exec_arg(&arg))
    .join(" ");
    format!(
        "[Desktop Entry]\nType=Application\nName={title}\nExec={exec}\nIcon={icon}\nStartupWMClass={class}\n\
         Terminal=false\n",
        title = entry_value(title),
        icon = entry_value(&icon.to_string_lossy()),
    )
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

/// A web app's two files, each with its path and its text.
fn web_app_files(host: &dyn Host, action: &Action) -> Vec<(PathBuf, String)> {
    let Kind::WebApp {
        id, title, url, svg, ..
    } = &action.kind
    else {
        return Vec::new();
    };
    let [entry, icon] = web_app_paths(id).map(|p| logical(host, &p));
    let text = web_app_entry(id, title, url, &icon);
    vec![(entry, text), (icon, svg.clone().unwrap_or_default())]
}

/// Whether a unit stands as `start` asks: started (running, or run and exited cleanly, as a
/// oneshot does), or not running.
fn started_as(state: UnitState, start: bool) -> bool {
    if start { state.started() } else { !state.active }
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
        Kind::Apt { packages, arch, .. } => {
            if let Some(only) = arch
                && only != host.arch()
            {
                return Err(PerformError::Unavailable {
                    arch: host.arch().to_owned(),
                });
            }
            let installed = host.installed(packages).map_err(PerformError::Host)?;
            Ok(parts("package", packages, |p| installed.contains(p)))
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
        Kind::Nix { packages } => {
            let listed = host.nix_list().map_err(PerformError::Host)?;
            Ok(parts("nix", packages, |p| listed.contains(p)))
        }
        Kind::Download {
            to,
            version,
            self_updates,
            ..
        } => {
            let path = logical(host, to);
            let there = host.place(&path).symlink_metadata().is_ok();
            let holds = match recorded {
                Recorded::Done(had) if had.path() == action.path() => same_download(had, action, host.arch()),
                Recorded::Unsettled(had) => had.path() != action.path(),
                Recorded::Done(_) | Recorded::Nothing => true,
            };
            let moved_on = *self_updates && !there && newer_version_there(host, &path, version);
            Ok(at_path(host, &path, (there && holds) || moved_on))
        }
        Kind::File { to, text, mode, .. } => {
            let path = logical(host, to);
            let want_mode = mode.as_deref().and_then(parse_mode).unwrap_or(DEFAULT_FILE_MODE);
            let done = has_file(host, &path, text.as_deref().unwrap_or_default(), want_mode);
            Ok(at_path(host, &path, done))
        }
        Kind::WebApp { .. } => {
            let files = web_app_files(host, action);
            let done = files
                .iter()
                .all(|(path, text)| has_file(host, path, text, DEFAULT_FILE_MODE));
            let mut look = Look {
                done,
                ..Look::default()
            };
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
        Kind::Setting { file, key, value } => look_setting(host, file, key, value),
        Kind::Unit { .. } | Kind::Group { .. } | Kind::Linger {} => look_machine(action, host, &user),
        Kind::Run { check, .. } => {
            let done = ran(run_recorded(host, action, check)?)?;
            Ok(Look {
                done,
                claims: vec![Claim::new("run", !done)],
                prior: Prior::default(),
            })
        }
    }
}

/// Whether `path` is a file holding `text` with `mode`.
fn has_file(host: &dyn Host, path: &Path, text: &str, mode: u32) -> bool {
    let placed = host.place(path);
    placed.symlink_metadata().is_ok_and(|m| m.is_file())
        && fs::read_to_string(&placed).is_ok_and(|t| t == text)
        && fs::metadata(&placed).is_ok_and(|m| m.permissions().mode() & 0o777 == mode)
}

/// Does `action`, which `look`, just taken, found not done. Before apt installs anything,
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
) -> Result<(), PerformError> {
    let owned = |prefix: &str| owned_names(&look.claims, prefix);
    match &action.kind {
        Kind::Apt { repo, packages, .. } => {
            if let Some(repo) = repo {
                host.apt_repo(repo).map_err(PerformError::Host)?;
            }
            let missing = owned("package:");
            if missing.is_empty() {
                return Ok(());
            }
            let simulated = host
                .apt_simulate(AptPlan::Install(&missing))
                .map_err(PerformError::Host)?;
            record_dependencies(simulated, packages, record)?;
            host.apt_install(&missing).map_err(PerformError::Host)
        }
        Kind::Deb { package, .. } => {
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
            record_dependencies(simulated, std::slice::from_ref(package), record)?;
            host.deb_install(&deb).map_err(PerformError::Host)
        }
        Kind::Nix { .. } => host.nix_add(&owned("nix:")).map_err(PerformError::Host),
        Kind::Download { to, unpack, strip, .. } => {
            let download = action.download(host.arch()).ok_or_else(|| PerformError::Unavailable {
                arch: host.arch().to_owned(),
            })?;
            let path = host.place(&logical(host, to));
            make_parents(&path)?;
            download_to(host, &download.url, &download.sha256, &path, *unpack, *strip)
        }
        Kind::File { to, text, mode, .. } => {
            let logical_path = logical(host, to);
            let path = host.place(&logical_path);
            make_parents(&path)?;
            let mode = mode.as_deref().and_then(parse_mode).unwrap_or(DEFAULT_FILE_MODE);
            write_file(&path, text.as_deref().unwrap_or_default(), mode)?;
            reload_if_unit(host, &logical_path)
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
        Kind::Setting { file, key, value } => apply_setting(host, file, key, value),
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
        Kind::Run { run, .. } => run_recorded(host, action, run)?.map_err(PerformError::Host),
    }
}

/// Refuses a change apt would make by removing other packages; else hands `record` the claims of
/// what it would newly install beyond `own`, the action's packages.
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
/// stays as it is.
///
/// # Errors
///
/// [`PerformError`] for the change the machine refused, or a removal apt would take further
/// than the action's own packages.
pub fn undo(action: &Action, host: &mut dyn Host, claims: &[Claim], prior: &Prior) -> Result<(), PerformError> {
    let owned = |prefix: &str| owned_names(claims, prefix);
    match &action.kind {
        Kind::Apt { .. } | Kind::Deb { .. } => undo_packages(host, &owned("package:"), &owned("dep:")),
        Kind::Nix { .. } => {
            let mine = owned("nix:");
            if mine.is_empty() {
                return Ok(());
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
        Kind::Setting { file, key, value } => {
            if owned("key:").is_empty() {
                return Ok(());
            }
            undo_setting(host, file, key, value, prior)?;
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
                return Ok(());
            }
            let state = host.unit_state(*scope, unit).map_err(PerformError::Host)?;
            let as_left = enable.is_none_or(|e| state.enabled == e) && start.is_none_or(|s| started_as(state, s));
            if !state.exists || !as_left {
                return Ok(());
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
                return Ok(());
            }
            host.set_linger(false).map_err(PerformError::Host)
        }
        Kind::Run { undo, .. } => {
            if owned("run").is_empty() {
                return Ok(());
            }
            run_recorded(host, action, undo)?.map_err(PerformError::Host)
        }
    }
}

/// Removes `mine`, the packages a step installed, and of `dependencies`, those it pulled in, the
/// ones nothing else on the machine needs and the user hasn't installed by name since: apt
/// simulates each removal first, and never takes a package that isn't the step's.
fn undo_packages(host: &mut dyn Host, mine: &[String], dependencies: &[String]) -> Result<(), PerformError> {
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
    let pulled: Vec<String> = pulled.into_iter().filter(|p| !theirs.contains(p)).collect();
    let mut gone = there;
    gone.extend(unneeded(host, &gone, pulled)?);
    if gone.is_empty() {
        return Ok(());
    }
    host.apt_remove(&gone).map_err(PerformError::Host)
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

/// Writes `text` to `path` through a new file renamed into place, so a reader never sees half of
/// it and a link left at the staging path is never followed.
fn write_file(path: &Path, text: &str, mode: u32) -> Result<(), PerformError> {
    let tmp = staging(path);
    write_new(&tmp, text.as_bytes()).map_err(PerformError::io(&tmp))?;
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
            Unpack::TarGz => {
                fs::create_dir(&stage).map_err(PerformError::io(&stage))?;
                let out = std::process::Command::new("tar")
                    .arg("-xzf")
                    .arg(&file)
                    .arg("-C")
                    .arg(&stage)
                    .arg(format!("--strip-components={strip}"))
                    .args(["--no-same-owner", "--no-same-permissions"])
                    .output()
                    .map_err(|source| {
                        PerformError::Host(HostError::Spawn {
                            program: "tar".to_owned(),
                            source,
                        })
                    })?;
                if !out.status.success() {
                    return Err(PerformError::Host(HostError::Failed {
                        command: format!("tar -xzf {}", file.display()),
                        code: out.status.code(),
                        stderr: String::from_utf8_lossy(&out.stderr).trim().to_owned(),
                    }));
                }
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
    let scope = if path.starts_with("/etc/systemd/system") {
        Scope::System
    } else if path.starts_with(host.home().join(".config/systemd/user")) {
        Scope::User
    } else {
        return Ok(());
    };
    host.daemon_reload(scope).map_err(PerformError::Host)
}

/// A settings file's settings and its text; none when it isn't there.
fn read_settings(path: &Path, placed: &Path) -> Result<Option<(Settings, String)>, PerformError> {
    let text = match fs::read_to_string(placed) {
        Ok(text) => text,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(PerformError::io(placed)(e)),
    };
    let settings = Settings::parse(path, &text).map_err(PerformError::Settings)?;
    Ok(Some((settings, text)))
}

fn look_setting(host: &dyn Host, file: &str, key: &str, value: &toml::Value) -> Result<Look, PerformError> {
    let path = logical(host, file);
    let found = read_settings(&path, &host.place(&path))?;
    let exists = found.is_some();
    let (settings, text) = found.unwrap_or_else(|| (Settings::empty(&path), String::new()));
    let done = settings.has(&path, key, value).map_err(PerformError::Settings)?;
    let old = settings.get(&path, key).map_err(PerformError::Settings)?;
    let tables = settings.tables(&path, key).map_err(PerformError::Settings)?;
    let mut claims = Vec::new();
    if !exists {
        claims = at_path(host, &path, false).claims;
        claims.retain(|c| c.claim.starts_with("dir:"));
    }
    claims.push(Claim::new(format!("key:{}#{key}", path.display()), !done));
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

fn write_settings(placed: &Path, settings: &Settings) -> Result<(), PerformError> {
    make_parents(placed)?;
    let mode = fs::metadata(placed).map_or(DEFAULT_FILE_MODE, |m| m.permissions().mode() & 0o777);
    write_file(placed, &settings.render(), mode)
}

fn apply_setting(host: &dyn Host, file: &str, key: &str, value: &toml::Value) -> Result<(), PerformError> {
    let path = logical(host, file);
    let placed = host.place(&path);
    let (mut settings, _) = read_settings(&path, &placed)?.unwrap_or_else(|| (Settings::empty(&path), String::new()));
    let written = settings::written(&settings, value);
    settings
        .set(&path, key, Some(&written), usize::MAX)
        .map_err(PerformError::Settings)?;
    write_settings(&placed, &settings)
}

/// Puts a setting back. While the file is exactly as the step left it, it gets its old text back
/// whole, so its layout and key order are the user's again; once something else changed it, only
/// the key goes back. A key changed since, or a file that is no longer settings this action can
/// read, stays as it is.
fn undo_setting(
    host: &dyn Host,
    file: &str,
    key: &str,
    value: &toml::Value,
    prior: &Prior,
) -> Result<(), PerformError> {
    let path = logical(host, file);
    let placed = host.place(&path);
    let Ok(Some((mut settings, text))) = read_settings(&path, &placed) else {
        return Ok(());
    };
    if !settings.has(&path, key, value).unwrap_or(false) {
        return Ok(());
    }
    let written = settings::written(&settings, value);
    if let Some(old) = &prior.text
        && let Ok(mut left) = Settings::parse(&path, old)
        && left.set(&path, key, Some(&written), usize::MAX).is_ok()
        && left.render() == text
    {
        let mode = fs::metadata(&placed).map_or(DEFAULT_FILE_MODE, |m| m.permissions().mode() & 0o777);
        return write_file(&placed, old, mode);
    }
    let keep = prior.tables.unwrap_or(usize::MAX);
    settings
        .set(&path, key, prior.value.as_deref(), keep)
        .map_err(PerformError::Settings)?;
    if prior.file == Some(false) && settings.is_empty() {
        return remove_path(&placed);
    }
    write_settings(&placed, &settings)
}
