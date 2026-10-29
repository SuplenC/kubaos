//! The VPN's secrets: in the Bitwarden vault, the one place they are kept, and copied into the
//! GNOME keyring, where `on` reads them without bw or the master password, as `use bwenv` reads
//! a project's variables.
//!
//! In the vault, one secure note (`kubaos-proxy`, or the profile's `vault_item`) holds them as
//! hidden fields. A WireGuard VPN's are `private-key`, and `preshared-key <peer's public key>` for
//! each peer that has one; an OpenVPN one's are named as [`Credentials::NAMES`] (`cert`, `key`,
//! `tls-auth`, `username`, `password`...), each it has. In the keyring, each is an entry with the
//! attributes `service=kubaos-proxy`, `key=<its name>` and, for a preshared key,
//! `peer=<public key>`.
//!
//! A secret goes to `bw` and `secret-tool` on stdin, never in their argv, where every process on
//! the machine could read it; the vault session goes in bw's environment alone.
//!
//! # Owns
//!
//! - [`Secret`], [`Secrets`] and WireGuard's [`Keys`], storing and reading them in the keyring
//!   ([`store`], [`load`], [`forget`]) and in the vault ([`to_vault`], [`from_vault`]).
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
use crate::openvpn::Credentials;
use crate::vpn;
use crate::wireguard;
use crate::wireguard::Key;

/// The keyring's service attribute for the VPN's secrets.
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

/// A secret: a key, a certificate, a password. It never prints.
#[derive(Clone, PartialEq, Eq)]
pub struct Secret(String);

impl Secret {
    #[must_use]
    pub fn new(text: impl Into<String>) -> Secret {
        Secret(text.into())
    }

    /// The secret as it is, for the vault, the keyring and NetworkManager alone.
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Secret(<redacted>)")
    }
}

/// The secrets of a VPN, by its type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Secrets {
    WireGuard(Keys),
    OpenVpn(Credentials),
}

