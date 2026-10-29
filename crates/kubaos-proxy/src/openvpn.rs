//! An OpenVPN client config as IT hands it out, a `.ovpn`: directives a line each, and the
//! certificates and keys either inline (`<cert>…</cert>`) or in files it names beside it.
//!
//! The client's certificate and key, the `tls-auth`, `tls-crypt` and `tls-crypt-v2` keys, and a
//! username and password are secrets ([`Credentials`]). They are read into [`Secret`]s, which never
//! print, and an error names the line and the directive but never the value. [`Config::sanitized`]
//! is the same config without them and with the server's CA inline: what the user layer keeps.
//!
//! NetworkManager's OpenVPN plugin runs it, so only what the plugin takes is kept: the servers, the
//! CA, the ciphers and the checks on the server's certificate, compression, the MTU and the
//! timers. The rest (`up` scripts, `verb`, `persist-key`...) is left out, and the import names what
//! of it could matter ([`Config::ignored`]).
//!
//! # Owns
//!
//! - Parsing and checking a `.ovpn` ([`Config::parse`]), its servers ([`Remote`], [`Proto`]), its
//!   secrets ([`Credentials`]), whether it is a full tunnel ([`Config::is_full`]), and what of it
//!   goes to the plugin ([`Config::data`]).
//!
//! # Not here
//!
//! - Where the secrets go: [`crate::secrets`]. What NetworkManager is given: [`crate::tunnel`].

use std::fmt;
use std::fs;
use std::io;
use std::io::Read as _;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::path::Path;
use std::path::PathBuf;

use crate::net::Cidr;
use crate::net::Endpoint;
use crate::net::is_hostname;
use crate::secrets::Secret;

/// OpenVPN's port when the config names none: IANA's.
pub const DEFAULT_PORT: u16 = 1194;
/// Larger than any certificate or key, so a wrong file is refused before it is read whole.
const MAX_FILE_BYTES: u64 = 64 * 1024;
/// More servers than any config lists: the kill switch lets each out by its address.
pub const MAX_REMOTES: usize = 16;
/// Longer than any cipher list or certificate name a directive takes.
const MAX_ARG: usize = 256;

/// How a server is reached.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Proto {
    Udp,
    Udp4,
    Udp6,
    Tcp,
    Tcp4,
    Tcp6,
}

impl Proto {
    /// OpenVPN's names, `tcp-client` as `tcp` (a client always is one).
    pub fn parse(text: &str) -> Option<Proto> {
        Some(match text.to_ascii_lowercase().as_str() {
            "udp" => Proto::Udp,
            "udp4" => Proto::Udp4,
            "udp6" => Proto::Udp6,
            "tcp" | "tcp-client" => Proto::Tcp,
            "tcp4" | "tcp4-client" => Proto::Tcp4,
            "tcp6" | "tcp6-client" => Proto::Tcp6,
            _ => return None,
        })
    }

    /// The name NetworkManager's plugin takes.
    pub const fn name(self) -> &'static str {
        match self {
            Proto::Udp => "udp",
            Proto::Udp4 => "udp4",
            Proto::Udp6 => "udp6",
            Proto::Tcp => "tcp",
            Proto::Tcp4 => "tcp4",
            Proto::Tcp6 => "tcp6",
        }
    }

    pub const fn is_tcp(self) -> bool {
        matches!(self, Proto::Tcp | Proto::Tcp4 | Proto::Tcp6)
    }

    /// Whether it reaches the server over IPv6 alone.
    pub const fn is_v6_only(self) -> bool {
        matches!(self, Proto::Udp6 | Proto::Tcp6)
    }
}

/// A server: its name or address, port and protocol.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Remote {
    pub endpoint: Endpoint,
    pub proto: Proto,
}

impl fmt::Display for Remote {
    /// As NetworkManager's plugin takes it: `vpn.example.com:1194:udp`, `[2001:db8::1]:443:tcp`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.endpoint, self.proto.name())
    }
}

/// The secrets of a config, each by the name the vault and the keyring keep it under.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct Credentials {
    pub cert: Option<Secret>,
    pub key: Option<Secret>,
    pub tls_auth: Option<Secret>,
    pub tls_crypt: Option<Secret>,
    pub tls_crypt_v2: Option<Secret>,
    pub username: Option<Secret>,
    pub password: Option<Secret>,
    /// The private key's passphrase.
    pub cert_pass: Option<Secret>,
}

impl fmt::Debug for Credentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let held: Vec<&str> = self.iter().map(|(name, _)| name).collect();
        f.debug_struct("Credentials").field("held", &held).finish()
    }
}

impl Credentials {
    /// Every secret's name, in the vault, the keyring and the order they are stored.
    pub const NAMES: [&'static str; 8] = [
        "cert",
        "key",
        "tls-auth",
        "tls-crypt",
        "tls-crypt-v2",
        "username",
        "password",
        "cert-pass",
    ];

