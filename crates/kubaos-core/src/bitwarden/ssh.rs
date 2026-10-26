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

/// Why an item's public key can't be written to a `.pub` file.
#[derive(Debug, PartialEq, Eq)]
pub enum KeyProblem {
    Empty,
    /// A newline or another control character, after `before`: a `.pub` file is one line, and
    /// the same text goes into `allowed_signers`.
    Control { before: String, newline: bool },
    /// A type Bitwarden's agent doesn't serve.
    Kind(String),
    /// The second word isn't base64, or is missing.
    Blob,
}

/// The key of a public key line, `<type> <base64>`, from `<type> <base64> [comment]` with any
/// run of spaces between the words. The type is `ssh-ed25519`, `ssh-rsa` or
/// `ecdsa-sha2-nistpNNN`. The comment is dropped: the `.pub` gets a comment of its own, and keys
/// are compared without one.
///
/// # Errors
///
/// What is wrong with the line.
pub fn public_key(k: &str) -> Result<String, KeyProblem> {
    // A key pasted into Bitwarden often keeps the line ending it was copied with; at the very
    // end it can't carry anything into ssh_config, so it goes. One mid-line still can.
    let k = k.trim_end();
    if let Some((before, newline)) = control(k) {
        return Err(KeyProblem::Control { before, newline });
    }
    let mut words = k.split(' ').filter(|w| !w.is_empty());
    let Some(kind) = words.next() else {
        return Err(KeyProblem::Empty);
    };
    let kind_ok = kind == "ssh-ed25519"
        || kind == "ssh-rsa"
        || kind
            .strip_prefix("ecdsa-sha2-nistp")
            .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()));
    if !kind_ok {
        return Err(KeyProblem::Kind(kind.to_string()));
    }
    let blob = words.next().unwrap_or("");
    if blob.is_empty() || !blob.bytes().all(|b| b.is_ascii_alphanumeric() || b"+/=".contains(&b)) {
        return Err(KeyProblem::Blob);
    }
    Ok(format!("{kind} {blob}"))
}

/// The text before the first control character, and whether that character ends a line.
fn control(s: &str) -> Option<(String, bool)> {
    let at = s.find(char::is_control)?;
    let newline = s[at..].starts_with(['\n', '\r']);
    Some((s[..at].trim().to_string(), newline))
}

/// The start of a value for a message, short enough for one line of the summary.
fn preview(s: &str) -> String {
    const SHOWN: usize = 24;
    let s = one_line(s);
    match s.char_indices().nth(SHOWN) {
        Some((at, _)) => format!("{}…", &s[..at]),
        None => s,
    }
}

/// The warning for a key that gets no `.pub`, with the fix in Bitwarden.
fn key_warning(label: &str, problem: &KeyProblem) -> String {
    const SKIPPED: &str = "Skipped: no .pub, no Host block";
    match problem {
        KeyProblem::Empty => format!(
            "{label}: the public key field is empty; in Bitwarden, import the key again or paste its .pub line. {SKIPPED}"
        ),
        KeyProblem::Control { before, newline } => {
            let what = if *newline { "a newline" } else { "a control character" };
            format!(
                "{label}: the public key field holds `{}` followed by {what}; in Bitwarden, make the public key \
                 that one line (or import the key again). {SKIPPED}",
                preview(before)
            )
        }
        KeyProblem::Kind(kind) => format!(
            "{label}: the public key is of type `{}`; kubaos bitwarden takes ssh-ed25519, ssh-rsa and \
             ecdsa-sha2-nistp*. Create an Ed25519 key in Bitwarden instead. {SKIPPED}",
            preview(kind)
        ),
        KeyProblem::Blob => format!(
            "{label}: the public key's second word isn't base64, so it isn't a key; in Bitwarden, import the key \
             again or paste its .pub line (`ssh-ed25519 AAAA… comment`). {SKIPPED}"
        ),
    }
}

/// The key of a `.pub` file, without its comment; the file as it is when it holds no key.
fn key_of(pub_file: &str) -> String {
    let line = pub_file.trim_end();
    public_key(line).unwrap_or_else(|_| line.to_string())
}

/// The custom fields an SSH key item can carry.
const KNOWN_FIELDS: [&str; 7] = ["hosts", "hostname", "user", "port", "signing", "emails", "file"];

