//! A machine of the tests' own: NetworkManager, the root helpers and their pins, systemd's user
//! environment, gsettings, the keyring, the vault and the resolver as maps, every command it ran
//! kept, and any command made to fail.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::net::IpAddr;
use std::os::unix::fs::PermissionsExt as _;
use std::path::PathBuf;

use kubaos_core::bitwarden::base64;
use serde_json::Value;

use crate::machine::Cmd;
use crate::machine::Machine;
use crate::machine::Ran;
use crate::machine::ResolveError;
use crate::machine::Said;
use crate::machine::StartError;
use crate::machine::TunnelError;
use crate::tunnel::Connection;
use crate::tunnel::Prop;

/// What `gsettings get` says of a key the fake never set, and what `set` takes as going back to
/// it.
pub const DEFAULT: &str = "@default";

#[derive(Debug, Default)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "Each is a way the tests make the machine fail."
)]
pub struct Fake {
    /// Each command, as `program arg arg`.
    pub ran: Vec<String>,
    /// What each command got on stdin, by its line.
    pub stdins: Vec<(String, String)>,
    /// Entries by their attributes, `service kubaos-proxy key private-key`.
    pub keyring: BTreeMap<String, String>,
    pub env: BTreeMap<String, String>,
    pub gsettings: BTreeMap<String, String>,
    /// The helper's parts in place: `apt`, `killswitch`...
    pub root: BTreeSet<String>,
    /// The pins, by name, the CA's as the PEM it was given.
    pub pins: BTreeMap<String, String>,
    pub tunnel: Option<Connection>,
    pub vault: Vec<Value>,
    pub names: BTreeMap<String, Vec<IpAddr>>,
    /// What `ip -json address` says: the interfaces' addresses.
    pub ip: String,
    /// What `ip -json route` says: the routes on the VPN's interface.
    pub routes: String,
    /// Each file an OpenVPN connection names, by its key, with its mode and what it held as the
    /// VPN came up.
    pub files_at_up: Vec<(String, u32, String)>,
    /// A command whose line contains one of these fails.
    pub fail: Vec<String>,
    pub tunnel_fails: bool,
    pub down_fails: bool,
    /// No name resolves, as with the kill switch up.
    pub resolver_down: bool,
    /// pkexec refuses every run, as polkit does when the prompt is dismissed.
    pub pkexec_refuses: bool,
    /// A file read each time a root helper runs, into `seen`.
    pub watch: Option<PathBuf>,
    pub seen: Vec<String>,
}

fn ok(stdout: impl Into<String>) -> Ran {
    Ran {
        code: Some(0),
        stdout: stdout.into(),
        stderr: String::new(),
    }
}

fn failed(stderr: &str) -> Ran {
    Ran {
        code: Some(1),
        stdout: String::new(),
        stderr: stderr.to_owned(),
    }
}

impl Fake {
    pub fn new() -> Fake {
        Fake::default()
    }

    /// The keyring holding `secret` under `attrs`.
    pub fn keyring_has(&mut self, attrs: &str, secret: &str) {
        self.keyring
            .insert(format!("service kubaos-proxy {attrs}"), secret.to_owned());
    }

    pub fn fail_on(&mut self, part: &str) {
        self.fail.push(part.to_owned());
    }

    /// Whether a command was run whose line starts with `prefix`.
    pub fn ran_like(&self, prefix: &str) -> bool {
        self.ran.iter().any(|line| line.starts_with(prefix))
    }

    /// How many commands ran whose line contains `part`.
    pub fn count(&self, part: &str) -> usize {
        self.ran.iter().filter(|line| line.contains(part)).count()
    }

    fn secret_tool(&mut self, args: &[String], stdin: Option<&str>) -> Ran {
        match args.first().map(String::as_str) {
            Some("store") => {
                let attrs = args[2..].join(" ");
                self.keyring.insert(attrs, stdin.unwrap_or_default().to_owned());
                ok("")
            }
            Some("lookup") => match self.keyring.get(&args[1..].join(" ")) {
                Some(secret) => ok(secret.clone()),
                None => Ran {
                    code: Some(1),
                    ..Ran::default()
                },
            },
            Some("clear") => {
                let prefix = args[1..].join(" ");
                self.keyring.retain(|attrs, _| !attrs.starts_with(&prefix));
                ok("")
            }
            _ => failed("secret-tool: unknown command"),
        }
    }