    fn slot(&mut self, name: &str) -> Option<&mut Option<Secret>> {
        Some(match name {
            "cert" => &mut self.cert,
            "key" => &mut self.key,
            "tls-auth" => &mut self.tls_auth,
            "tls-crypt" => &mut self.tls_crypt,
            "tls-crypt-v2" => &mut self.tls_crypt_v2,
            "username" => &mut self.username,
            "password" => &mut self.password,
            "cert-pass" => &mut self.cert_pass,
            _ => return None,
        })
    }

    /// The secret named `name`.
    pub fn get(&self, name: &str) -> Option<&Secret> {
        match name {
            "cert" => self.cert.as_ref(),
            "key" => self.key.as_ref(),
            "tls-auth" => self.tls_auth.as_ref(),
            "tls-crypt" => self.tls_crypt.as_ref(),
            "tls-crypt-v2" => self.tls_crypt_v2.as_ref(),
            "username" => self.username.as_ref(),
            "password" => self.password.as_ref(),
            "cert-pass" => self.cert_pass.as_ref(),
            _ => None,
        }
    }

    /// Sets the secret named `name`; false for a name that isn't one.
    pub fn set(&mut self, name: &str, secret: Secret) -> bool {
        match self.slot(name) {
            Some(slot) => {
                *slot = Some(secret);
                true
            }
            None => false,
        }
    }

    /// The secrets held, by name, in [`Credentials::NAMES`]' order.
    pub fn iter(&self) -> impl Iterator<Item = (&'static str, &Secret)> {
        Credentials::NAMES
            .iter()
            .filter_map(|name| self.get(name).map(|secret| (*name, secret)))
    }

    pub fn is_empty(&self) -> bool {
        self.iter().next().is_none()
    }

    /// What `config` can't come up without that these lack, by name: a certificate and its key,
    /// or the username and password `auth-user-pass` asks for.
    pub fn missing(&self, config: &Config) -> Vec<&'static str> {
        let mut missing = Vec::new();
        if config.auth_user_pass {
            missing.extend(["username", "password"].into_iter().filter(|n| self.get(n).is_none()));
        }
        if !config.auth_user_pass || self.cert.is_some() || self.key.is_some() {
            missing.extend(["cert", "key"].into_iter().filter(|n| self.get(n).is_none()));
        }
        if self.key.as_ref().is_some_and(|key| is_encrypted(key.expose())) && self.cert_pass.is_none() {
            missing.push("cert-pass");
        }
        missing
    }
}

/// Whether a private key in PEM needs its passphrase.
pub fn is_encrypted(pem: &str) -> bool {
    pem.contains("ENCRYPTED")
}

/// A whole `.ovpn`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Config {
    pub remotes: Vec<Remote>,
    pub remote_random: bool,
    /// The server's CA, in PEM: public, so it stays in the user layer.
    pub ca: Option<String>,
    pub credentials: Credentials,
    /// `tls-auth`'s direction.
    pub key_direction: Option<u8>,
    /// The server asks for a username and password.
    pub auth_user_pass: bool,
    /// The directives handed to the plugin, as the config writes them.
    pub options: Vec<Vec<String>>,
    /// `redirect-gateway` or a route to every address: everything goes through it.
    pub full: bool,
    /// `redirect-gateway`'s flags, as the config writes them.
    pub redirect_gateway: Option<Vec<String>>,
    /// The config's own routes; the server pushes its routes as the tunnel comes up.
    pub routes: Vec<Cidr>,
    /// `dhcp-option DNS`'s addresses.
    pub dns_servers: Vec<IpAddr>,
    /// `dhcp-option DOMAIN`'s names, the tunnel's routing domains here.
    pub dns_domains: Vec<String>,
    /// The directives KubaOS doesn't act on that could matter (`up`, `http-proxy`...): the import
    /// says it left them out.
    pub ignored: Vec<String>,
}

/// Why a `.ovpn` can't be used: the line, 0 for the whole file, and what is wrong with it, the
/// source. A secret's value is never in it.
#[derive(Debug)]
pub struct ParseError {
    pub line: usize,
    pub problem: Problem,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.line {
            0 => f.write_str("the OpenVPN config"),
            line => write!(f, "line {line} of the OpenVPN config"),
        }
    }
}

impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.problem)
    }
}

