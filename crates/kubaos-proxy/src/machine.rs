//! What the Proxy changes outside its own files, behind one trait, so putting a profile in place
//! and taking it away is tested without NetworkManager, pkexec, systemd, GNOME or a resolver.
//!
//! # Owns
//!
//! - [`Machine`]: running a program, the tunnel in NetworkManager, resolving a name; and their
//!   errors.
//!
//! # Not here
//!
//! - The machine for real: [`crate::real`]. The tests' own: `fake.rs`.

use std::net::IpAddr;

pub use kubaos_core::bitwarden::Cmd;
pub use kubaos_core::bitwarden::Ran;

use crate::tunnel::Connection;

/// What a program or service said when it failed, in its own words.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{0}")]
pub struct Said(pub String);

/// A program couldn't be started.
#[derive(Debug, thiserror::Error)]
#[error("couldn't run {program}")]
pub struct StartError {
    pub program: String,
    #[source]
    pub said: Said,
}

/// Why NetworkManager didn't do what the tunnel needed.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum TunnelError {
    #[error("NetworkManager couldn't be reached on the system bus")]
    Unreachable(#[source] Said),
    #[error("NetworkManager refused the tunnel")]
    Refused(#[source] Said),
    #[error("NetworkManager dropped the tunnel as it came up (journalctl -u NetworkManager says why)")]
    Dropped,
    #[error("the tunnel wasn't up after {0} s")]
    NotUp(u64),
    #[error("the tunnel was still up after {0} s")]
    StillUp(u64),
}

/// A name didn't resolve.
#[derive(Debug, thiserror::Error)]
#[error("{host} doesn't resolve")]
pub struct ResolveError {
    pub host: String,
    #[source]
    pub said: Said,
}

/// The programs, NetworkManager and the resolver.
pub trait Machine {
    /// Runs a program to its end; its stdin, when the command has one, is never in its argv.
    ///
    /// # Errors
    ///
    /// The program couldn't start.
    fn run(&mut self, cmd: &Cmd) -> Result<Ran, StartError>;

    /// Brings the tunnel up as `connection` says, in place of one that is up, and waits until
    /// NetworkManager has it up.
    ///
    /// # Errors
    ///
    /// NetworkManager refused the connection or couldn't bring it up.
    fn tunnel_up(&mut self, connection: &Connection) -> Result<(), TunnelError>;

    /// Takes the tunnel down; down already is fine.
    ///
    /// # Errors
    ///
    /// NetworkManager couldn't be asked, or refused.
    fn tunnel_down(&mut self) -> Result<(), TunnelError>;

    /// Whether the tunnel is up.
    ///
    /// # Errors
    ///
    /// NetworkManager couldn't be asked.
    fn tunnel_active(&mut self) -> Result<bool, TunnelError>;

    /// The addresses `host` resolves to.
    ///
    /// # Errors
    ///
    /// The name doesn't resolve, or the resolver can't be reached.
    fn resolve(&mut self, host: &str, port: u16) -> Result<Vec<IpAddr>, ResolveError>;
}