    fn pinned(&self, arg: &str, stdin: Option<&str>) -> bool {
        let pin = |name: &str| self.pins.get(name).map(String::as_str);
        let Some((name, value)) = arg.split_once('=') else {
            return true;
        };
        if value == "off" || name == "killswitch" {
            return true;
        }
        let proxy = || {
            format!(
                "{}|{}",
                pin("proxy").unwrap_or_default(),
                pin("no-proxy").unwrap_or_default()
            )
        };
        match name {
            "ca" => stdin.is_some() && pin("ca") == stdin,
            "apt" => pin("proxy") == Some(value),
            "docker" | "nix" => value == proxy(),
            "chrome" => {
                value == format!("proxy|{}", proxy())
                    || Some(value) == pin("pac").map(|p| format!("pac|{p}")).as_deref()
            }
            _ => true,
        }
    }

    fn helper(&mut self, args: &[String], stdin: Option<&str>) -> Ran {
        if let Some(watch) = &self.watch {
            self.seen.push(std::fs::read_to_string(watch).unwrap_or_default());
        }
        if !args.iter().all(|arg| self.pinned(arg, stdin)) {
            return Ran {
                code: Some(3),
                stdout: String::new(),
                stderr: "kubaos-proxy-write: isn't what an admin pinned".to_owned(),
            };
        }
        let mut done = String::new();
        for arg in args {
            match arg.split_once('=') {
                _ if arg == "off" => self.root.clear(),
                _ if arg == "stats" => return ok("xTIBA5rboUvnH4htodjb6e697QjLERt1NAB4mZqp8Dg=\t1700000000\n"),
                Some((name, "off")) => {
                    self.root.remove(name);
                }
                Some((name, _)) => {
                    self.root.insert(name.to_owned());
                }
                None => return failed("kubaos-proxy-write: usage"),
            }
            done.push_str(&["done ", arg, "\n"].concat());
        }
        ok(done)
    }

    fn pin(&mut self, args: &[String], stdin: Option<&str>) -> Ran {
        for arg in args {
            let (name, value) = arg.split_once('=').expect("NAME=VALUE");
            let value = if value == "-" { stdin.unwrap_or_default() } else { value };
            self.pins.insert(name.to_owned(), value.to_owned());
        }
        ok("")
    }

    fn bw(&mut self, args: &[String], stdin: Option<&str>) -> Ran {
        let args: Vec<&str> = args.iter().map(String::as_str).collect();
        match args.as_slice() {
            ["list", "items", "--search", name] => {
                let found: Vec<&Value> = self
                    .vault
                    .iter()
                    .filter(|item| item["name"].as_str().is_some_and(|n| n.contains(name)))
                    .collect();
                ok(serde_json::to_string(&found).unwrap())
            }
            ["create", "item"] => {
                let mut item = decode(stdin);
                item["id"] = Value::from(format!("id-{}", self.vault.len()));
                self.vault.push(item);
                ok("{}")
            }
            ["edit", "item", id] => {
                let item = decode(stdin);
                for existing in &mut self.vault {
                    if existing["id"].as_str() == Some(id) {
                        *existing = item.clone();
                    }
                }
                ok("{}")
            }
            _ => failed("bw: unknown command"),
        }
    }

    fn systemctl(&mut self, args: &[String]) -> Ran {
        match args.get(1).map(String::as_str) {
            Some("set-environment") => {
                for var in &args[2..] {
                    let (name, value) = var.split_once('=').expect("NAME=VALUE");
                    self.env.insert(name.to_owned(), value.to_owned());
                }
            }
            Some("unset-environment") => {
                for name in &args[2..] {
                    self.env.remove(name);
                }
            }
            Some("show-environment") => {
                let lines: Vec<String> = self.env.iter().map(|(name, value)| format!("{name}={value}")).collect();
                return ok(lines.join("\n") + "\n");
            }
            _ => {}
        }
        ok("")
    }