/// What is wrong with a line of a `.ovpn`, or with the whole of it (line 0).
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Problem {
    #[error("a quote isn't closed")]
    Quote,
    #[error("<{0}> isn't closed")]
    Unclosed(String),
    #[error("<{0}> isn't a block of an OpenVPN config")]
    Block(String),
    #[error("{0} is in the config twice")]
    Twice(String),
    #[error("{directive} has `{value}`, which isn't {wanted}")]
    BadValue {
        directive: String,
        value: String,
        wanted: &'static str,
    },
    #[error("{0} needs a value")]
    NoValue(String),
    #[error("{0} names [inline] but the config has no <{0}> block")]
    NoInline(String),
    #[error("<{0}> isn't PEM (-----BEGIN …-----)")]
    NotPem(String),
    #[error("reading {}", .0.display())]
    Read(PathBuf, #[source] io::Error),
    #[error("{} is larger than any certificate or key", .0.display())]
    TooLarge(PathBuf),
    #[error("it is a TAP config: KubaOS's VPN runs TUN alone")]
    Tap,
    #[error("it is a PKCS#12 bundle: ask IT for the certificate and key in PEM")]
    Pkcs12,
    #[error("it is a static-key config: KubaOS's VPN runs TLS clients alone")]
    StaticKey,
    #[error("<connection> blocks aren't supported: give each server a remote line")]
    Connection,
    #[error("it isn't a client config (no client, or tls-client and pull)")]
    NotClient,
    #[error("the config has no remote")]
    NoRemote,
    #[error("the config has more than {MAX_REMOTES} remotes")]
    TooManyRemotes,
    #[error("the config has no ca: the server's certificate can't be checked")]
    NoCa,
    #[error("cert and key go together, and the config has one of them")]
    HalfPair,
}

/// The directives the plugin takes as they are: the config's name, the plugin's key, and whether
/// the value is a number.
const PASSED: [(&str, &str, bool); 17] = [
    ("cipher", "cipher", false),
    ("data-ciphers", "data-ciphers", false),
    ("data-ciphers-fallback", "data-ciphers-fallback", false),
    ("auth", "auth", false),
    ("tls-cipher", "tls-cipher", false),
    ("tls-version-max", "tls-version-max", false),
    ("remote-cert-tls", "remote-cert-tls", false),
    ("ns-cert-type", "ns-cert-type", false),
    ("allow-compression", "allow-compression", false),
    ("tun-mtu", "tunnel-mtu", true),
    ("fragment", "fragment-size", true),
    ("reneg-sec", "reneg-seconds", true),
    ("ping", "ping", true),
    ("ping-restart", "ping-restart", true),
    ("ping-exit", "ping-exit", true),
    ("connect-timeout", "connect-timeout", true),
    ("server-poll-timeout", "connect-timeout", true),
];

/// The directives the plugin takes in a form of its own: [`Config::data`] turns each.
const SHAPED: [&str; 7] = [
    "tls-version-min",
    "comp-lzo",
    "compress",
    "float",
    "mssfix",
    "verify-x509-name",
    "keepalive",
];

/// What a client config has that the plugin does itself or that changes nothing on Linux, so the
/// import doesn't name it as left out.
const QUIET: [&str; 26] = [
    "client",
    "tls-client",
    "pull",
    "nobind",
    "persist-key",
    "persist-tun",
    "resolv-retry",
    "verb",
    "mute",
    "mute-replay-warnings",
    "auth-nocache",
    "explicit-exit-notify",
    "auth-retry",
    "connect-retry",
    "connect-retry-max",
    "sndbuf",
    "rcvbuf",
    "ncp-disable",
    "block-outside-dns",
    "register-dns",
    "setenv",
    "ignore-unknown-option",
    "user",
    "group",
    "route-delay",
    "route-method",
];

/// The inline blocks a config may hold.
const BLOCKS: [&str; 7] = [
    "ca",
    "cert",
    "key",
    "tls-auth",
    "tls-crypt",
    "tls-crypt-v2",
    "auth-user-pass",
];

/// The inline block named `name`, as [`BLOCKS`] spells it.
fn block(name: &str) -> Option<&'static str> {
    BLOCKS.iter().copied().find(|known| *known == name)
}

/// One line's words, as OpenVPN splits them: on blanks, `"…"` and `'…'` keeping theirs, a
/// backslash taking the next character as it is. A word starting with `#` or `;` starts a
/// comment.
fn words(line: &str) -> Result<Vec<String>, Problem> {
    let mut words = Vec::new();
    let mut chars = line.chars().peekable();
    loop {
        while chars.peek().is_some_and(|c| c.is_whitespace()) {
            chars.next();
        }
        match chars.peek() {
            None | Some('#' | ';') => return Ok(words),
            Some(_) => {}
        }
        let mut word = String::new();
        while let Some(&c) = chars.peek() {
            if c.is_whitespace() {
                break;
            }
            chars.next();
            match c {
                '"' => loop {
                    match chars.next() {
                        Some('"') => break,
                        Some('\\') => word.push(chars.next().ok_or(Problem::Quote)?),
                        Some(c) => word.push(c),
                        None => return Err(Problem::Quote),
                    }
                },
                '\'' => loop {
                    match chars.next() {
                        Some('\'') => break,
                        Some(c) => word.push(c),
                        None => return Err(Problem::Quote),
                    }
                },
                '\\' => {
                    if let Some(next) = chars.next() {
                        word.push(next);
                    }
                }
                c => word.push(c),
            }
        }
        words.push(word);
    }
}

