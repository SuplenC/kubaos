//! SSH: a `~/.ssh/<name>.pub` per SSH key item, and a Host block in `~/.ssh/config.d/bitwarden`
//! for each one with a `hosts` field. `IdentityFile` names the public key, so Bitwarden
//! desktop's agent offers that key and no other; the private key stays in the vault.

use std::collections::BTreeMap;

use super::Error;
use super::KeyInfo;
use super::Machine;
use super::Paths;
use super::Progress;
use super::list;
use super::one_line;
use super::vault::Item;

/// An SSH key item and the `.pub` it gets.
pub struct Key {
    pub item: Item,
    /// `~/.ssh/<file>.pub`.
    pub file: String,
}

/// A file name from an item name: "GitHub (work)" is `github-work`.
pub fn slug(name: &str) -> String {
    let mut s = String::new();
    let mut run = false;
    for c in name.chars().map(|c| c.to_ascii_lowercase()) {
        if c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '.' | '_' | '-') {
            s.push(c);
            run = false;
        } else if !run {
            s.push('-');
            run = true;
        }
    }
    let s = s.trim_start_matches(['-', '.']).trim_end_matches('-');
    if s.is_empty() { "key".to_string() } else { s.to_string() }
}

/// A file name this module could have written: the state file is read back, and a name in it
/// is removed from `~/.ssh`, so nothing with a slash gets that far.
pub fn safe_file(name: &str) -> bool {
    !name.is_empty() && !name.starts_with('.') && !name.contains('/')
}

/// Gives every key its `.pub` name, each one different. Keys come oldest first, so of two with
/// the same name the older keeps the plain name, and a new key never takes over a `.pub` in use.
pub fn name_files(items: Vec<Item>, p: &mut Progress) -> Vec<Key> {
    let mut owner: BTreeMap<String, String> = BTreeMap::new();
    let mut keys = vec![];
    for item in items {
        let base = match item.field("file") {
            "" => slug(&item.name),
            want => slug(want),
        };
        let mut file = base.clone();
        let mut k = 2;
        while owner.contains_key(&file) {
            file = format!("{base}-{k}");
            k += 1;
        }
        let who = one_line(&format!("\"{}\" ({})", item.name, item.folder));
        if file != base {
            p.warn(format!(
                "~/.ssh/{base}.pub is taken by {}: {who} gets ~/.ssh/{file}.pub (a 'file' field on it picks the name)",
                owner[&base]
            ));
        }
        owner.insert(file.clone(), who);
        keys.push(Key { item, file });
    }
    keys
}

/// `ssh-ed25519 AAAA…`, `ssh-rsa …` or `ecdsa-sha2-nistpNNN …`, and nothing else on the line.
pub fn usable_public_key(k: &str) -> bool {
    let Some((kind, blob)) = k.split_once(' ') else {
        return false;
    };
    let kind_ok = kind == "ssh-ed25519"
        || kind == "ssh-rsa"
        || kind
            .strip_prefix("ecdsa-sha2-nistp")
            .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()));
    kind_ok && !blob.is_empty() && blob.bytes().all(|b| b.is_ascii_alphanumeric() || b"+/=".contains(&b))
}

/// The first two words of a `.pub` file: the key without its comment.
fn key_of(pub_file: &str) -> String {
    pub_file.split(' ').take(2).collect::<Vec<_>>().join(" ").trim_end().to_string()
}

/// The values a Host block takes from an item, checked for ssh_config.
struct HostFields {
    /// The `hosts` field, split on commas and whitespace.
    hosts: Vec<String>,
    hostname: String,
    user: String,
    port: String,
}

/// Why an item gets no Host block.
enum NoBlock {
    /// A control character: a newline plus `ProxyCommand` would run commands.
    Control,
    NoHosts,
    /// Characters ssh_config can't take safely.
    Unsafe,
}

