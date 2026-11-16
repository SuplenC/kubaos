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
use crate::apps::host::Host;
use crate::apps::host::HostError;
use crate::apps::journal::Claim;
use crate::apps::journal::Prior;

/// The mode a file gets without a `mode`.
const DEFAULT_FILE_MODE: u32 = 0o644;

/// The mode a single-file download gets: such a download is a program.
const DOWNLOAD_FILE_MODE: u32 = 0o755;

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
    #[error("{} isn't settings this action can read", path.display())]
    Settings {
        path: PathBuf,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
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

/// Where the path `named` (`~/…` or absolute) is, as the actions name it.
pub fn logical(host: &dyn Host, named: &str) -> PathBuf {
    match named.strip_prefix("~/") {
        Some(rest) => host.home().join(rest),
        None => PathBuf::from(named),
    }
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
            let done = enable.is_none_or(|e| state.enabled == e) && start.is_none_or(|s| state.active == s);
            Ok(Look {
                done,
                claims: vec![Claim::new(format!("unit:{}:{unit}", scope.word()), !done)],
                prior: Prior {
                    enabled: Some(state.enabled),
                    active: Some(state.active),
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

/// What `action` finds on the machine, a `run` checked in its app's `folder`.
///
/// # Errors
///
/// [`PerformError`] when the machine can't be read, or a `run`'s check can't be started.
pub fn look(action: &Action, host: &mut dyn Host, folder: &Path) -> Result<Look, PerformError> {
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
        Kind::Download { to, .. } => {
            let path = logical(host, to);
            let done = host.place(&path).symlink_metadata().is_ok();
            Ok(at_path(host, &path, done))
        }
        Kind::File { to, text, mode, .. } => {
            let path = logical(host, to);
            let placed = host.place(&path);
            let want_mode = mode.as_deref().and_then(parse_mode).unwrap_or(DEFAULT_FILE_MODE);
            let done = placed.symlink_metadata().is_ok_and(|m| m.is_file())
                && fs::read_to_string(&placed).ok().as_deref() == text.as_deref()
                && fs::metadata(&placed).is_ok_and(|m| m.permissions().mode() & 0o777 == want_mode);
            Ok(at_path(host, &path, done))
        }
        Kind::Symlink { to, target } => {
            let path = logical(host, to);
            let done = fs::read_link(host.place(&path)).is_ok_and(|t| t == Path::new(target));
            Ok(at_path(host, &path, done))
        }
        Kind::Output { path } => {
            let path = logical(host, path);
            Ok(Look {
                done: false,
                claims: vec![Claim::new(format!("path:{}", path.display()), true)],
                prior: Prior::default(),
            })
        }
        Kind::Setting { file, key, value } => look_setting(host, file, key, value),
        Kind::Unit { .. } | Kind::Group { .. } | Kind::Linger {} => look_machine(action, host, &user),
        Kind::Run { check, .. } => {
            let done = ran(host.run(check, folder))?;
            Ok(Look {
                done,
                claims: vec![Claim::new("run", !done)],
                prior: Prior::default(),
            })
        }
    }
}

/// Does `action`, which `look` found not done; `folder` is its app's.
///
/// # Errors
///
/// [`PerformError`] for the change the machine refused.
pub fn apply(action: &Action, host: &mut dyn Host, look: &Look, folder: &Path) -> Result<(), PerformError> {
    let owned = |prefix: &str| owned_names(&look.claims, prefix);
    match &action.kind {
        Kind::Apt { repo, .. } => host
            .apt_install(repo.as_deref(), &owned("package:"))
            .map_err(PerformError::Host),
        Kind::Deb { .. } => {
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
        Kind::Symlink { to, target } => {
            let path = host.place(&logical(host, to));
            make_parents(&path)?;
            if path.symlink_metadata().is_ok() {
                fs::remove_file(&path).map_err(PerformError::io(&path))?;
            }
            symlink(target, &path).map_err(PerformError::io(&path))
        }
        Kind::Output { .. } => Ok(()),
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
                && state.active != start
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
        Kind::Run { run, .. } => host.run(run, folder).map_err(PerformError::Host),
    }
}

/// Whether `action` is done: its check, a `run`'s in its app's `folder`.
///
/// # Errors
///
/// [`PerformError`] when the machine can't be read.
pub fn check(action: &Action, host: &mut dyn Host, folder: &Path) -> Result<bool, PerformError> {
    look(action, host, folder).map(|look| look.done)
}

/// Takes back what `claims`, the owned claims of a done step that pass to no other app, changed,
/// from `prior`. What was changed since, a key the user set again, a unit they turned back on,
/// stays as it is.
///
/// # Errors
///
/// [`PerformError`] for the change the machine refused.
pub fn undo(
    action: &Action,
    host: &mut dyn Host,
    claims: &[Claim],
    prior: &Prior,
    folder: &Path,
) -> Result<(), PerformError> {
    let owned = |prefix: &str| owned_names(claims, prefix);
    match &action.kind {
        Kind::Apt { .. } | Kind::Deb { .. } => {
            let mine = owned("package:");
            let installed = host.installed(&mine).map_err(PerformError::Host)?;
            let there: Vec<String> = mine.into_iter().filter(|p| installed.contains(p)).collect();
            if there.is_empty() {
                return Ok(());
            }
            host.apt_remove(&there).map_err(PerformError::Host)
        }
        Kind::Nix { .. } => {
            let mine = owned("nix:");
            if mine.is_empty() {
                return Ok(());
            }
            host.nix_remove(&mine).map_err(PerformError::Host)
        }
        Kind::Download { .. } | Kind::File { .. } | Kind::Symlink { .. } | Kind::Output { .. } => {
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
            let as_left = enable.is_none_or(|e| state.enabled == e) && start.is_none_or(|s| state.active == s);
            if !state.exists || !as_left {
                return Ok(());
            }
            if let (Some(_), Some(was)) = (enable, prior.enabled)
                && state.enabled != was
            {
                host.set_enabled(*scope, unit, was).map_err(PerformError::Host)?;
            }
            if let (Some(_), Some(was)) = (start, prior.active)
                && state.active != was
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
            host.run(undo, folder).map_err(PerformError::Host)
        }
    }
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

/// Writes `text` to `path` through a file renamed into place, so a reader never sees half of it.
fn write_file(path: &Path, text: &str, mode: u32) -> Result<(), PerformError> {
    let tmp = staging(path);
    fs::write(&tmp, text).map_err(PerformError::io(&tmp))?;
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
/// there whole or not at all.
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
        host.fetch(url, sha256, &file).map_err(PerformError::Host)?;
        match unpack {
            Unpack::File => {
                fs::set_permissions(&file, fs::Permissions::from_mode(DOWNLOAD_FILE_MODE))
                    .map_err(PerformError::io(&file))?;
                fs::rename(&file, path).map_err(PerformError::io(path))
            }
            Unpack::TarGz => {
                fs::create_dir_all(&stage).map_err(PerformError::io(&stage))?;
                let out = std::process::Command::new("tar")
                    .arg("-xzf")
                    .arg(&file)
                    .arg("-C")
                    .arg(&stage)
                    .arg(format!("--strip-components={strip}"))
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
                fs::rename(&stage, path).map_err(PerformError::io(path))
            }
        }
    })();
    let _ = fs::remove_file(&file);
    if result.is_err() {
        let _ = remove_path(&stage);
    }
    result
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

/// A settings file, as JSON, whichever its format.
struct Settings {
    /// The file is there.
    exists: bool,
    /// Its settings; empty when it isn't there or can't be read as settings.
    value: serde_json::Map<String, serde_json::Value>,
    /// The file as it was, when it is there.
    text: Option<String>,
}

fn is_toml(path: &Path) -> bool {
    path.extension().is_some_and(|e| e == "toml")
}

fn read_settings(path: &Path) -> Result<Settings, PerformError> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            return Ok(Settings {
                exists: false,
                value: serde_json::Map::new(),
                text: None,
            });
        }
        Err(e) => return Err(PerformError::io(path)(e)),
    };
    let value = match read_text_settings(path, &text) {
        Some(serde_json::Value::Object(value)) => value,
        _ => serde_json::Map::new(),
    };
    Ok(Settings {
        exists: true,
        value,
        text: Some(text),
    })
}

/// The value at the dotted `key`.
fn get_key<'a>(map: &'a serde_json::Map<String, serde_json::Value>, key: &str) -> Option<&'a serde_json::Value> {
    let mut parts = key.split('.');
    let mut here = map.get(parts.next()?)?;
    for part in parts {
        here = here.as_object()?.get(part)?;
    }
    Some(here)
}