/// A word as OpenVPN reads it back: quoted when it holds a blank, a quote or a backslash.
fn quoted(word: &str) -> String {
    if !word.is_empty() && !word.chars().any(|c| c.is_whitespace() || "\"'\\#;".contains(c)) {
        return word.to_owned();
    }
    let mut out = String::from("\"");
    for c in word.chars() {
        if c == '"' || c == '\\' {
            out.push('\\');
        }
        out.push(c);
    }
    out.push('"');
    out
}

fn bad(directive: &str, value: &str, wanted: &'static str) -> Problem {
    Problem::BadValue {
        directive: directive.to_owned(),
        value: value.to_owned(),
        wanted,
    }
}

/// A directive's argument: no control characters, at most [`MAX_ARG`] long.
fn plain<'a>(directive: &str, value: &'a str) -> Result<&'a str, Problem> {
    if value.is_empty() || value.len() > MAX_ARG || value.chars().any(char::is_control) {
        return Err(bad(directive, value, "a plain value"));
    }
    Ok(value)
}

fn number(directive: &str, value: &str) -> Result<u32, Problem> {
    value.parse().map_err(|_| bad(directive, value, "a number"))
}

/// A secret as PEM: what the file or the block holds from its `-----BEGIN` line, a newline after
/// it. The comment lines `openvpn --genkey` writes above a static key are left out.
fn pem(name: &str, text: &str) -> Result<Secret, Problem> {
    let not_pem = || Problem::NotPem(name.to_owned());
    let start = text.find("-----BEGIN ").ok_or_else(not_pem)?;
    let (before, text) = text.split_at(start);
    let commented = before
        .lines()
        .map(str::trim)
        .all(|line| line.is_empty() || line.starts_with('#'));
    let text = text.trim();
    if !commented || !text.contains("-----END ") {
        return Err(not_pem());
    }
    Ok(Secret::new(format!("{text}\n")))
}

/// A netmask as its prefix: `255.255.255.0` is 24.
fn prefix_of(mask: Ipv4Addr) -> Option<u8> {
    let bits = u32::from(mask);
    let prefix = bits.leading_ones();
    (bits.checked_shl(prefix).unwrap_or(0) == 0).then_some(prefix as u8)
}

/// A file a directive names: in `dir` unless absolute, at most [`MAX_FILE_BYTES`].
fn read_file(dir: &Path, name: &str) -> Result<String, Problem> {
    let path = if Path::new(name).is_absolute() {
        PathBuf::from(name)
    } else {
        dir.join(name)
    };
    let file = fs::File::open(&path).map_err(|err| Problem::Read(path.clone(), err))?;
    let mut text = String::new();
    file.take(MAX_FILE_BYTES + 1)
        .read_to_string(&mut text)
        .map_err(|err| Problem::Read(path.clone(), err))?;
    if text.len() as u64 > MAX_FILE_BYTES {
        return Err(Problem::TooLarge(path));
    }
    Ok(text)
}

/// A file or an inline block, by the directive that names it: `ca`, `cert`, `key`...
#[derive(Default)]
struct Source {
    line: usize,
    file: Option<String>,
    inline: Option<String>,
}

/// What a `remote` line says, before the config's `port` and `proto` fill in what it leaves out.
struct RemoteLine {
    line: usize,
    host: String,
    port: Option<u16>,
    proto: Option<Proto>,
}

/// What the lines say, before it is checked whole.
#[derive(Default)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "Each is a directive the config has or lacks, read once into the Config."
)]
struct Draft {
    client: bool,
    tls_client: bool,
    pull: bool,
    remotes: Vec<RemoteLine>,
    port: Option<u16>,
    proto: Option<Proto>,
    remote_random: bool,
    sources: Vec<(&'static str, Source)>,
    key_direction: Option<u8>,
    auth_user_pass: bool,
    /// The file `auth-user-pass` names, with the username and password.
    auth_file: Option<String>,
    /// The file `askpass` names, with the key's passphrase.
    askpass_file: Option<String>,
    options: Vec<Vec<String>>,
    redirect_gateway: Option<Vec<String>>,
    routes: Vec<Cidr>,
    full: bool,
    dns_servers: Vec<IpAddr>,
    dns_domains: Vec<String>,
    ignored: Vec<String>,
}

impl Draft {
    fn source(&mut self, name: &'static str) -> &mut Source {
        let index = if let Some(index) = self.sources.iter().position(|(n, _)| *n == name) {
            index
        } else {
            self.sources.push((name, Source::default()));
            self.sources.len() - 1
        };
        &mut self.sources[index].1
    }

    fn ignore(&mut self, directive: &str) {
        if !QUIET.contains(&directive) && !self.ignored.iter().any(|d| d == directive) {
            self.ignored.push(directive.to_owned());
        }
    }

