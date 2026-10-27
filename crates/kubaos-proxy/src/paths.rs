//! Where the Proxy keeps what it knows: the profile in the user layer, what is applied in the
//! state directory (it outlives a reboot, so a login can take it away), and what the session sees
//! in the runtime directory.
//!
//! # Owns
//!
//! - [`Paths`] and every file name the Proxy writes.
//!
//! # Not here
//!
//! - The root side's files: `system/files/proxy/kubaos-proxy-write`.

use std::path::Path;
use std::path::PathBuf;

use kubaos_user_layer::App;

/// The profile's file in the user layer's network folder.
pub const PROFILE: &str = "proxy.toml";
/// The tunnel's `.conf` without its keys, beside the profile, as `kubaos-proxy import` writes it.
pub const DEFAULT_CONF: &str = "wireguard.conf";
/// The root helper, as `install_proxy_helper` (`system/lib.sh`) installs it.
pub const HELPER: &str = "/usr/local/libexec/kubaos-proxy-write";
/// The same helper by its other name, which pins what the first may put in place, under a polkit
/// action that always asks for an admin's password.
pub const PIN_HELPER: &str = "/usr/local/libexec/kubaos-proxy-pin";
/// Ubuntu's pkexec, by path: polkit's own, never one earlier on the PATH.
pub const PKEXEC: &str = "/usr/bin/pkexec";

/// Every place the Proxy reads or writes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Paths {
    /// The user layer's network folder: `proxy.toml`, the `.conf`, the CA.
    pub network: PathBuf,
    /// `~/.local/state/kubaos-proxy`: what is applied, and git's include. Not `XDG_STATE_HOME`:
    /// `config/git/config` names the include by this path.
    pub state: PathBuf,
    /// `$XDG_RUNTIME_DIR`: the shell's file and the session's switch, gone at logout.
    pub runtime: PathBuf,
    /// Docker's client config folder: `$DOCKER_CONFIG`, else `~/.docker`.
    pub docker: PathBuf,
    /// `/sys`, where the tunnel's byte counts are.
    pub sys: PathBuf,
}

impl Paths {
    /// From the environment: `HOME`, `XDG_CONFIG_HOME`, `XDG_RUNTIME_DIR` and `DOCKER_CONFIG`.
    /// None without a home.
    pub fn from_env() -> Option<Paths> {
        let home = std::env::var_os("HOME").filter(|h| !h.is_empty()).map(PathBuf::from)?;
        let config = kubaos_user_layer::config_home()?;
        let state = home.join(".local/state/kubaos-proxy");
        let runtime = std::env::var_os("XDG_RUNTIME_DIR")
            .filter(|dir| !dir.is_empty())
            .map_or_else(|| state.clone(), PathBuf::from);
        let docker = std::env::var_os("DOCKER_CONFIG")
            .filter(|dir| !dir.is_empty())
            .map_or_else(|| home.join(".docker"), PathBuf::from);
        Some(Paths {
            network: kubaos_user_layer::dir(&config, App::Network),
            state,
            runtime,
            docker,
            sys: PathBuf::from("/sys"),
        })
    }

    /// Everything under `root`, laid out as a home is: what the tests use.
    #[must_use]
    pub fn under(root: &Path) -> Paths {
        Paths {
            network: kubaos_user_layer::dir(&root.join(".config"), App::Network),
            state: root.join(".local/state/kubaos-proxy"),
            runtime: root.join("run"),
            docker: root.join(".docker"),
            sys: root.join("sys"),
        }
    }

    pub fn profile(&self) -> PathBuf {
        self.network.join(PROFILE)
    }

    /// The file the shell watches: whether the Proxy is on and what it holds.
    pub fn shell(&self) -> PathBuf {
        self.runtime.join("kubaos-proxy.json")
    }

    /// The session's own files.
    pub fn session(&self) -> PathBuf {
        self.runtime.join("kubaos-proxy")
    }

    /// `on` while the user wants the Proxy on in this session. A login starts without it, so
    /// nothing switches on by itself.
    pub fn wanted(&self) -> PathBuf {
        self.session().join("wanted")
    }

    /// Taken by whatever changes the Proxy, so the service and a command never interleave.
    pub fn lock(&self) -> PathBuf {
        self.session().join("lock")
    }

    /// The proxy variables for new shells, which zsh sources (`home/shell.nix`).
    pub fn env(&self) -> PathBuf {
        self.session().join("env")
    }

    /// The parts applied, one a line, in the order they went up.
    pub fn applied(&self) -> PathBuf {
        self.state.join("applied")
    }

    /// Git's proxy, which `config/git/config` includes.
    pub fn git(&self) -> PathBuf {
        self.state.join("proxy.gitconfig")
    }

    /// The `proxies.default` the Proxy wrote into Docker's client config and the one it found
    /// there, so `off` puts the user's back and never touches one the user changed since.
    pub fn docker_record(&self) -> PathBuf {
        self.state.join("docker-proxies.json")
    }

    /// GNOME's proxy settings as they were before the Proxy set them, for `off` to put back. In
    /// the state directory, as gsettings outlive the session.
    pub fn gnome_before(&self) -> PathBuf {
        self.state.join("gnome-before")
    }

    /// The user manager's proxy variables as they were before the Proxy set them, for `off` to put
    /// back. In the session's folder, as that environment ends with the session.
    pub fn env_before(&self) -> PathBuf {
        self.session().join("env-before")
    }

    /// The address each peer's endpoint name last resolved to: with the kill switch up, the
    /// tunnel comes back on it without a lookup outside the tunnel.
    pub fn endpoints(&self) -> PathBuf {
        self.state.join("endpoints")
    }

    pub fn docker_config(&self) -> PathBuf {
        self.docker.join("config.json")
    }
}