/// The known field an unknown name was most likely meant to be: a common synonym, or a name
/// one or two typos away (one for short names, where two would match half the list).
fn meant_field(name: &str) -> Option<&'static str> {
    let name = name.trim().to_ascii_lowercase();
    if KNOWN_FIELDS.contains(&name.as_str()) {
        return None;
    }
    let synonym = match name.as_str() {
        "username" | "login" => Some("user"),
        "host_name" | "host-name" | "address" | "server" => Some("hostname"),
        "email" | "mail" => Some("emails"),
        "sign" | "signingkey" => Some("signing"),
        "filename" | "identityfile" => Some("file"),
        _ => None,
    };
    if synonym.is_some() {
        return synonym;
    }
    let most = if name.chars().count() <= 4 { 1 } else { 2 };
    KNOWN_FIELDS
        .iter()
        .map(|k| (distance(&name, k), *k))
        .filter(|(d, _)| *d <= most)
        .min_by_key(|(d, _)| *d)
        .map(|(_, k)| k)
}

/// The Levenshtein distance: insertions, deletions and substitutions.
fn distance(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut diag = row[0];
        row[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let next = (diag + usize::from(ca != *cb)).min(row[j] + 1).min(row[j + 1] + 1);
            diag = row[j + 1];
            row[j + 1] = next;
        }
    }
    row[b.len()]
}

/// Warns about each custom field whose name looks like a misspelt known one.
fn warn_misspelt_fields(item: &Item, label: &str, p: &mut Progress) {
    for f in &item.fields {
        if let Some(known) = meant_field(&f.name) {
            p.warn(format!(
                "{label}: kubaos bitwarden reads no field `{}`; in Bitwarden, rename it to `{known}`",
                preview(f.name.trim())
            ));
        }
    }
}

/// A Host pattern ssh can't resolve as it stands: no dot or colon, so neither a domain nor an
/// address, and no wildcard.
fn bare_alias(host: &str) -> bool {
    host != "localhost" && !host.contains(['.', ':', '*', '?', '!', '[', '%'])
}

/// The `hostname` an alias most likely stands for, for the fix in its warning.
fn likely_hostname(alias: &str) -> &'static str {
    let alias = alias.to_ascii_lowercase();
    [("github", "github.com"), ("gitlab", "gitlab.com"), ("bitbucket", "bitbucket.org"), ("codeberg", "codeberg.org")]
        .into_iter()
        .find(|(word, _)| alias.contains(word))
        .map_or("the server's name or address", |(_, host)| host)
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
    /// A control character in a field: a newline plus `ProxyCommand` would run commands.
    Control { field: &'static str, before: String, newline: bool },
    NoHosts,
    /// Characters ssh_config can't take safely in a field, and the ones it takes there.
    Unsafe { field: &'static str, value: String, allowed: &'static str },
}

fn host_fields(item: &Item) -> Result<HostFields, NoBlock> {
    let names = ["hosts", "hostname", "user", "port"];
    for field in names {
        if let Some((before, newline)) = control(item.field(field)) {
            return Err(NoBlock::Control { field, before, newline });
        }
    }
    let [hosts, hostname, user, port] = names.map(|f| item.field(f).trim());
    let hosts: Vec<String> = hosts
        .split(|c: char| c == ',' || c.is_whitespace())
        .filter(|w| !w.is_empty())
        .map(str::to_string)
        .collect();
    if hosts.is_empty() {
        return Err(NoBlock::NoHosts);
    }
    let all = |s: &str, ok: fn(char) -> bool| s.chars().all(ok);
    let unsafe_in = |field, value: &str, allowed| NoBlock::Unsafe { field, value: value.to_string(), allowed };
    if let Some(h) = hosts.iter().find(|h| !all(h, |c| c.is_ascii_alphanumeric() || "[].*?!_:%-".contains(c))) {
        return Err(unsafe_in("hosts", h, "letters, digits and . - _ : % * ? ! [ ]"));
    }
    if !all(hostname, |c| c.is_ascii_alphanumeric() || "._:%-".contains(c)) {
        return Err(unsafe_in("hostname", hostname, "letters, digits and . - _ : %"));
    }
    if !all(user, |c| c.is_ascii_alphanumeric() || "._@-".contains(c)) {
        return Err(unsafe_in("user", user, "letters, digits and . _ @ -"));
    }
    if !(port.is_empty() || (port.len() <= 5 && all(port, |c| c.is_ascii_digit()))) {
        return Err(unsafe_in("port", port, "a number"));
    }
    Ok(HostFields {
        hosts,
        hostname: hostname.to_string(),
        user: user.to_string(),
        port: port.to_string(),
    })
}