    /// One directive and its arguments.
    fn directive(&mut self, line: usize, words: &[String]) -> Result<(), Problem> {
        let name = words[0].trim_start_matches("--").to_ascii_lowercase();
        let args = &words[1..];
        let first = || {
            args.first()
                .map(String::as_str)
                .ok_or_else(|| Problem::NoValue(name.clone()))
        };
        match name.as_str() {
            "client" => self.client = true,
            "tls-client" => self.tls_client = true,
            "pull" => self.pull = true,
            "dev" | "dev-type" => {
                let dev = first()?;
                if dev.starts_with("tap") {
                    return Err(Problem::Tap);
                }
                if !dev.starts_with("tun") {
                    return Err(bad(&name, dev, "tun"));
                }
            }
            "proto" => {
                let proto = first()?;
                self.proto = Some(Proto::parse(proto).ok_or_else(|| bad("proto", proto, "udp or tcp"))?);
            }
            "port" | "rport" => {
                let port = first()?;
                self.port = Some(
                    port.parse()
                        .ok()
                        .filter(|&p| p != 0)
                        .ok_or_else(|| bad(&name, port, "a port"))?,
                );
            }
            "remote" => self.remote(line, args)?,
            "remote-random" => self.remote_random = true,
            "ca" | "cert" | "key" | "tls-auth" | "tls-crypt" | "tls-crypt-v2" => {
                let file = first()?.to_owned();
                if name == "tls-auth"
                    && let Some(direction) = args.get(1)
                {
                    self.key_direction = Some(direction_of(direction)?);
                }
                let Some(name) = block(&name) else {
                    return Err(Problem::Block(name));
                };
                let source = self.source(name);
                if source.file.is_some() {
                    return Err(Problem::Twice(name.to_owned()));
                }
                source.line = line;
                source.file = Some(file);
            }
            "key-direction" => self.key_direction = Some(direction_of(first()?)?),
            "auth-user-pass" => {
                self.auth_user_pass = true;
                self.auth_file = args.first().filter(|file| *file != "[inline]").cloned();
            }
            "askpass" => self.askpass_file = args.first().cloned(),
            "pkcs12" => return Err(Problem::Pkcs12),
            "secret" => return Err(Problem::StaticKey),
            "redirect-gateway" => {
                self.full = true;
                self.redirect_gateway = Some(args.to_vec());
            }
            "route" => self.route_v4(args)?,
            "route-ipv6" => {
                let cidr = first()?;
                let cidr = Cidr::parse(cidr).ok_or_else(|| bad("route-ipv6", cidr, "a range like 2001:db8::/32"))?;
                self.route(cidr);
            }
            "dhcp-option" => self.dhcp_option(args)?,
            _ if PASSED.iter().any(|(directive, _, _)| *directive == name) || SHAPED.contains(&name.as_str()) => {
                self.option(&name, args)?;
            }
            _ => self.ignore(&name),
        }
        Ok(())
    }

    /// `remote HOST [PORT] [PROTO]`: a server, the config's `port` and `proto` for what it leaves
    /// out.
    fn remote(&mut self, line: usize, args: &[String]) -> Result<(), Problem> {
        let host = args
            .first()
            .ok_or_else(|| Problem::NoValue("remote".to_owned()))?
            .to_ascii_lowercase();
        if host.parse::<IpAddr>().is_err() && !is_hostname(host.trim_end_matches('.')) {
            return Err(bad("remote", &host, "a server's name or address"));
        }
        let port = match args.get(1) {
            Some(port) => Some(
                port.parse()
                    .ok()
                    .filter(|&p| p != 0)
                    .ok_or_else(|| bad("remote", port, "a port"))?,
            ),
            None => None,
        };
        let proto = match args.get(2) {
            Some(proto) => Some(Proto::parse(proto).ok_or_else(|| bad("remote", proto, "udp or tcp"))?),
            None => None,
        };
        self.remotes.push(RemoteLine {
            line,
            host: host.trim_end_matches('.').to_owned(),
            port,
            proto,
        });
        Ok(())
    }

    fn route(&mut self, cidr: Cidr) {
        if cidr.is_default() {
            self.full = true;
        } else if !self.routes.contains(&cidr.network()) {
            self.routes.push(cidr.network());
        }
    }

    /// `route NETWORK [NETMASK] [GATEWAY] [METRIC]`: a range the tunnel carries. A network given as
    /// a name, or the server's own keywords, carry nothing the kill switch can name.
    fn route_v4(&mut self, args: &[String]) -> Result<(), Problem> {
        let network = args.first().ok_or_else(|| Problem::NoValue("route".to_owned()))?;
        let Ok(network) = network.parse::<Ipv4Addr>() else {
            self.ignore("route");
            return Ok(());
        };
        let prefix = match args.get(1) {
            Some(mask) if mask != "default" => {
                let parsed: Ipv4Addr = mask.parse().map_err(|_| bad("route", mask, "a netmask"))?;
                prefix_of(parsed).ok_or_else(|| bad("route", mask, "a netmask"))?
            }
            _ => 32,
        };
        self.route(Cidr {
            addr: IpAddr::V4(network),
            prefix,
        });
        Ok(())
    }

