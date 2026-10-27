//! The root helper's parts: `kubaos-proxy-write` through pkexec, without a password, for what an
//! admin pinned, and `kubaos-proxy-pin`, with the password, when the profile asks for something
//! else.
//!
//! # Owns
//!
//! - Running the helpers, and the pins a plan's root parts need.
//!
//! # Not here
//!
//! - What the helpers check and write: `system/files/proxy/kubaos-proxy-write`.

use super::Ran;
use super::Said;
use super::StepError;
use crate::machine::Cmd;
use crate::machine::Machine;
use crate::paths::HELPER;
use crate::paths::PIN_HELPER;
use crate::paths::PKEXEC;
use crate::plan::ROOT_PARTS;
use crate::plan::RootArg;

/// The write helper's exit when a value isn't pinned; it changed nothing.
const NOT_PINNED: i32 = 3;

/// Runs `program` through pkexec with `args`, `stdin` on its stdin.
fn pkexec(m: &mut dyn Machine, program: &str, args: &[String], stdin: Option<&str>) -> Result<Ran, StepError> {
    let mut cmd = Cmd::new(PKEXEC).arg("--disable-internal-agent").arg(program).args(args);
    if let Some(stdin) = stdin {
        cmd = cmd.secret_stdin(stdin);
    }
    match m.run(&cmd) {
        Ok(ran) if ran.ok() => Ok(ran),
        // pkexec's own exits; the helper's are 0 to 3.
        Ok(ran) if matches!(ran.code, Some(126 | 127)) => Err(StepError::NotRun),
        Ok(ran) if ran.code == Some(NOT_PINNED) => Err(StepError::NotPinned),
        Ok(ran) => {
            let said: Vec<&str> = ran
                .stderr
                .lines()
                .map(|line| line.trim().trim_start_matches("kubaos-proxy-write: "))
                .filter(|line| !line.is_empty())
                .collect();
            Err(StepError::Root(Said(if said.is_empty() {
                format!(
                    "it exited with {}",
                    ran.code.map_or_else(|| "a signal".to_owned(), |c| c.to_string())
                )
            } else {
                said.join("; ")
            })))
        }
        Err(err) => Err(StepError::Root(err.said)),
    }
}

/// The pin helper's arguments for `args`, and its stdin: the CA, the proxy and its `no_proxy`, the
/// PAC file, `none` for each the plan lacks.
fn pins(args: &[RootArg]) -> (Vec<String>, Option<&str>) {
    let ca = args.iter().find_map(RootArg::stdin);
    let proxy = args.iter().find_map(|arg| match arg {
        RootArg::Docker { proxy, no_proxy }
        | RootArg::Nix { proxy, no_proxy }
        | RootArg::ChromeProxy { proxy, no_proxy } => Some((proxy.as_str(), no_proxy.as_str())),
        _ => None,
    });
    let apt = args.iter().find_map(|arg| match arg {
        RootArg::Apt(proxy) => Some(proxy.as_str()),
        _ => None,
    });
    let (proxy, no_proxy) = proxy.or(apt.map(|proxy| (proxy, ""))).unwrap_or(("none", ""));
    let pac = args
        .iter()
        .find_map(|arg| match arg {
            RootArg::ChromePac(pac) => Some(pac.as_str()),
            _ => None,
        })
        .unwrap_or("none");
    let line = vec![
        if ca.is_some() {
            "ca=-".to_owned()
        } else {
            "ca=none".to_owned()
        },
        format!("proxy={proxy}"),
        format!("no-proxy={no_proxy}"),
        format!("pac={pac}"),
    ];
    (line, ca)
}

/// The plan's root parts, then every other root part off, so one dropped from the profile (the
/// kill switch, the CA) goes at the next `on`. A value an admin hasn't pinned is pinned first,
/// which asks for the password, and the parts go again.
pub(super) fn put(m: &mut dyn Machine, args: &[RootArg]) -> Result<(), StepError> {
    let stdin = args.iter().find_map(RootArg::stdin);
    let mut line: Vec<String> = args.iter().map(RootArg::arg).collect();
    for name in ROOT_PARTS {
        if !args.iter().any(|arg| arg.name() == name) {
            line.push(format!("{name}=off"));
        }
    }
    match pkexec(m, HELPER, &line, stdin) {
        Err(StepError::NotPinned) => {
            let (pin_line, pin_stdin) = pins(args);
            pkexec(m, PIN_HELPER, &pin_line, pin_stdin)?;
            pkexec(m, HELPER, &line, stdin).map(drop)
        }
        done => done.map(drop),
    }
}

/// Takes every root part away, the kill switch too unless `keep_kill_switch`.
pub(super) fn remove(m: &mut dyn Machine, keep_kill_switch: bool) -> Result<(), StepError> {
    if !keep_kill_switch {
        return pkexec(m, HELPER, &["off".to_owned()], None).map(drop);
    }
    let line: Vec<String> = ROOT_PARTS
        .iter()
        .filter(|name| **name != "killswitch")
        .map(|name| format!("{name}=off"))
        .collect();
    pkexec(m, HELPER, &line, None).map(drop)
}

/// The tunnel's latest handshake from the helper's `stats` (`PUBLIC-KEY EPOCH` lines).
pub fn handshake(m: &mut dyn Machine) -> Option<u64> {
    let ran = pkexec(m, HELPER, &["stats".to_owned()], None).ok()?;
    ran.stdout
        .lines()
        .filter_map(|line| line.split_whitespace().nth(1)?.parse::<u64>().ok())
        .max()
}