fn host_fields(item: &Item) -> Result<HostFields, NoBlock> {
    let (hosts, hostname, user, port) = (item.field("hosts"), item.field("hostname"), item.field("user"), item.field("port"));
    if [hosts, hostname, user, port].iter().any(|v| v.chars().any(char::is_control)) {
        return Err(NoBlock::Control);
    }
    let hosts: Vec<String> = hosts
        .split(|c: char| c == ',' || c.is_whitespace())
        .filter(|w| !w.is_empty())
        .map(str::to_string)
        .collect();
    if hosts.is_empty() {
        return Err(NoBlock::NoHosts);
    }
    let all = |s: &str, ok: fn(char) -> bool| s.chars().all(ok);
    let host_ok = hosts
        .iter()
        .all(|h| all(h, |c| c.is_ascii_alphanumeric() || "[].*?!_:%-".contains(c)));
    let hostname_ok = all(hostname, |c| c.is_ascii_alphanumeric() || "._:%-".contains(c));
    let user_ok = all(user, |c| c.is_ascii_alphanumeric() || "._@-".contains(c));
    let port_ok = port.is_empty() || (port.len() <= 5 && all(port, |c| c.is_ascii_digit()));
    if !(host_ok && hostname_ok && user_ok && port_ok) {
        return Err(NoBlock::Unsafe);
    }
    Ok(HostFields {
        hosts,
        hostname: hostname.to_string(),
        user: user.to_string(),
        port: port.to_string(),
    })
}

