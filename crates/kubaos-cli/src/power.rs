//! `kubaos power`: the power stops, through `kubaos-power`, which the home installs with its
//! service (`home/power.nix`). The words after `power` go to it as they are typed:
//!
//!   kubaos power set STOP | undo | status | measure [--seconds N] | apply
//!   kubaos power panel-power-saving on|off | charge-limit 80|90|100
//!   kubaos power low-battery power-saver|super-saver|off|PERCENT | panel STOP 60hz|vrr|native
//!   kubaos power scheduler [on|off|status]
//!
//! # Not here
//!
//! - Everything it does: the `kubaos-power` crate.

use std::os::unix::process::CommandExt;
use std::process::Command;
use std::process::ExitCode;

use crate::out::err;

/// The power stops' own command.
const PROGRAM: &str = "kubaos-power";

/// Runs `kubaos-power` with `args` in place of this process, so its output and exit code are
/// `kubaos power`'s.
pub fn main(args: &[String]) -> ExitCode {
    let err = Command::new(PROGRAM).args(args).exec();
    err!("kubaos: couldn't run {PROGRAM} ({err}); a home-manager switch installs it (home/power.nix)");
    ExitCode::FAILURE
}