    fn gsettings(&mut self, args: &[String]) -> Ran {
        let key = format!("{} {}", args[1], args[2]);
        match args[0].as_str() {
            "get" => {
                return ok(format!(
                    "{}\n",
                    self.gsettings.get(&key).map_or(DEFAULT, String::as_str)
                ));
            }
            "set" if args[3] == DEFAULT => {
                self.gsettings.remove(&key);
            }
            "set" => {
                self.gsettings.insert(key, args[3].clone());
            }
            _ => {
                self.gsettings.remove(&key);
            }
        }
        ok("")
    }
}

fn decode(stdin: Option<&str>) -> Value {
    let bytes = base64::decode(stdin.unwrap_or_default()).expect("bw gets base64");
    serde_json::from_slice(&bytes).expect("bw gets JSON")
}

impl Machine for Fake {
    fn run(&mut self, cmd: &Cmd) -> Result<Ran, StartError> {
        let line = std::iter::once(cmd.program.as_str())
            .chain(cmd.args.iter().map(String::as_str))
            .collect::<Vec<_>>()
            .join(" ");
        self.ran.push(line.clone());
        let stdin = cmd.stdin.as_ref().map(|s| s.0.clone());
        if let Some(stdin) = &stdin {
            self.stdins.push((line.clone(), stdin.clone()));
        }
        if self.fail.iter().any(|part| line.contains(part.as_str())) {
            return Ok(failed("it was made to fail"));
        }
        let program = cmd.program.rsplit('/').next().unwrap_or_default();
        let args = &cmd.args;
        Ok(match program {
            "secret-tool" => self.secret_tool(args, stdin.as_deref()),
            "systemctl" => self.systemctl(args),
            "gsettings" => self.gsettings(args),
            "pkexec" if self.pkexec_refuses => Ran {
                code: Some(126),
                ..Ran::default()
            },
            "pkexec" if args[1].ends_with("kubaos-proxy-pin") => self.pin(&args[2..], stdin.as_deref()),
            "pkexec" => self.helper(&args[2..], stdin.as_deref()),
            "bw" => self.bw(args, stdin.as_deref()),
            "ip" if args.get(1).is_some_and(|arg| arg == "route") => ok(self.routes.clone()),
            "ip" => ok(self.ip.clone()),
            _ => ok(""),
        })
    }

    fn tunnel_up(&mut self, connection: &Connection) -> Result<(), TunnelError> {
        self.ran.push("nm up".to_owned());
        if let Some(Prop::Map(data)) = connection.get("vpn").and_then(|vpn| vpn.get("data")) {
            for key in ["ca", "cert", "key", "ta", "tls-crypt", "tls-crypt-v2"] {
                let Some(path) = data.get(key) else {
                    continue;
                };
                let mode = std::fs::metadata(path).map_or(0, |meta| meta.permissions().mode() & 0o777);
                let text = std::fs::read_to_string(path).unwrap_or_default();
                self.files_at_up.push((key.to_owned(), mode, text));
            }
        }
        if self.tunnel_fails {
            return Err(TunnelError::Dropped);
        }
        self.tunnel = Some(connection.clone());
        Ok(())
    }

    fn tunnel_down(&mut self) -> Result<(), TunnelError> {
        self.ran.push("nm down".to_owned());
        if self.down_fails {
            return Err(TunnelError::Unreachable(Said(
                "NetworkManager isn't running".to_owned(),
            )));
        }
        self.tunnel = None;
        Ok(())
    }

    fn tunnel_active(&mut self) -> Result<bool, TunnelError> {
        Ok(self.tunnel.is_some())
    }

    fn resolve(&mut self, host: &str, _port: u16) -> Result<Vec<IpAddr>, ResolveError> {
        self.ran.push(format!("resolve {host}"));
        let not = |said: &str| ResolveError {
            host: host.to_owned(),
            said: Said(said.to_owned()),
        };
        if self.resolver_down {
            return Err(not("the kill switch blocks the lookup"));
        }
        self.names.get(host).cloned().ok_or_else(|| not("no such name"))
    }
}