/// Sets the dotted `key`, creating the tables above it; `None` takes it out.
fn set_key(map: &mut serde_json::Map<String, serde_json::Value>, key: &str, value: Option<serde_json::Value>) {
    let parts: Vec<&str> = key.split('.').collect();
    let Some((last, above)) = parts.split_last() else {
        return;
    };
    let mut here = map;
    for part in above {
        let next = here
            .entry((*part).to_owned())
            .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));
        if !next.is_object() {
            *next = serde_json::Value::Object(serde_json::Map::new());
        }
        let serde_json::Value::Object(next) = next else {
            return;
        };
        here = next;
    }
    match value {
        Some(value) => {
            here.insert((*last).to_owned(), value);
        }
        None => {
            here.remove(*last);
        }
    }
}

fn want(value: &toml::Value) -> serde_json::Value {
    serde_json::to_value(value).unwrap_or(serde_json::Value::Null)
}

fn look_setting(host: &dyn Host, file: &str, key: &str, value: &toml::Value) -> Result<Look, PerformError> {
    let path = logical(host, file);
    let settings = read_settings(&host.place(&path))?;
    let current = get_key(&settings.value, key);
    let done = current == Some(&want(value));
    let mut claims = Vec::new();
    if !settings.exists {
        claims = at_path(host, &path, false).claims;
        claims.retain(|c| c.claim.starts_with("dir:"));
    }
    claims.push(Claim::new(format!("key:{}#{key}", path.display()), !done));
    Ok(Look {
        done,
        claims,
        prior: Prior {
            value: current.map(serde_json::Value::to_string),
            text: settings.text,
            file: Some(settings.exists),
            ..Prior::default()
        },
    })
}

