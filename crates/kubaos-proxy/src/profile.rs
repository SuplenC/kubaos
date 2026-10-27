//! The Proxy's profile, `~/.config/kubaos/desktop/network/proxy.toml`, and the files it names
//! beside it: the tunnel's `.conf` without its keys and the company's root certificate.
//!
//! ```toml
//! [tunnel]
//! conf = "wireguard.conf"     # written by `kubaos-proxy import`
//! kill_switch = true          # a full tunnel blocks what would leave outside it
//! vault_item = "kubaos-proxy" # the vault item with the keys
//!
//! [dns]
//! servers = ["10.0.0.53"]
//! domains = ["corp.example.com"]
//!
//! [http]
//! proxy = "http://proxy.corp.example.com:3128"
//! pac = "http://wpad.corp.example.com/proxy.pac"
//! no_proxy = [".corp.example.com"]
//!
//! [ca]
//! file = "company-ca.crt"
//! ```
//!
//! Every value is checked here, against what each place it goes to can take safely: a value in an
//! apt config, a systemd drop-in or a gsettings call has none of the characters those read
//! specially. The root helper checks them again.
//!
//! # Owns
//!
//! - Reading and checking the profile ([`Profile::load`]), its values ([`ProxyUrl`], [`pac_ok`],
//!   [`no_proxy_ok`]), and naming the tunnel's `.conf` in it ([`set_conf`]).
//!
//! # Not here
//!
//! - What a profile comes to on the machine: [`crate::plan`].

use std::fmt;
use std::fs;
use std::io;
use std::net::IpAddr;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;

use crate::wireguard;
use crate::wireguard::Config;
use crate::wireguard::ParseError;
use crate::wireguard::is_hostname;

/// The vault item with the tunnel's keys, when the profile names none.
pub const DEFAULT_VAULT_ITEM: &str = "kubaos-proxy";

/// What `no_proxy` always holds, the user's entries after it: the machine itself never goes
/// through the proxy.
pub const DEFAULT_NO_PROXY: [&str; 3] = ["localhost", "127.0.0.1", "::1"];

/// Larger than any root certificate, so a wrong file is refused before it is read whole.
const MAX_CA_BYTES: u64 = 64 * 1024;

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Raw {
    tunnel: RawTunnel,
    dns: RawDns,
    http: RawHttp,
    ca: RawCa,
}

#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct RawTunnel {
    conf: Option<String>,
    kill_switch: bool,
    vault_item: String,
}

impl Default for RawTunnel {
    fn default() -> Self {
        RawTunnel {
            conf: None,
            kill_switch: true,
            vault_item: DEFAULT_VAULT_ITEM.to_owned(),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct RawDns {
    servers: Vec<String>,
    domains: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct RawHttp {
    proxy: Option<String>,
    pac: Option<String>,
    no_proxy: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct RawCa {
    file: Option<String>,
}

/// The tunnel: IT's `.conf`, without its keys.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tunnel {
    pub file: PathBuf,
    pub config: Config,
    /// A full tunnel blocks everything that would leave outside it while it is up.
    pub kill_switch: bool,
    /// The file holds a key in plain text, which `status` warns of: the import keeps them out.
    pub keys_on_disk: bool,
}

/// The work DNS servers and the domains they answer for, the `.conf`'s first.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Dns {
    pub servers: Vec<IpAddr>,
    pub domains: Vec<String>,
}

impl Dns {
    pub fn is_empty(&self) -> bool {
        self.servers.is_empty() && self.domains.is_empty()
    }
}

/// The HTTP proxy, the PAC file, and what goes around them.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Http {
    pub proxy: Option<ProxyUrl>,
    pub pac: Option<String>,
    /// [`DEFAULT_NO_PROXY`], then the user's.
    pub no_proxy: Vec<String>,
}

impl Http {
    pub fn is_empty(&self) -> bool {
        self.proxy.is_none() && self.pac.is_none()
    }
}

/// The company's root certificate, as the file has it.
#[derive(Clone, PartialEq, Eq)]
pub struct Ca {
    pub file: PathBuf,
    pub pem: String,
}

impl fmt::Debug for Ca {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Ca").field("file", &self.file).finish_non_exhaustive()
    }
}

/// A profile, every file it names read and checked.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Profile {
    pub tunnel: Option<Tunnel>,
    pub dns: Dns,
    pub http: Http,
    pub ca: Option<Ca>,
    pub vault_item: String,
}