    fn dhcp_option(&mut self, args: &[String]) -> Result<(), Problem> {
        let (Some(kind), Some(value)) = (args.first(), args.get(1)) else {
            return Err(Problem::NoValue("dhcp-option".to_owned()));
        };
        match kind.to_ascii_uppercase().as_str() {
            "DNS" | "DNS6" => {
                let addr: IpAddr = value.parse().map_err(|_| bad("dhcp-option DNS", value, "an address"))?;
                if !self.dns_servers.contains(&addr) {
                    self.dns_servers.push(addr);
                }
            }
            "DOMAIN" | "DOMAIN-SEARCH" | "ADAPTER_DOMAIN_SUFFIX" => {
                let domain = value.trim_end_matches('.').to_ascii_lowercase();
                if !is_hostname(&domain) {
                    return Err(bad("dhcp-option DOMAIN", value, "a domain"));
                }
                if !self.dns_domains.contains(&domain) {
                    self.dns_domains.push(domain);
                }
            }
            _ => self.ignore("dhcp-option"),
        }
        Ok(())
    }

    /// A directive the plugin takes, checked as [`Config::data`] needs it.
    fn option(&mut self, name: &str, args: &[String]) -> Result<(), Problem> {
        for arg in args {
            plain(name, arg)?;
        }
        let arg = |index: usize| args.get(index).map(String::as_str);
        let needs = |index: usize| arg(index).ok_or_else(|| Problem::NoValue(name.to_owned()));
        if let Some((_, _, numeric)) = PASSED.iter().find(|(directive, _, _)| *directive == name) {
            let value = needs(0)?;
            if *numeric {
                number(name, value)?;
            }
        }
        match name {
            "tls-version-min" => {
                needs(0)?;
                if let Some(extra) = arg(1).filter(|extra| *extra != "or-highest") {
                    return Err(bad(name, extra, "or-highest"));
                }
            }
            "comp-lzo" => {
                if let Some(mode) = arg(0).filter(|mode| !matches!(*mode, "yes" | "no" | "adaptive")) {
                    return Err(bad(name, mode, "yes, no or adaptive"));
                }
            }
            "compress" => {
                if let Some(algo) =
                    arg(0).filter(|algo| !matches!(*algo, "lzo" | "lz4" | "lz4-v2" | "stub" | "stub-v2"))
                {
                    return Err(bad(name, algo, "lzo, lz4, lz4-v2, stub or stub-v2"));
                }
            }
            "mssfix" => {
                if let Some(size) = arg(0) {
                    number(name, size)?;
                }
            }
            "verify-x509-name" => {
                needs(0)?;
                if let Some(kind) = arg(1).filter(|kind| !matches!(*kind, "subject" | "name" | "name-prefix")) {
                    return Err(bad(name, kind, "subject, name or name-prefix"));
                }
            }
            "keepalive" => {
                number(name, needs(0)?)?;
                number(name, needs(1)?)?;
            }
            _ => {}
        }
        if self.options.iter().any(|option| option[0] == name) {
            return Err(Problem::Twice(name.to_owned()));
        }
        self.options
            .push(std::iter::once(name.to_owned()).chain(args.iter().cloned()).collect());
        Ok(())
    }
}

fn direction_of(text: &str) -> Result<u8, Problem> {
    match text {
        "0" => Ok(0),
        "1" => Ok(1),
        _ => Err(bad("key-direction", text, "0 or 1")),
    }
}

impl Config {
    /// Parses a `.ovpn`, reading the files it names in `dir` (the config's own folder). `#` and
    /// `;` start a comment, and a directive may be written with OpenVPN's `--`.
    ///
    /// # Errors
    ///
    /// The first line that isn't an OpenVPN client config, a file it names that can't be read, or
    /// what the whole lacks: a client with a server and the server's CA.
    pub fn parse(text: &str, dir: &Path) -> Result<Config, ParseError> {
        let mut draft = Draft::default();
        let mut open: Option<(usize, &'static str, String)> = None;
        for (index, raw) in text.lines().enumerate() {
            let line = index + 1;
            let err = |problem| ParseError { line, problem };
            let trimmed = raw.trim();
            if let Some((start, name, content)) = &mut open {
                if trimmed.eq_ignore_ascii_case(&format!("</{name}>")) {
                    let name = *name;
                    let content = std::mem::take(content);
                    let start = *start;
                    let source = draft.source(name);
                    if source.inline.is_some() {
                        return Err(ParseError {
                            line: start,
                            problem: Problem::Twice(format!("<{name}>")),
                        });
                    }
                    source.inline = Some(content);
                    if source.line == 0 {
                        source.line = start;
                    }
                    open = None;
                } else {
                    content.push_str(raw);
                    content.push('\n');
                }
                continue;
            }
            if let Some(name) = trimmed.strip_prefix('<').and_then(|t| t.strip_suffix('>')) {
                let name = name.trim().to_ascii_lowercase();
                if name == "connection" {
                    return Err(err(Problem::Connection));
                }
                if name == "pkcs12" {
                    return Err(err(Problem::Pkcs12));
                }
                if name == "secret" {
                    return Err(err(Problem::StaticKey));
                }
                let Some(name) = block(&name) else {
                    return Err(err(Problem::Block(name)));
                };
                open = Some((line, name, String::new()));
                continue;
            }
            let words = words(trimmed).map_err(err)?;
            if words.is_empty() {
                continue;
            }
            draft.directive(line, &words).map_err(err)?;
        }
        if let Some((line, name, _)) = open {
            return Err(ParseError {
                line,
                problem: Problem::Unclosed(name.to_owned()),
            });
        }
        draft.finish(dir)
    }
}

/// A username on the first line and a password on the second, as `auth-user-pass FILE` holds them.
fn credentials_from_lines(credentials: &mut Credentials, text: &str) {
    let mut lines = text.lines();
    if let Some(username) = lines.next().map(str::trim).filter(|u| !u.is_empty()) {
        credentials.username = Some(Secret::new(username.to_owned()));
    }
    if let Some(password) = lines.next().filter(|p| !p.is_empty()) {
        credentials.password = Some(Secret::new(password.to_owned()));
    }
}

impl Draft {
    /// The CA and the credentials, from their files or their blocks.
    fn sources(&mut self, dir: &Path) -> Result<(Option<String>, Credentials), ParseError> {
        let mut ca = None;
        let mut credentials = Credentials::default();
        for (name, source) in std::mem::take(&mut self.sources) {
            let at = |problem| ParseError {
                line: source.line,
                problem,
            };
            let text = match (&source.file, source.inline) {
                (Some(file), _) if file != "[inline]" => read_file(dir, file).map_err(at)?,
                (_, Some(inline)) => inline,
                (_, None) => return Err(at(Problem::NoInline(name.to_owned()))),
            };
            if name == "auth-user-pass" {
                self.auth_user_pass = true;
                credentials_from_lines(&mut credentials, &text);
                continue;
            }
            let secret = pem(name, &text).map_err(at)?;
            if name == "ca" {
                ca = Some(secret.expose().to_owned());
            } else {
                credentials.set(name, secret);
            }
        }
        Ok((ca, credentials))
    }