/// The settings as the file's text.
fn render(path: &Path, value: &serde_json::Map<String, serde_json::Value>) -> Result<String, PerformError> {
    let settings = |source: Box<dyn std::error::Error + Send + Sync>| PerformError::Settings {
        path: path.to_path_buf(),
        source,
    };
    if is_toml(path) {
        toml::to_string(value).map_err(|e| settings(Box::new(e)))
    } else {
        let mut text = serde_json::to_string_pretty(value).map_err(|e| settings(Box::new(e)))?;
        text.push('\n');
        Ok(text)
    }
}

fn write_settings(path: &Path, value: &serde_json::Map<String, serde_json::Value>) -> Result<(), PerformError> {
    let text = render(path, value)?;
    make_parents(path)?;
    let mode = fs::metadata(path).map_or(DEFAULT_FILE_MODE, |m| m.permissions().mode() & 0o777);
    write_file(path, &text, mode)
}

fn apply_setting(host: &dyn Host, file: &str, key: &str, value: &toml::Value) -> Result<(), PerformError> {
    let path = host.place(&logical(host, file));
    let mut settings = read_settings(&path)?;
    set_key(&mut settings.value, key, Some(want(value)));
    write_settings(&path, &settings.value)
}

/// Puts a setting back. While the file is exactly as the step left it, it gets its old text back
/// whole, so its layout and key order are the user's again; once something else changed it, only
/// the key goes back, and a key changed since stays.
fn undo_setting(
    host: &dyn Host,
    file: &str,
    key: &str,
    value: &toml::Value,
    prior: &Prior,
) -> Result<(), PerformError> {
    let path = host.place(&logical(host, file));
    let mut settings = read_settings(&path)?;
    if get_key(&settings.value, key) != Some(&want(value)) {
        return Ok(());
    }
    if let Some(old) = &prior.text {
        let mut left = match read_text_settings(&path, old) {
            Some(serde_json::Value::Object(map)) => map,
            _ => serde_json::Map::new(),
        };
        set_key(&mut left, key, Some(want(value)));
        if settings.text.as_deref() == Some(render(&path, &left)?.as_str()) {
            let mode = fs::metadata(&path).map_or(DEFAULT_FILE_MODE, |m| m.permissions().mode() & 0o777);
            return write_file(&path, old, mode);
        }
    }
    let old = prior.value.as_deref().and_then(|v| serde_json::from_str(v).ok());
    set_key(&mut settings.value, key, old);
    if prior.file == Some(false) && settings.value.is_empty() {
        return remove_path(&path);
    }
    write_settings(&path, &settings.value)
}

/// `text` read as the settings of the file `path`.
fn read_text_settings(path: &Path, text: &str) -> Option<serde_json::Value> {
    if is_toml(path) {
        toml::from_str::<toml::Table>(text)
            .ok()
            .and_then(|t| serde_json::to_value(t).ok())
    } else {
        serde_json::from_str(text).ok()
    }
}