/// Why a profile can't be used.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum LoadError {
    #[error("reading {}", .0.display())]
    Read(PathBuf, #[source] io::Error),
    #[error("proxy.toml isn't valid TOML or has a key it doesn't know")]
    Toml(#[source] toml::de::Error),
    #[error("proxy.toml isn't valid TOML")]
    Edit(#[source] toml_edit::TomlError),
    #[error("{} isn't a WireGuard config the Proxy can use", .0.display())]
    WireGuard(PathBuf, #[source] ParseError),
    #[error("{key} has `{value}`, which isn't {wanted}")]
    Value {
        key: &'static str,
        value: String,
        wanted: &'static str,
    },
    #[error("[dns] needs the tunnel: the work DNS servers are reached through it (kubaos proxy import FILE)")]
    DnsWithoutTunnel,
    #[error("{} isn't a PEM certificate (-----BEGIN CERTIFICATE-----)", .0.display())]
    NotPem(PathBuf),
}

fn value(key: &'static str, value: &str, wanted: &'static str) -> LoadError {
    LoadError::Value {
        key,
        value: value.to_owned(),
        wanted,
    }
}

impl Profile {
    /// The profile in `network`, the user layer's network folder. No `proxy.toml` is the empty
    /// profile: nothing to switch.
    ///
    /// # Errors
    ///
    /// The file or one it names can't be read, isn't valid, or a value in it isn't what its key
    /// takes.
    pub fn load(network: &Path) -> Result<Profile, LoadError> {
        let path = network.join(crate::paths::PROFILE);
        let text = match fs::read_to_string(&path) {
            Ok(text) => text,
            Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(Profile::empty()),
            Err(err) => return Err(LoadError::Read(path, err)),
        };
        Profile::parse(&text, network)
    }

    /// The profile with no part.
    pub fn empty() -> Profile {
        Profile {
            vault_item: DEFAULT_VAULT_ITEM.to_owned(),
            ..Profile::default()
        }
    }

    /// `text` as a profile whose relative file names are in `dir`.
    ///
    /// # Errors
    ///
    /// As [`Profile::load`].
    pub fn parse(text: &str, dir: &Path) -> Result<Profile, LoadError> {
        let raw: Raw = toml::from_str(text).map_err(LoadError::Toml)?;
        let vault_item = raw.tunnel.vault_item.trim().to_owned();
        if vault_item.is_empty() || vault_item.len() > 128 || vault_item.chars().any(char::is_control) {
            return Err(value("tunnel.vault_item", &vault_item, "a vault item's name"));
        }
        let tunnel = match raw.tunnel.conf.as_deref().map(str::trim).filter(|c| !c.is_empty()) {
            Some(conf) => {
                let file = resolve(dir, conf);
                let text = fs::read_to_string(&file).map_err(|err| LoadError::Read(file.clone(), err))?;
                let whole = Config::parse(&text).map_err(|err| LoadError::WireGuard(file.clone(), err))?;
                let keys_on_disk = whole.interface.private_key.is_some()
                    || whole.peers.iter().any(|peer| peer.preshared_key.is_some());
                Some(Tunnel {
                    file,
                    config: whole.sanitized(),
                    kill_switch: raw.tunnel.kill_switch,
                    keys_on_disk,
                })
            }
            None => None,
        };
        let dns = dns(&raw.dns, tunnel.as_ref())?;
        let http = http(&raw.http)?;
        let ca = match raw.ca.file.as_deref().map(str::trim).filter(|f| !f.is_empty()) {
            Some(file) => Some(ca(&resolve(dir, file))?),
            None => None,
        };
        Ok(Profile {
            tunnel,
            dns,
            http,
            ca,
            vault_item,
        })
    }

    /// Whether the profile has any part to switch.
    pub fn is_empty(&self) -> bool {
        self.tunnel.is_none() && self.dns.is_empty() && self.http.is_empty() && self.ca.is_none()
    }

    /// Whether its tunnel is a full one.
    pub fn is_full(&self) -> bool {
        self.tunnel.as_ref().is_some_and(|tunnel| tunnel.config.is_full())
    }

    /// Whether a full tunnel blocks what would leave outside it.
    pub fn kill_switch(&self) -> bool {
        self.tunnel
            .as_ref()
            .is_some_and(|tunnel| tunnel.kill_switch && tunnel.config.is_full())
    }
}

/// `name` in `dir`, unless it is absolute or starts at the home (`~/`).
fn resolve(dir: &Path, name: &str) -> PathBuf {
    if let Some(rest) = name.strip_prefix("~/")
        && let Some(home) = std::env::var_os("HOME")
    {
        return PathBuf::from(home).join(rest);
    }
    dir.join(name)
}

fn dns(raw: &RawDns, tunnel: Option<&Tunnel>) -> Result<Dns, LoadError> {
    let mut dns = tunnel.map_or_else(Dns::default, |tunnel| Dns {
        servers: tunnel.config.interface.dns_servers.clone(),
        domains: tunnel.config.interface.dns_domains.clone(),
    });
    for server in &raw.servers {
        let addr: IpAddr = server
            .trim()
            .parse()
            .map_err(|_| value("dns.servers", server, "an IP address"))?;
        if !dns.servers.contains(&addr) {
            dns.servers.push(addr);
        }
    }
    for domain in &raw.domains {
        let name = domain
            .trim()
            .trim_start_matches('~')
            .trim_matches('.')
            .to_ascii_lowercase();
        if !is_hostname(&name) {
            return Err(value("dns.domains", domain, "a domain like corp.example.com"));
        }
        if !dns.domains.contains(&name) {
            dns.domains.push(name);
        }
    }
    if tunnel.is_none() && !(raw.servers.is_empty() && raw.domains.is_empty()) {
        return Err(LoadError::DnsWithoutTunnel);
    }
    Ok(dns)
}

fn http(raw: &RawHttp) -> Result<Http, LoadError> {
    let proxy = match raw.proxy.as_deref().map(str::trim).filter(|p| !p.is_empty()) {
        Some(url) => Some(
            ProxyUrl::parse(url)
                .ok_or_else(|| value("http.proxy", url, "a proxy like http://proxy.corp.example.com:3128"))?,
        ),
        None => None,
    };
    let pac = match raw.pac.as_deref().map(str::trim).filter(|p| !p.is_empty()) {
        Some(url) if pac_ok(url) => Some(url.to_owned()),
        Some(url) => {
            return Err(value("http.pac", url, "an http:// or https:// address of a PAC file"));
        }
        None => None,
    };
    let mut no_proxy: Vec<String> = DEFAULT_NO_PROXY.map(str::to_owned).to_vec();
    for entry in &raw.no_proxy {
        let entry = entry.trim();
        if !no_proxy_ok(entry) {
            return Err(value(
                "http.no_proxy",
                entry,
                "a host, a domain (.corp.example.com) or a range",
            ));
        }
        if !no_proxy.iter().any(|known| known == entry) {
            no_proxy.push(entry.to_owned());
        }
    }
    Ok(Http { proxy, pac, no_proxy })
}

fn ca(file: &Path) -> Result<Ca, LoadError> {
    let size = fs::metadata(file)
        .map_err(|err| LoadError::Read(file.to_path_buf(), err))?
        .len();
    if size > MAX_CA_BYTES {
        return Err(LoadError::NotPem(file.to_path_buf()));
    }
    let pem = fs::read_to_string(file).map_err(|err| LoadError::Read(file.to_path_buf(), err))?;
    if !pem.contains("-----BEGIN CERTIFICATE-----") || !pem.contains("-----END CERTIFICATE-----") {
        return Err(LoadError::NotPem(file.to_path_buf()));
    }
    Ok(Ca {
        file: file.to_path_buf(),
        pem,
    })
}

/// A proxy's address: `http://host:port` or `https://host:port`, with no user or password (they
/// would end up in world-readable files) and no path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProxyUrl {
    pub https: bool,
    /// A name, an IPv4 address, or an IPv6 address without its brackets.
    pub host: String,
    pub port: u16,
}

impl ProxyUrl {
    /// `http://proxy.corp.example.com:3128`, a `/` after it allowed. The port is required: tools
    /// disagree on the default.
    pub fn parse(text: &str) -> Option<ProxyUrl> {
        let lower = text.to_ascii_lowercase();
        let (https, rest) = if let Some(rest) = lower.strip_prefix("http://") {
            (false, rest)
        } else {
            (true, lower.strip_prefix("https://")?)
        };
        let rest = rest.strip_suffix('/').unwrap_or(rest);
        let (host, port) = rest.rsplit_once(':')?;
        if port.is_empty() || !port.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        let port: u16 = port.parse().ok().filter(|&p| p != 0)?;
        let host = match host.strip_prefix('[').and_then(|h| h.strip_suffix(']')) {
            Some(v6) => v6.parse::<std::net::Ipv6Addr>().ok()?.to_string(),
            None if host.parse::<std::net::Ipv4Addr>().is_ok() || is_hostname(host) => host.to_owned(),
            None => return None,
        };
        Some(ProxyUrl { https, host, port })
    }
}

impl fmt::Display for ProxyUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let scheme = if self.https { "https" } else { "http" };
        if self.host.contains(':') {
            write!(f, "{scheme}://[{}]:{}", self.host, self.port)
        } else {
            write!(f, "{scheme}://{}:{}", self.host, self.port)
        }
    }
}

