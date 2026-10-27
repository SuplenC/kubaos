//! The tunnel's keys: in the Bitwarden vault, the one place they are kept, and copied into the
//! GNOME keyring, where `on` reads them without bw or the master password, as `use bwenv` reads
//! a project's variables.
//!
//! In the vault, one secure note (`kubaos-proxy`, or the profile's `vault_item`) holds them as
//! hidden fields: `private-key`, and `preshared-key <peer's public key>` for each peer that has
//! one. In the keyring, each is an entry with the attributes `service=kubaos-proxy`,
//! `key=private-key|preshared-key` and, for a preshared key, `peer=<public key>`.
//!
//! A key goes to `bw` and `secret-tool` on stdin, never in their argv, where every process on the
//! machine could read it; the vault session goes in bw's environment alone.
//!
//! # Owns
//!
//! - [`Keys`], storing and reading them in the keyring ([`store`], [`load`], [`forget`]) and in the
//!   vault ([`to_vault`], [`from_vault`]).
//!
//! # Not here
//!
//! - Unlocking the vault: `kubaos_core::bitwarden`, through `main.rs`.

use std::fmt;

use kubaos_core::bitwarden::Session;
use kubaos_core::bitwarden::base64;
use serde_json::Value;
use serde_json::json;

use crate::machine::Cmd;
use crate::machine::Machine;
use crate::machine::Ran;
use crate::machine::Said;
use crate::machine::StartError;
use crate::wireguard::Config;
use crate::wireguard::Key;
use crate::wireguard::Secret;

/// The keyring's service attribute for the tunnel's keys.
pub const SERVICE: &str = "kubaos-proxy";
/// The vault field and keyring entry of the private key.
pub const PRIVATE: &str = "private-key";
/// The vault fields' and keyring entries' name of a preshared key, the peer after it.
pub const PRESHARED: &str = "preshared-key";
/// Bitwarden's item type of a secure note, and its field type of a hidden field.
const SECURE_NOTE: u64 = 2;
const HIDDEN: u64 = 1;
/// Where `system/70-bitwarden.sh` installs bw.
const BW: &str = "/usr/local/bin/bw";

/// The private key, and each peer's preshared key.
#[derive(Clone, PartialEq, Eq)]
pub struct Keys {
    pub private: Secret,
    pub preshared: Vec<(Key, Secret)>,
}

impl fmt::Debug for Keys {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Keys")
            .field(
                "preshared_for",
                &self.preshared.iter().map(|(peer, _)| peer).collect::<Vec<_>>(),
            )
            .finish_non_exhaustive()
    }
}

impl Keys {
    /// The keys of a whole `.conf`; none without its private key.
    pub fn of(config: &Config) -> Option<Keys> {
        Some(Keys {
            private: config.interface.private_key.clone()?,
            preshared: config
                .peers
                .iter()
                .filter_map(|peer| Some((peer.public_key, peer.preshared_key.clone()?)))
                .collect(),
        })
    }

    pub fn preshared_for(&self, peer: &Key) -> Option<&Secret> {
        self.preshared
            .iter()
            .find(|(key, _)| key == peer)
            .map(|(_, secret)| secret)
    }
}

/// Why the keys couldn't be stored or read.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    #[error("the keyring has no key for the tunnel: run kubaos proxy pull (or import IT's config again)")]
    NotInKeyring,
    #[error("the GNOME keyring couldn't be reached")]
    Keyring(#[source] Said),
    #[error("bw isn't installed: run system/70-bitwarden.sh")]
    NoBw,
    #[error("the vault refused")]
    Vault(#[source] Said),
    #[error("the vault has no item {0:?} with the tunnel's keys: import IT's config (kubaos proxy import FILE)")]
    NotInVault(String),
    #[error("the vault item {0:?} has no valid private-key field")]
    BadItem(String),
}

/// The stdout of a run that went well, else what the program said.
fn output(program: &str, ran: Result<Ran, StartError>) -> Result<String, Said> {
    match ran {
        Ok(ran) if ran.ok() => Ok(ran.stdout),
        Ok(ran) => {
            let said = ran.stderr.trim();
            Err(Said(if said.is_empty() {
                format!("{program} failed")
            } else {
                said.to_owned()
            }))
        }
        Err(err) => Err(err.said),
    }
}

fn store_one(m: &mut dyn Machine, label: &str, attrs: &[&str], secret: &Secret) -> Result<(), Error> {
    let cmd = Cmd::new("secret-tool")
        .arg("store")
        .arg(format!("--label={label}"))
        .args(["service", SERVICE])
        .args(attrs.iter().copied())
        .secret_stdin(secret.expose());
    output("secret-tool", m.run(&cmd)).map(drop).map_err(Error::Keyring)
}

/// Puts `keys` in the keyring in place of the ones there.
///
/// # Errors
///
/// The keyring refused one; the ones before it stay.
pub fn store(m: &mut dyn Machine, keys: &Keys) -> Result<(), Error> {
    forget(m);
    store_one(
        m,
        "KubaOS Proxy: the tunnel's private key",
        &["key", PRIVATE],
        &keys.private,
    )?;
    for (peer, secret) in &keys.preshared {
        let peer = peer.to_string();
        store_one(
            m,
            &format!("KubaOS Proxy: the preshared key with {peer}"),
            &["key", PRESHARED, "peer", &peer],
            secret,
        )?;
    }
    Ok(())
}

/// Forgets every key of the tunnel in the keyring.
pub fn forget(m: &mut dyn Machine) {
    let _ = m.run(&Cmd::new("secret-tool").args(["clear", "service", SERVICE]));
}

/// One secret, `None` when the keyring has none: secret-tool exits 1 and says nothing then.
fn lookup(m: &mut dyn Machine, attrs: &[&str]) -> Result<Option<Secret>, Error> {
    let cmd = Cmd::new("secret-tool")
        .args(["lookup", "service", SERVICE])
        .args(attrs.iter().copied());
    let ran = m.run(&cmd).map_err(|err| Error::Keyring(err.said))?;
    if ran.code == Some(1) && ran.stdout.is_empty() && ran.stderr.trim().is_empty() {
        return Ok(None);
    }
    let text = output("secret-tool", Ok(ran)).map_err(Error::Keyring)?;
    Ok(Secret::parse(text.trim()))
}

/// The keys of `config`'s peers from the keyring; a peer without a preshared key there has none.
///
/// # Errors
///
/// The keyring can't be read, or has no private key.
pub fn load(m: &mut dyn Machine, config: &Config) -> Result<Keys, Error> {
    let private = lookup(m, &["key", PRIVATE])?.ok_or(Error::NotInKeyring)?;
    let mut preshared = Vec::new();
    for peer in &config.peers {
        let key = peer.public_key.to_string();
        if let Some(secret) = lookup(m, &["key", PRESHARED, "peer", &key])? {
            preshared.push((peer.public_key, secret));
        }
    }
    Ok(Keys { private, preshared })
}

/// bw, where `system/70-bitwarden.sh` installs it, else the one on the PATH.
fn bw() -> &'static str {
    if std::path::Path::new(BW).exists() { BW } else { "bw" }
}

