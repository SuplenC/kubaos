//! gh: the token from the item named `gh-token` ([`super::vault::Names::gh_item`]) into gh's own
//! keyring entry.

use super::Cmd;
use super::Machine;
use super::Progress;
use super::vault::Vault;

pub fn apply(m: &mut dyn Machine, vault: &Vault, p: &mut Progress) {
    let item = &vault.names.gh_item;
    let tokens = vault.gh_tokens();
    if tokens.len() > 1 {
        p.warn(format!("more than one item \"{item}\" in the vault; gh gets the first"));
    }
    let host = tokens
        .first()
        .map(|i| i.field("host"))
        .filter(|h| !h.is_empty())
        .unwrap_or("github.com")
        .to_string();
    let token = tokens.first().and_then(|i| i.token.clone());

    p.step("gh: token from the vault into the system keyring");
    let Some(token) = token.filter(|t| !t.0.is_empty()) else {
        p.info(format!("no item \"{item}\" in the vault; skipped (gh auth login works by hand too)"));
        p.summary(format!("gh     no {item} item, skipped"));
        return;
    };
    if !m.has("gh") {
        p.warn("gh not installed");
        return;
    }
    let current = m
        .run(&Cmd::new("gh").args(["auth", "token", "--hostname", &host]))
        .ok()
        .filter(super::Ran::ok)
        .map(|r| r.stdout.trim().to_string());
    if current.as_deref() == Some(token.0.as_str()) {
        p.ok(format!("gh already uses this token for {host}"));
    } else {
        // gh stores the token in the Secret Service keyring; with no keyring it falls back to
        // a plaintext ~/.config/gh/hosts.yml, which the status check below catches.
        let login = Cmd::new("gh")
            .args(["auth", "login", "--hostname", &host, "--git-protocol", "ssh", "--with-token"])
            .secret_stdin(&token.0);
        if !m.run(&login).is_ok_and(|r| r.ok()) {
            p.warn("gh auth login failed (token revoked or missing scopes? gh needs repo, read:org)");
            p.summary(format!("gh     login FAILED for {host}"));
            return;
        }
    }
    let status = m
        .run(&Cmd::new("gh").args(["auth", "status", "--hostname", &host]))
        .map(|r| format!("{}{}", r.stdout, r.stderr))
        .unwrap_or_default();
    if status.contains("(keyring)") {
        p.ok(format!("gh logged in to {host}, token in the keyring"));
        p.summary(format!("gh     logged in to {host} (token in the keyring)"));
    } else {
        let _ = m.run(&Cmd::new("gh").args(["auth", "logout", "--hostname", &host]));
        p.warn("gh had no keyring and wrote the token to a file: logged out again. Is gnome-keyring running?");
        p.summary("gh     NOT set up: no Secret Service keyring");
    }
}