/// A WireGuard VPN's private key, and each peer's preshared key.
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
    pub fn of(config: &wireguard::Config) -> Option<Keys> {
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

/// Why the secrets couldn't be stored or read.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// Only reading the secrets for the VPN meets it, so it says "its", after the words naming it.
    #[error("its secrets aren't in the keyring: run kubaos proxy pull (or import IT's config again)")]
    NotInKeyring,
    #[error("the GNOME keyring couldn't be reached")]
    Keyring(#[source] Said),
    #[error("bw isn't installed: run system/70-bitwarden.sh")]
    NoBw,
    #[error("the vault refused")]
    Vault(#[source] Said),
    #[error("the vault has no item {0:?} with the VPN's secrets: import IT's config (kubaos proxy import FILE)")]
    NotInVault(String),
    #[error("the vault item {0:?} has neither a valid private-key field nor an OpenVPN certificate or password")]
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

/// Puts `secrets` in the keyring in place of the ones there.
///
/// # Errors
///
/// The keyring refused one; the ones before it stay.
pub fn store(m: &mut dyn Machine, secrets: &Secrets) -> Result<(), Error> {
    forget(m);
    match secrets {
        Secrets::WireGuard(keys) => {
            store_one(
                m,
                "KubaOS Proxy: the VPN's private key",
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
        }
        Secrets::OpenVpn(credentials) => {
            for (name, secret) in credentials.iter() {
                store_one(m, &format!("KubaOS Proxy: the VPN's {name}"), &["key", name], secret)?;
            }
        }
    }
    Ok(())
}

/// Forgets every secret of the VPN in the keyring.
pub fn forget(m: &mut dyn Machine) {
    let _ = m.run(&Cmd::new("secret-tool").args(["clear", "service", SERVICE]));
}

/// One secret as the keyring holds it, `None` when it has none: secret-tool exits 1 and says
/// nothing then.
fn lookup(m: &mut dyn Machine, attrs: &[&str]) -> Result<Option<String>, Error> {
    let cmd = Cmd::new("secret-tool")
        .args(["lookup", "service", SERVICE])
        .args(attrs.iter().copied());
    let ran = m.run(&cmd).map_err(|err| Error::Keyring(err.said))?;
    if ran.code == Some(1) && ran.stdout.is_empty() && ran.stderr.trim().is_empty() {
        return Ok(None);
    }
    output("secret-tool", Ok(ran)).map(Some).map_err(Error::Keyring)
}

/// The secrets `config` takes, from the keyring: a WireGuard peer without a preshared key there
/// has none, and an OpenVPN config gets each secret the keyring has.
///
/// # Errors
///
/// The keyring can't be read, or lacks what the VPN can't come up without: WireGuard's private
/// key, OpenVPN's certificate and key or its username and password.
pub fn load(m: &mut dyn Machine, config: &vpn::Config) -> Result<Secrets, Error> {
    match config {
        vpn::Config::WireGuard(config) => {
            let private = lookup(m, &["key", PRIVATE])?
                .as_deref()
                .and_then(|text| wireguard::secret(text.trim()))
                .ok_or(Error::NotInKeyring)?;
            let mut preshared = Vec::new();
            for peer in &config.peers {
                let key = peer.public_key.to_string();
                let found = lookup(m, &["key", PRESHARED, "peer", &key])?;
                if let Some(secret) = found.as_deref().and_then(|text| wireguard::secret(text.trim())) {
                    preshared.push((peer.public_key, secret));
                }
            }
            Ok(Secrets::WireGuard(Keys { private, preshared }))
        }
        vpn::Config::OpenVpn(config) => {
            let mut credentials = Credentials::default();
            for name in Credentials::NAMES {
                if let Some(text) = lookup(m, &["key", name])? {
                    credentials.set(name, Secret::new(text));
                }
            }
            if !credentials.missing(config).is_empty() {
                return Err(Error::NotInKeyring);
            }
            Ok(Secrets::OpenVpn(credentials))
        }
    }
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

/// Whether a vault field is one of the VPN's, of either type.
fn is_ours(name: &str) -> bool {
    name == PRIVATE || name.starts_with(PRESHARED) || Credentials::NAMES.contains(&name)
}

/// The item's fields with `secrets` in place of the VPN's old ones, the others kept.
fn with_secrets(fields: Option<&Value>, secrets: &Secrets) -> Vec<Value> {
    let mut out: Vec<Value> = fields
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|field| !is_ours(field.get("name").and_then(Value::as_str).unwrap_or("").trim()))
        .cloned()
        .collect();
    let mut hidden = |name: String, secret: &Secret| {
        out.push(json!({ "name": name, "value": secret.expose(), "type": HIDDEN }));
    };
    match secrets {
        Secrets::WireGuard(keys) => {
            hidden(PRIVATE.to_owned(), &keys.private);
            for (peer, secret) in &keys.preshared {
                hidden(format!("{PRESHARED} {peer}"), secret);
            }
        }
        Secrets::OpenVpn(credentials) => {
            for (name, secret) in credentials.iter() {
                hidden(name.to_owned(), secret);
            }
        }
    }
    out
}

/// Puts `secrets` in the vault, in the secure note `name`, made when there is none.
///
/// # Errors
///
/// bw failed; the vault is as it was.
pub fn to_vault(m: &mut dyn Machine, session: &Session, name: &str, secrets: &Secrets) -> Result<(), Error> {
    let found = items_named(m, session, name)?;
    let (args, item) = if let Some(mut item) = found.into_iter().next() {
        let id = item.get("id").and_then(Value::as_str).unwrap_or_default().to_owned();
        let fields = with_secrets(item.get("fields"), secrets);
        item["fields"] = Value::Array(fields);
        (vec!["edit".to_owned(), "item".to_owned(), id], item)
    } else {
        let item = json!({
            "type": SECURE_NOTE,
            "name": name,
            "notes": "The VPN secrets of KubaOS's Proxy (kubaos proxy import).",
            "secureNote": { "type": 0 },
            "fields": with_secrets(None, secrets),
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

/// The secrets in the vault's secure note `name`: WireGuard's when it has a private key, else
/// OpenVPN's.
///
/// # Errors
///
/// bw failed, or the vault has no such item, or it holds neither a WireGuard private key nor an
/// OpenVPN certificate and key or username and password.
pub fn from_vault(m: &mut dyn Machine, session: &Session, name: &str) -> Result<Secrets, Error> {
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
    let mut credentials = Credentials::default();
    for (key, value) in fields.iter().map(field) {
        if key == PRIVATE {
            private = wireguard::secret(&value);
        } else if let Some(peer) = key.strip_prefix(PRESHARED).map(str::trim)
            && let (Some(peer), Some(secret)) = (Key::parse(peer), wireguard::secret(&value))
        {
            preshared.push((peer, secret));
        } else if !value.is_empty() {
            credentials.set(&key, Secret::new(value));
        }
    }
    if let Some(private) = private {
        return Ok(Secrets::WireGuard(Keys { private, preshared }));
    }
    let pair = credentials.cert.is_some() && credentials.key.is_some();
    let login = credentials.username.is_some() && credentials.password.is_some();
    if pair || login {
        return Ok(Secrets::OpenVpn(credentials));
    }
    Err(Error::BadItem(name.to_owned()))
}

#[cfg(test)]
mod tests;