/// A PAC file's address: `http://` or `https://`, then the characters of a URL other than quotes,
/// backslashes, spaces and control characters.
pub fn pac_ok(url: &str) -> bool {
    let lower = url.to_ascii_lowercase();
    let Some(rest) = lower.strip_prefix("http://").or_else(|| lower.strip_prefix("https://")) else {
        return false;
    };
    !rest.is_empty()
        && url.len() <= 2048
        && url
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-._~:/?#@!$&()*+,;=%".contains(&b))
}

/// A `no_proxy` entry: a host, a domain (`.corp.example.com`), an address or a range, or `*`.
pub fn no_proxy_ok(entry: &str) -> bool {
    !entry.is_empty()
        && entry.len() <= 255
        && entry
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b".:/*_-".contains(&b))
}

/// Names `conf` as the tunnel's `.conf` in the profile at `path`, keeping everything else in it,
/// comments included; a missing profile is made with that alone.
///
/// # Errors
///
/// The profile can't be read or written, or isn't valid TOML.
pub fn set_conf(path: &Path, conf: &str) -> Result<(), LoadError> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(err) if err.kind() == io::ErrorKind::NotFound => String::new(),
        Err(err) => return Err(LoadError::Read(path.to_path_buf(), err)),
    };
    let mut doc: toml_edit::DocumentMut = text.parse().map_err(LoadError::Edit)?;
    let tunnel = doc
        .entry("tunnel")
        .or_insert_with(|| toml_edit::Item::Table(toml_edit::Table::new()));
    let Some(tunnel) = tunnel.as_table_like_mut() else {
        return Err(value("tunnel", "a plain value", "a table, [tunnel]"));
    };
    tunnel.insert("conf", toml_edit::value(conf));
    write_atomic(path, &doc.to_string(), 0o644).map_err(|err| LoadError::Read(path.to_path_buf(), err))
}