fn bw_cmd(session: &Session, args: &[&str]) -> Cmd {
    Cmd::new(bw())
        .args(args.iter().copied())
        .env("BW_NOINTERACTION", "true")
        .env("BW_SESSION", session.expose())
}

/// The vault's items named `name` exactly.
fn items_named(m: &mut dyn Machine, session: &Session, name: &str) -> Result<Vec<Value>, Error> {
    let out = output("bw", m.run(&bw_cmd(session, &["list", "items", "--search", name]))).map_err(Error::Vault)?;
    let items: Vec<Value> = serde_json::from_str(&out).map_err(|err| Error::Vault(Said(err.to_string())))?;
    Ok(items
        .into_iter()
        .filter(|item| item.get("name").and_then(Value::as_str) == Some(name))
        .collect())
}

/// The item's fields with `keys` in place of the tunnel's old ones, the others kept.
fn with_keys(fields: Option<&Value>, keys: &Keys) -> Vec<Value> {
    let mut out: Vec<Value> = fields
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|field| {
            let name = field.get("name").and_then(Value::as_str).unwrap_or("");
            name != PRIVATE && !name.starts_with(PRESHARED)
        })
        .cloned()
        .collect();
    out.push(json!({ "name": PRIVATE, "value": keys.private.expose(), "type": HIDDEN }));
    for (peer, secret) in &keys.preshared {
        out.push(json!({ "name": format!("{PRESHARED} {peer}"), "value": secret.expose(), "type": HIDDEN }));
    }
    out
}

/// Puts `keys` in the vault, in the secure note `name`, made when there is none.
///
/// # Errors
///
/// bw failed; the vault is as it was.
pub fn to_vault(m: &mut dyn Machine, session: &Session, name: &str, keys: &Keys) -> Result<(), Error> {
    let found = items_named(m, session, name)?;
    let (args, item) = if let Some(mut item) = found.into_iter().next() {
        let id = item.get("id").and_then(Value::as_str).unwrap_or_default().to_owned();
        let fields = with_keys(item.get("fields"), keys);
        item["fields"] = Value::Array(fields);
        (vec!["edit".to_owned(), "item".to_owned(), id], item)
    } else {
        let item = json!({
            "type": SECURE_NOTE,
            "name": name,
            "notes": "The WireGuard keys of KubaOS's Proxy (kubaos proxy import).",
            "secureNote": { "type": 0 },
            "fields": with_keys(None, keys),
            "favorite": false,
            "reprompt": 0,
        });
        (vec!["create".to_owned(), "item".to_owned()], item)
    };
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let encoded = base64::encode(item.to_string().as_bytes());
    let cmd = bw_cmd(session, &args).secret_stdin(&encoded);
    output("bw", m.run(&cmd)).map(drop).map_err(Error::Vault)
}

/// The keys in the vault's secure note `name`.
///
/// # Errors
///
/// bw failed, or the vault has no such item, or it has no private key.
pub fn from_vault(m: &mut dyn Machine, session: &Session, name: &str) -> Result<Keys, Error> {
    let item = items_named(m, session, name)?
        .into_iter()
        .next()
        .ok_or_else(|| Error::NotInVault(name.to_owned()))?;
    let fields = item
        .get("fields")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let field = |field: &Value| {
        (
            field
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("")
                .trim()
                .to_owned(),
            field.get("value").and_then(Value::as_str).unwrap_or("").to_owned(),
        )
    };
    let mut private = None;
    let mut preshared = Vec::new();
    for (key, value) in fields.iter().map(field) {
        if key == PRIVATE {
            private = Secret::parse(&value);
        } else if let Some(peer) = key.strip_prefix(PRESHARED).map(str::trim)
            && let (Some(peer), Some(secret)) = (Key::parse(peer), Secret::parse(&value))
        {
            preshared.push((peer, secret));
        }
    }
    let private = private.ok_or_else(|| Error::BadItem(name.to_owned()))?;
    Ok(Keys { private, preshared })
}

#[cfg(test)]
mod tests;
