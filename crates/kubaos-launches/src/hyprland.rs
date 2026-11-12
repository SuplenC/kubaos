//! Telling Hyprland of a launch: `dot.launches.started(unit, app)` evaluated in its Lua config,
//! sent straight to its request socket, as `hyprctl eval` sends it, without a process per launch.
//!
//! # Owns
//!
//! - The socket's path and the request ([`Hyprland::started`]), quoting text for Lua
//!   ([`lua_string`]).
//!
//! # Not here
//!
//! - What Hyprland does with it: `config/hypr/launches.lua`.

use std::env;
use std::fmt::Write as _;
use std::io;
use std::io::Read;
use std::io::Write;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::time::Duration;

/// How long Hyprland may take to answer. It answers at once; a Hyprland that hangs must not hold
/// up the next launch for long.
const TIMEOUT: Duration = Duration::from_secs(1);

/// The Hyprland of this session.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hyprland {
    socket: PathBuf,
}

/// Hyprland couldn't be told of a launch.
#[derive(Debug, thiserror::Error)]
pub enum TellError {
    #[error("reaching Hyprland at {0}")]
    Reach(PathBuf, #[source] io::Error),
    #[error("Hyprland refused the launch: {0}")]
    Refused(String),
}

impl Hyprland {
    /// The Hyprland whose instance is `HYPRLAND_INSTANCE_SIGNATURE`, under `XDG_RUNTIME_DIR`; none
    /// without both.
    pub fn from_env() -> Option<Self> {
        let var = |name| env::var_os(name).filter(|value| !value.is_empty());
        let runtime = PathBuf::from(var("XDG_RUNTIME_DIR")?);
        let signature = var("HYPRLAND_INSTANCE_SIGNATURE")?;
        Some(Hyprland {
            socket: runtime.join("hypr").join(signature).join(".socket.sock"),
        })
    }

    /// Tells Hyprland that unit `unit` started app `app` just now.
    ///
    /// # Errors
    ///
    /// When the socket can't be reached, or Hyprland answers with anything but `ok`.
    pub fn started(&self, unit: &str, app: &str) -> Result<(), TellError> {
        let reach = |err| TellError::Reach(self.socket.clone(), err);
        let mut stream = UnixStream::connect(&self.socket).map_err(reach)?;
        stream.set_read_timeout(Some(TIMEOUT)).map_err(reach)?;
        stream.set_write_timeout(Some(TIMEOUT)).map_err(reach)?;
        stream.write_all(request(unit, app).as_bytes()).map_err(reach)?;
        let mut answer = String::new();
        stream.read_to_string(&mut answer).map_err(reach)?;
        match answer.trim() {
            "ok" => Ok(()),
            other => Err(TellError::Refused(other.to_owned())),
        }
    }
}

/// The request that evaluates `dot.launches.started(unit, app)`.
fn request(unit: &str, app: &str) -> String {
    format!("/eval dot.launches.started({}, {})", lua_string(unit), lua_string(app))
}

/// `text` as a Lua string literal: in double quotes, with backslashes, quotes and control
/// characters escaped, so a unit's `\x2d` reaches Lua as written.
pub fn lua_string(text: &str) -> String {
    let mut quoted = String::with_capacity(text.len() + 2);
    quoted.push('"');
    for c in text.chars() {
        match c {
            '\\' => quoted.push_str(r"\\"),
            '"' => quoted.push_str("\\\""),
            c if c.is_ascii_control() => {
                let _ = write!(quoted, "\\{:03}", u32::from(c));
            }
            c => quoted.push(c),
        }
    }
    quoted.push('"');
    quoted
}

#[cfg(test)]
mod tests;