/// Writes the `.pub` files and the Host blocks, removes the `.pub` files of keys gone from the
/// vault, and says which key got what.
///
/// # Errors
///
/// A file can't be written.
pub fn apply(m: &mut dyn Machine, paths: &Paths, keys: &mut [Key], scope: &str, p: &mut Progress) -> Result<Vec<KeyInfo>, Error> {
    p.step("SSH: public keys and Host blocks");
    for dir in [paths.ssh.clone(), paths.ssh.join("config.d"), paths.state.clone()] {
        m.mkdir(&dir, 0o700).map_err(|e| Error::write(&dir, e))?;
    }
    let sock = paths.agent.display().to_string();
    let mut conf = String::new();
    let mut written = vec![];
    let mut with_block = vec![];
    let mut key_only = vec![];
    let mut host_owner: BTreeMap<String, String> = BTreeMap::new();
    let mut infos = vec![];
    for key in keys.iter() {
        let item = &key.item;
        let label = item.label();
        let fp = one_line(item.fingerprint.as_deref().unwrap_or(""));
        let mut info = KeyInfo {
            name: item.name.clone(),
            folder: item.folder.clone(),
            fingerprint: fp.clone(),
            ..KeyInfo::default()
        };
        let public = item.public_key.as_deref().unwrap_or("");
        if !usable_public_key(public) {
            p.warn(format!("{label}: no usable public key in the item, skipped"));
            info.note = Some("no usable public key in the item".into());
            infos.push(info);
            continue;
        }
        let pub_path = paths.ssh.join(format!("{}.pub", key.file));
        // A private key of that name on disk means the name is taken by a real key.
        if m.exists(&paths.ssh.join(&key.file))
            && m.is_file(&pub_path)
            && m.read(&pub_path).is_some_and(|on_disk| key_of(&on_disk) != public)
        {
            p.warn(format!(
                "{label}: ~/.ssh/{} is a different key on disk; set a 'file' field to pick another name",
                key.file
            ));
            info.note = Some(format!("~/.ssh/{} is a different key on disk", key.file));
            infos.push(info);
            continue;
        }
        let contents = format!("{public} bitwarden:{}\n", one_line(&item.name));
        write_if_changed(m, &pub_path, &contents, 0o644)?;
        written.push(format!("{}.pub", key.file));
        info.file = Some(key.file.clone());
        p.ok(format!("~/.ssh/{}.pub  ({fp})", key.file));

        let fields = match host_fields(item) {
            Ok(f) => f,
            Err(NoBlock::Control) => {
                p.warn(format!(
                    "{label}: a hosts/hostname/user/port field has a newline or control character; no Host block"
                ));
                info.note = Some("a hosts/hostname/user/port field has a control character".into());
                infos.push(info);
                continue;
            }
            Err(NoBlock::NoHosts) => {
                p.info("no 'hosts' field: key only, no Host block");
                info.note = Some("no 'hosts' field".into());
                key_only.push(label);
                infos.push(info);
                continue;
            }
            Err(NoBlock::Unsafe) => {
                p.warn(format!(
                    "{label}: hosts/hostname/user/port contain characters ssh_config can't take safely; no Host block"
                ));
                info.note = Some("hosts/hostname/user/port have characters ssh_config can't take".into());
                infos.push(info);
                continue;
            }
        };
        // ssh collects IdentityFile from every matching block, so a host in two items gets
        // both keys offered and the server takes whichever it knows first.
        for h in &fields.hosts {
            match host_owner.get(h) {
                Some(other) if *other != label => p.warn(format!(
                    "Host {h} is in both {other} and {label}, so ssh offers both keys there; give one an alias of its own (hosts=github-work hostname=github.com)"
                )),
                Some(_) => {}
                None => {
                    host_owner.insert(h.clone(), label.clone());
                }
            }
        }
        let hosts = fields.hosts.join(" ");
        conf.push_str(&format!("# {label} ({fp})\nHost {hosts}\n"));
        for (k, v) in [("HostName", &fields.hostname), ("User", &fields.user), ("Port", &fields.port)] {
            if !v.is_empty() {
                conf.push_str(&format!("  {k} {v}\n"));
            }
        }
        conf.push_str(&format!(
            "  IdentityAgent {sock}\n  IdentityFile {}\n  IdentitiesOnly yes\n\n",
            pub_path.display()
        ));
        p.info(format!("Host {hosts}"));
        with_block.push(label);
        info.hosts = fields.hosts;
        info.block = true;
        infos.push(info);
    }

    let config = paths.ssh_config();
    let text = format!(
        "# Generated by `kubaos bitwarden` from the Bitwarden SSH keys in {}. Do not edit:\n\
         # change the vault, then run kubaos bitwarden refresh. The keys stay in the\n\
         # vault; IdentityFile names the public key so the agent offers that one.\n\n{conf}",
        one_line(scope)
    );
    write_if_changed(m, &config, &text, 0o600)?;
    p.ok(format!("~/.ssh/config.d/bitwarden ({} Host block(s))", with_block.len()));

    remove_stale(m, paths, &written, p);
    let state = paths.pubkeys();
    let listed: String = written.iter().map(|f| format!("{f}\n")).collect();
    m.write(&state, &listed, 0o600).map_err(|e| Error::write(&state, e))?;

    p.summary(format!(
        "ssh    {} public key(s), {} Host block(s) in ~/.ssh/config.d/bitwarden",
        written.len(),
        with_block.len()
    ));
    if !with_block.is_empty() {
        p.summary(format!("       Host blocks: {}", list(&with_block)));
    }
    if !key_only.is_empty() {
        p.summary(format!(
            "       key only: {}. A 'hosts' field (hosts=example.com) gives a key its Host block",
            list(&key_only)
        ));
    }
    Ok(infos)
}

/// The `.pub` files the last pull wrote whose item is gone.
fn remove_stale(m: &mut dyn Machine, paths: &Paths, written: &[String], p: &mut Progress) {
    let before = m.read(&paths.pubkeys()).unwrap_or_default();
    for old in before.lines().filter(|l| safe_file(l)) {
        if written.iter().any(|w| w == old) {
            continue;
        }
        m.remove(&paths.ssh.join(old));
        p.info(format!("removed ~/.ssh/{old} (no longer in the vault)"));
    }
}

/// Writes only when the content differs, so an unchanged file keeps its mtime.
///
/// # Errors
///
/// The file can't be written.
pub fn write_if_changed(m: &mut dyn Machine, path: &std::path::Path, contents: &str, mode: u32) -> Result<(), Error> {
    if m.read(path).as_deref() == Some(contents) {
        return Ok(());
    }
    m.write(path, contents, mode).map_err(|e| Error::write(path, e))
}

/// The key a `.pub` on disk holds, for git's check that it signs with the vault's key.
pub fn on_disk(m: &mut dyn Machine, path: &std::path::Path) -> Option<String> {
    m.read(path).map(|s| key_of(&s))
}

