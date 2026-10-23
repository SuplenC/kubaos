//! Bitwarden desktop's SSH agent: it serves the vault's keys once the app is unlocked, and the
//! session's `SSH_AUTH_SOCK` points at it (`home/secrets.nix`).

use serde::Deserialize;
use serde::Serialize;

use super::Cmd;
use super::Host;
use super::Paths;
use super::Progress;

/// What the agent's socket answered.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum Agent {
    /// No socket: the app isn't running, or its SSH agent is off.
    Off,
    /// Running, with no keys: the app's vault is locked.
    Locked,
    /// Running, serving these fingerprints.
    Serving { keys: Vec<String> },
    /// A socket that doesn't answer.
    Dead,
}

/// Asks the agent which keys it has.
pub fn probe(host: &mut dyn Host, paths: &Paths) -> Agent {
    if !host.is_socket(&paths.agent) {
        return Agent::Off;
    }
    let cmd = Cmd::new("timeout")
        .args(["20", "ssh-add", "-l", "-E", "sha256"])
        .env("SSH_AUTH_SOCK", &paths.agent.display().to_string());
    match host.run(&cmd) {
        Ok(r) if r.ok() => Agent::Serving {
            keys: r
                .stdout
                .lines()
                .filter_map(|l| l.split_whitespace().nth(1))
                .map(str::to_string)
                .collect(),
        },
        Ok(r) if r.code == Some(1) => Agent::Locked,
        _ => Agent::Dead,
    }
}

/// Says whether the agent serves every key in `fingerprints`.
pub fn check(host: &mut dyn Host, paths: &Paths, fingerprints: &[String], scope: &str, p: &mut Progress) {
    p.step(format!("Bitwarden desktop SSH agent ({})", paths.show(&paths.agent)));
    match probe(host, paths) {
        Agent::Off => {
            p.warn(
                "no agent socket. Start Bitwarden desktop, log in, then Settings -> SSH agent -> turn on \"Enable SSH agent\" (off by default; the app has no config-file switch for it).",
            );
            p.summary("agent  NOT running: start Bitwarden desktop and enable its SSH agent");
            return;
        }
        Agent::Serving { keys } => {
            let mut missing = false;
            for want in fingerprints {
                if !keys.contains(want) {
                    p.warn(format!("agent doesn't offer {want} (vault not synced in the app?)"));
                    missing = true;
                }
            }
            p.ok(format!("agent answers with {} key(s)", keys.len()));
            p.summary(if missing {
                "agent  running, but some vault keys are missing (sync the desktop app)".to_string()
            } else {
                format!("agent  running, serves every SSH key in {scope}")
            });
        }
        Agent::Locked => {
            p.warn("agent has no keys: unlock Bitwarden desktop (keys load after the first unlock)");
            p.summary("agent  running, vault locked in the desktop app");
        }
        Agent::Dead => {
            p.warn("agent socket doesn't answer: restart Bitwarden desktop");
            p.summary("agent  socket dead: restart Bitwarden desktop");
        }
    }
    let sock = paths.agent.display().to_string();
    if paths.auth_sock.as_deref() != Some(sock.as_str()) {
        p.warn(format!(
            "SSH_AUTH_SOCK is '{}', not the Bitwarden agent: log out and in after home-manager switch",
            paths.auth_sock.as_deref().unwrap_or("")
        ));
    }
}