/// Writes `contents` to `path` whole or not at all, with `mode`, making its folder.
///
/// # Errors
///
/// The folder or the file can't be written.
pub fn write_atomic(path: &Path, contents: &str, mode: u32) -> io::Result<()> {
    use std::io::Write as _;
    use std::os::unix::fs::OpenOptionsExt as _;
    use std::os::unix::fs::PermissionsExt as _;

    let dir = path.parent().ok_or_else(|| io::Error::other("no folder"))?;
    fs::create_dir_all(dir)?;
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let tmp = dir.join(format!(".{name}.{}.tmp", std::process::id()));
    let _ = fs::remove_file(&tmp);
    let written = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(mode)
        .open(&tmp)
        .and_then(|mut file| {
            file.write_all(contents.as_bytes())?;
            file.set_permissions(fs::Permissions::from_mode(mode))?;
            file.sync_all()
        })
        .and_then(|()| fs::rename(&tmp, path));
    if written.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    written
}

/// Whether `file` holds a WireGuard config: what `import` checks before it touches the vault.
///
/// # Errors
///
/// The file can't be read or isn't a WireGuard config.
pub fn read_conf(file: &Path) -> Result<wireguard::Config, LoadError> {
    let text = fs::read_to_string(file).map_err(|err| LoadError::Read(file.to_path_buf(), err))?;
    Config::parse(&text).map_err(|err| LoadError::WireGuard(file.to_path_buf(), err))
}

#[cfg(test)]
mod tests;
