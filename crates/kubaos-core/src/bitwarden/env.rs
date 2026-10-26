//! Project env: the vault's variables for a project, cached in the GNOME keyring, where
//! `use bwenv <project>` in an `.envrc` reads them (`home/secrets.nix`) without bw or the
//! master password.
//!
//! In the vault: every custom field of every item in the folder `env/<project>` (the prefix is
//! [`vault::Names::env_prefix`]), plus the items anywhere with a field `bwenv=<project>`. The
//! field's name is the variable's.
//!
//! Keyring entries carry the attributes `service=dotfiles-env project=<p> name=<VAR>` and the
//! label `dotfiles-env/<p>/<VAR>`. A value with a newline or a tab is stored base64 and its
//! label ends in `#base64`, so `secret-tool search` prints one line per value. Nothing goes to
//! disk in plaintext.

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use super::Cmd;
use super::Error;
use super::Host;
use super::HostError;
use super::Machine;
use super::Progress;
use super::Sensitive;
use super::Session;
use super::base64;
use super::vault;
use super::vault::Vault;

/// The keyring's service attribute. HAZARD: the name is the old dotfiles' and stays: every
/// entry a machine already holds carries it, and a new name would orphan them all.
pub const SERVICE: &str = "dotfiles-env";

/// A variable's name: what a shell can export.
pub fn name_ok(n: &str) -> bool {
    let mut c = n.chars();
    c.next().is_some_and(|f| f.is_ascii_alphabetic() || f == '_') && c.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// A project's name: letters, digits, `.`, `_` and `-`, not starting with a symbol.
pub fn project_ok(p: &str) -> bool {
    let mut c = p.chars();
    c.next().is_some_and(|f| f.is_ascii_alphanumeric())
        && c.all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
}

/// # Errors
///
/// The name isn't a project's.
pub fn check_project(p: &str) -> Result<(), Error> {
    if project_ok(p) { Ok(()) } else { Err(Error::Project(p.to_string())) }
}

/// One keyring entry.
#[derive(Debug, Clone)]
pub struct Entry {
    pub project: String,
    pub name: String,
    pub value: Sensitive,
}

/// Every entry of ours, or a project's, in one `secret-tool` call. secret-tool prints the
/// attributes on stderr, so the names come from the label on stdout.
///
/// # Errors
///
/// The keyring can't be read.
pub fn dump(host: &mut dyn Host, project: Option<&str>) -> Result<Vec<Entry>, Error> {
    let mut cmd = Cmd::new("secret-tool").args(["search", "--all", "--unlock", "service", SERVICE]);
    if let Some(p) = project {
        cmd = cmd.args(["project", p]);
    }
    let ran = host.run(&cmd).map_err(|e| Error::Keyring(HostError(e)))?;
    // secret-tool exits 1, silently, when nothing matches.
    let nothing = ran.code == Some(1) && ran.stdout.is_empty() && ran.stderr.trim().is_empty();
    if !(ran.ok() || nothing) {
        return Err(Error::Keyring(HostError(ran.stderr.trim().to_string())));
    }
    let prefix = format!("{SERVICE}/");
    let mut out = vec![];
    let mut label = String::new();
    for line in ran.stdout.lines() {
        if line.starts_with('[') {
            label.clear();
        } else if let Some(l) = line.strip_prefix("label = ") {
            label = l.to_string();
        } else if let Some(secret) = line.strip_prefix("secret = ") {
            let Some(rest) = label.strip_prefix(&prefix) else { continue };
            let (rest, b64) = match rest.strip_suffix("#base64") {
                Some(r) => (r, true),
                None => (rest, false),
            };
            let Some((p, name)) = rest.split_once('/') else { continue };
            let value = if b64 {
                base64::decode(secret).map(|b| String::from_utf8_lossy(&b).into_owned()).unwrap_or_default()
            } else {
                secret.to_string()
            };
            out.push(Entry {
                project: p.to_string(),
                name: name.to_string(),
                value: Sensitive(value),
            });
        }
    }
    Ok(out)
}

/// Puts one variable in the keyring; the value goes on secret-tool's stdin, never its argv.
fn store(host: &mut dyn Host, project: &str, name: &str, value: &str) -> Result<(), Error> {
    let mut label = format!("{SERVICE}/{project}/{name}");
    let encoded;
    let value = if value.contains(['\n', '\r', '\t']) {
        label.push_str("#base64");
        encoded = base64::encode(value.as_bytes());
        encoded.as_str()
    } else {
        value
    };
    let cmd = Cmd::new("secret-tool")
        .arg("store")
        .arg(format!("--label={label}"))
        .args(["service", SERVICE, "project", project, "name", name])
        .secret_stdin(value);
    match host.run(&cmd) {
        Ok(r) if r.ok() => Ok(()),
        Ok(r) => Err(Error::Keyring(HostError(r.stderr.trim().to_string()))),
        Err(e) => Err(Error::Keyring(HostError(e))),
    }
}

/// Forgets a project, or every project with `None`.
pub fn clear(host: &mut dyn Host, project: Option<&str>) {
    let mut cmd = Cmd::new("secret-tool").args(["clear", "service", SERVICE]);
    if let Some(p) = project {
        cmd = cmd.args(["project", p]);
    }
    let _ = host.run(&cmd);
}

/// Projects and their variables' names, never the values, sorted.
///
/// # Errors
///
/// The keyring can't be read.
pub fn list(host: &mut dyn Host, project: Option<&str>) -> Result<BTreeMap<String, BTreeSet<String>>, Error> {
    let mut out: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for e in dump(host, project)? {
        out.entry(e.project).or_default().insert(e.name);
    }
    Ok(out)
}

/// The projects the keyring holds, sorted.
///
/// # Errors
///
/// The keyring can't be read.
pub fn projects(host: &mut dyn Host) -> Result<Vec<String>, Error> {
    Ok(list(host, None)?.into_keys().collect())
}

/// A project's variables from the keyring alone, for `use bwenv`.
///
/// # Errors
///
/// A bad project name, or the keyring can't be read.
pub fn export(host: &mut dyn Host, project: &str) -> Result<Vec<(String, Sensitive)>, Error> {
    check_project(project)?;
    Ok(dump(host, Some(project))?
        .into_iter()
        .filter(|e| e.project == project && name_ok(&e.name))
        .map(|e| (e.name, e.value))
        .collect())
}

/// The projects the vault has: folders `env/<p>`, plus every `bwenv=<p>` field.
fn in_vault(vault: &Vault) -> BTreeSet<String> {
    let folders = vault.folders.iter().filter_map(|f| vault.names.project(&f.name));
    let fields = vault.items.iter().flat_map(|i| {
        i.fields
            .iter()
            .filter(|f| f.is("bwenv"))
            .filter_map(|f| f.value.as_deref())
    });
    folders.chain(fields).map(str::to_string).collect()
}

/// Replaces a project's keyring entries with its items' custom fields. The count stored.
fn import_project(host: &mut dyn Host, vault: &Vault, project: &str, p: &mut Progress) -> Result<usize, Error> {
    let folder = vault.names.env_folder(project);
    let ids: Vec<&String> = vault
        .folders
        .iter()
        .filter(|f| f.name == folder)
        .filter_map(|f| f.id.as_ref())
        .collect();
    let items = vault.items.iter().filter(|i| {
        i.folder_id.as_ref().is_some_and(|f| ids.contains(&f))
            || i.fields.iter().any(|f| f.is("bwenv") && f.value.as_deref() == Some(project))
    });
    let mut vars: Vec<(&str, &str)> = vec![];
    let mut skipped: Vec<&str> = vec![];
    for (name, value) in items.flat_map(vault::Item::env_fields) {
        if name.is_empty() {
            continue;
        }
        if name_ok(name) {
            vars.push((name, value));
        } else {
            skipped.push(name);
        }
    }
    clear(host, Some(project));
    for (name, value) in &vars {
        store(host, project, name, value)?;
    }
    if !skipped.is_empty() {
        p.warn(format!(
            "{project}: skipped fields that aren't variable names: {}",
            super::one_line(&skipped.join(" "))
        ));
    }
    p.info(format!("{project:<20} {} variable(s)", vars.len()));
    Ok(vars.len())
}

/// Imports `want`, or every project in the vault when it's empty. With `prune`, forgets the
/// keyring's projects the vault no longer has. The projects imported, with their counts.
///
/// # Errors
///
/// The keyring can't be written.
pub fn import(
    host: &mut dyn Host,
    vault: &Vault,
    want: &[String],
    prune: bool,
    p: &mut Progress,
) -> Result<Vec<(String, usize)>, Error> {
    let found = in_vault(vault);
    let want: Vec<String> = if want.is_empty() { found.iter().cloned().collect() } else { want.to_vec() };
    let mut done = vec![];
    for project in &want {
        if !project_ok(project) {
            p.warn(format!(
                "skipping project '{}' (letters, digits, . _ - only)",
                super::one_line(project)
            ));
            continue;
        }
        if !found.contains(project) {
            let folder = vault.names.env_folder(project);
            p.warn(format!("{project}: no folder {folder} and no item with bwenv={project} in the vault"));
            continue;
        }
        let n = import_project(host, vault, project, p)?;
        done.push((project.clone(), n));
    }
    if prune {
        for old in projects(host)? {
            if found.contains(&old) {
                continue;
            }
            clear(host, Some(&old));
            p.info(format!("{old}: gone from the vault, removed from the keyring"));
        }
    }
    Ok(done)
}

/// The env part of a full pull: every project, pruning the ones gone.
pub fn setup(m: &mut dyn Machine, vault: &Vault, p: &mut Progress) {
    p.step("Project env: vault -> GNOME keyring");
    match import(m, vault, &[], true, p) {
        Err(e) => p.warn(format!("project env import failed: {e} (keyring locked or missing?)")),
        Ok(done) if done.is_empty() => {
            let folders = vault.names.env_folder("<project>");
            p.info(format!("no {folders} folders and no bwenv fields in the vault"));
            p.summary("env    no projects in the vault");
        }
        Ok(done) => {
            let names: Vec<&str> = done.iter().map(|(n, _)| n.as_str()).collect();
            p.summary(format!("env    {} project(s) in the keyring: {}", done.len(), names.join(" ")));
        }
    }
}

/// Fetches the projects from the vault, finding them by `names`, and replaces what the keyring
/// holds for them.
///
/// # Errors
///
/// A bad project name, bw failed to read the vault, or the keyring can't be written.
pub fn pull(
    m: &mut dyn Machine,
    session: &Session,
    names: &vault::Names,
    projects: &[String],
    p: &mut Progress,
) -> Result<(), Error> {
    for project in projects {
        check_project(project)?;
    }
    p.info("syncing the vault (each bw call takes a few seconds)");
    m.sync(session).map_err(Error::Sync)?;
    let folders = m.folders(session).map_err(|e| Error::List("folders", e))?;
    let folders = vault::folders(&folders).map_err(|e| Error::Parse("folders", e))?;
    let items = m.items(session).map_err(|e| Error::List("items", e))?;
    let vault = Vault::parse(&items, &folders, names).map_err(|e| Error::Parse("items", e))?;
    drop(items);
    import(m, &vault, projects, false, p)?;
    Ok(())
}