/// A Host block to write: the item it comes from and its checked fields.
struct Block {
    label: String,
    fingerprint: String,
    fields: HostFields,
    pub_path: std::path::PathBuf,
}

impl Block {
    fn has_wildcard(&self) -> bool {
        self.fields.hosts.iter().any(|h| wildcard(h))
    }
}

/// A Host pattern that can match more than one name, or that leaves names out.
fn wildcard(host: &str) -> bool {
    host.contains(['*', '?', '!', '['])
}

/// The text of the Host blocks: those with only specific patterns first, then those with a
/// wildcard, each group in vault order.
///
/// ssh takes `IdentityFile` from every block that matches, in file order, so a `Host *` block
/// written first has its key offered first on every alias, and a server that knows both keys
/// (GitHub with two accounts) logs in as the default one. A wildcard block also leaves out every
/// other block's specific hosts with `!host`, so an alias offers its own key alone: offered
/// second, the default key would still log in as the wrong account whenever the alias's own key
/// is refused.
fn host_blocks(blocks: &[Block], sock: &str) -> String {
    let (specific, wild): (Vec<&Block>, Vec<&Block>) = blocks.iter().partition(|b| !b.has_wildcard());
    let mut conf = String::new();
    for block in specific.into_iter().chain(wild) {
        let mut patterns = block.fields.hosts.clone();
        if block.has_wildcard() {
            let others = blocks.iter().filter(|b| !std::ptr::eq(*b, block));
            for host in others.flat_map(|b| &b.fields.hosts).filter(|h| !wildcard(h)) {
                let left_out = format!("!{host}");
                if !block.fields.hosts.contains(host) && !patterns.contains(&left_out) {
                    patterns.push(left_out);
                }
            }
        }
        let f = &block.fields;
        let mut lines = vec![
            format!("# {} ({})", block.label, block.fingerprint),
            format!("Host {}", patterns.join(" ")),
        ];
        for (k, v) in [("HostName", &f.hostname), ("User", &f.user), ("Port", &f.port)] {
            if !v.is_empty() {
                lines.push(format!("  {k} {v}"));
            }
        }
        lines.push(format!("  IdentityAgent {sock}"));
        lines.push(format!("  IdentityFile {}", block.pub_path.display()));
        lines.push("  IdentitiesOnly yes\n\n".to_owned());
        conf.push_str(&lines.join("\n"));
    }
    conf
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
    let mut blocks = vec![];
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
        warn_misspelt_fields(item, &label, p);
        let public = match public_key(item.public_key.as_deref().unwrap_or("")) {
            Ok(k) => k,
            Err(problem) => {
                p.warn(key_warning(&label, &problem));
                info.note = Some("no usable public key in the item".into());
                infos.push(info);
                continue;
            }
        };
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
            Err(NoBlock::Control { field, before, newline }) => {
                let what = if newline { "a newline" } else { "a control character" };
                p.warn(format!(
                    "{label}: the field `{field}` holds `{}` followed by {what}, and ssh_config would read the rest \
                     as directives; in Bitwarden, make the value one line. No Host block",
                    preview(&before)
                ));
                info.note = Some(format!("the field `{field}` has {what}"));
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
            Err(NoBlock::Unsafe { field, value, allowed }) => {
                p.warn(format!(
                    "{label}: the field `{field}` holds `{}`, with characters ssh_config can't take safely \
                     (it takes {allowed}); fix the value in Bitwarden. No Host block",
                    preview(&value)
                ));
                info.note = Some(format!("the field `{field}` has characters ssh_config can't take"));
                infos.push(info);
                continue;
            }
        };
        // Without `HostName`, ssh resolves the alias itself. The block is still written: a name
        // the resolver knows (a LAN name, an `/etc/hosts` entry) works as it is, and the key and
        // user apply either way.
        if fields.hostname.is_empty()
            && let Some(alias) = fields.hosts.iter().find(|h| bare_alias(h))
        {
            p.warn(format!(
                "{label}: Host {alias} has no `hostname` field, so ssh looks up \"{alias}\" as a server name and \
                 fails; in Bitwarden, add a custom text field `hostname` = {}",
                likely_hostname(alias)
            ));
        }
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
        p.info(format!("Host {}", fields.hosts.join(" ")));
        info.hosts.clone_from(&fields.hosts);
        info.block = true;
        infos.push(info);
        blocks.push(Block { label: label.clone(), fingerprint: fp, fields, pub_path });
        with_block.push(label);
    }
    let conf = host_blocks(&blocks, &sock);

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