    fn finish(mut self, dir: &Path) -> Result<Config, ParseError> {
        let whole = |problem| ParseError { line: 0, problem };
        if !(self.client || self.tls_client && self.pull) {
            return Err(whole(Problem::NotClient));
        }
        let (ca, mut credentials) = self.sources(dir)?;
        if let Some(file) = &self.auth_file {
            let text = read_file(dir, file).map_err(whole)?;
            credentials_from_lines(&mut credentials, &text);
        }
        if let Some(file) = &self.askpass_file {
            let text = read_file(dir, file).map_err(whole)?;
            if let Some(pass) = text.lines().next().filter(|p| !p.is_empty()) {
                credentials.cert_pass = Some(Secret::new(pass.to_owned()));
            }
        }
        if self.remotes.is_empty() {
            return Err(whole(Problem::NoRemote));
        }
        if self.remotes.len() > MAX_REMOTES {
            return Err(ParseError {
                line: self.remotes[MAX_REMOTES].line,
                problem: Problem::TooManyRemotes,
            });
        }
        if ca.is_none() {
            return Err(whole(Problem::NoCa));
        }
        if credentials.cert.is_some() != credentials.key.is_some() {
            return Err(whole(Problem::HalfPair));
        }
        let proto = self.proto.unwrap_or(Proto::Udp);
        let port = self.port.unwrap_or(DEFAULT_PORT);
        let remotes = self
            .remotes
            .iter()
            .map(|remote| Remote {
                endpoint: Endpoint {
                    host: remote.host.clone(),
                    port: remote.port.unwrap_or(port),
                },
                proto: remote.proto.unwrap_or(proto),
            })
            .collect();
        Ok(Config {
            remotes,
            remote_random: self.remote_random,
            ca,
            credentials,
            key_direction: self.key_direction,
            auth_user_pass: self.auth_user_pass,
            options: self.options,
            full: self.full,
            redirect_gateway: self.redirect_gateway,
            routes: self.routes,
            dns_servers: self.dns_servers,
            dns_domains: self.dns_domains,
            ignored: self.ignored,
        })
    }
}

impl Config {
    /// Everything goes through it: `redirect-gateway`, or a route to every address.
    pub fn is_full(&self) -> bool {
        self.full
    }

