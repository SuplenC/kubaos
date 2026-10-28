//! `kubaos proxy`: the Proxy, one switch for the work network, through `kubaos-proxy`, which the
//! home installs with its service (`home/proxy.nix`). The words after `proxy` go to it as they
//! are typed:
//!
//!   kubaos proxy on | off | status [--json] [--live] | import FILE | pull
//!
//! # Not here
//!
//! - Everything it does: the `kubaos-proxy` crate.

use std::os::unix::process::CommandExt;
use std::process::Command;
use std::process::ExitCode;

use crate::out::err;

/// The Proxy's own command.
const PROGRAM: &str = "kubaos-proxy";

/// Runs `kubaos-proxy` with `args` in place of this process, so its output and exit code are
/// `kubaos proxy`'s.
pub fn main(args: &[String]) -> ExitCode {
    let err = Command::new(PROGRAM).args(args).exec();
    err!("kubaos: couldn't run {PROGRAM} ({err}); a home-manager switch installs it (home/proxy.nix)");
    ExitCode::FAILURE
}