    /// The first server, which the shell shows.
    pub fn endpoint(&self) -> Option<&Endpoint> {
        self.remotes.first().map(|remote| &remote.endpoint)
    }

    /// The config without its secrets, as the user layer keeps it.
    #[must_use]
    pub fn sanitized(&self) -> Config {
        Config {
            credentials: Credentials::default(),
            ignored: Vec::new(),
            ..self.clone()
        }
    }

    /// The plugin's settings (`vpn.data`) for what the config says, the servers as they are here:
    /// NetworkManager's names, each value a string.
    pub fn data(&self) -> Vec<(&'static str, String)> {
        let remotes: Vec<String> = self.remotes.iter().map(ToString::to_string).collect();
        let mut data = vec![("remote", remotes.join(", "))];
        if self.remote_random {
            data.push(("remote-random", "yes".to_owned()));
        }
        for option in &self.options {
            let name = option[0].as_str();
            let arg = |index: usize| option.get(index + 1).cloned();
            if let Some((_, key, _)) = PASSED.iter().find(|(directive, _, _)| *directive == name) {
                data.push((key, arg(0).unwrap_or_default()));
                continue;
            }
            match name {
                "tls-version-min" => {
                    data.push(("tls-version-min", arg(0).unwrap_or_default()));
                    if arg(1).is_some() {
                        data.push(("tls-version-min-or-highest", "yes".to_owned()));
                    }
                }
                "comp-lzo" => data.push(("comp-lzo", arg(0).unwrap_or_else(|| "adaptive".to_owned()))),
                "compress" => data.push(("compress", arg(0).unwrap_or_else(|| "yes".to_owned()))),
                "float" => data.push(("float", "yes".to_owned())),
                "mssfix" => data.push(("mssfix", arg(0).unwrap_or_else(|| "yes".to_owned()))),
                "verify-x509-name" => {
                    let kind = arg(1).unwrap_or_else(|| "subject".to_owned());
                    data.push(("verify-x509-name", format!("{kind}:{}", arg(0).unwrap_or_default())));
                }
                "keepalive" => {
                    data.push(("ping", arg(0).unwrap_or_default()));
                    data.push(("ping-restart", arg(1).unwrap_or_default()));
                }
                _ => {}
            }
        }
        data
    }

    /// The config in OpenVPN's format, its secrets inline where it has them.
    pub fn render(&self) -> String {
        let mut lines = vec!["client".to_owned(), "dev tun".to_owned()];
        for remote in &self.remotes {
            let host = &remote.endpoint.host;
            lines.push(format!(
                "remote {host} {} {}",
                remote.endpoint.port,
                remote.proto.name()
            ));
        }
        if self.remote_random {
            lines.push("remote-random".to_owned());
        }
        if let Some(flags) = &self.redirect_gateway {
            lines.push(join("redirect-gateway", flags));
        } else if self.full {
            lines.push("redirect-gateway def1".to_owned());
        }
        for route in &self.routes {
            lines.push(match route.addr {
                IpAddr::V4(addr) => format!("route {addr} {}", netmask(route.prefix)),
                IpAddr::V6(_) => format!("route-ipv6 {route}"),
            });
        }
        for server in &self.dns_servers {
            lines.push(format!("dhcp-option DNS {server}"));
        }
        for domain in &self.dns_domains {
            lines.push(format!("dhcp-option DOMAIN {domain}"));
        }
        for option in &self.options {
            lines.push(join(&option[0], &option[1..]));
        }
        if let Some(direction) = self.key_direction {
            lines.push(format!("key-direction {direction}"));
        }
        if self.auth_user_pass {
            lines.push("auth-user-pass".to_owned());
        }
        let mut block = |name: &str, pem: &str| {
            lines.push(format!("<{name}>"));
            lines.push(pem.trim_end().to_owned());
            lines.push(format!("</{name}>"));
        };
        if let Some(ca) = &self.ca {
            block("ca", ca);
        }
        for (name, secret) in self.credentials.iter() {
            if !matches!(name, "username" | "password" | "cert-pass") {
                block(name, secret.expose());
            }
        }
        lines.push(String::new());
        lines.join("\n")
    }
}

fn join(name: &str, args: &[String]) -> String {
    std::iter::once(name.to_owned())
        .chain(args.iter().map(|arg| quoted(arg)))
        .collect::<Vec<_>>()
        .join(" ")
}

/// A prefix as a netmask: 24 is `255.255.255.0`.
fn netmask(prefix: u8) -> Ipv4Addr {
    Ipv4Addr::from(u32::MAX.checked_shl(32 - u32::from(prefix.min(32))).unwrap_or(0))
}

#[cfg(test)]
pub(crate) mod tests;
