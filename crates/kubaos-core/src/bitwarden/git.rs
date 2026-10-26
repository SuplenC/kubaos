//! git: commits signed with the SSH key whose item has `signing=true`, through the agent.
//! `signing=verify` keys only go into `allowed_signers`, so old signatures still verify.

use super::Cmd;
use super::Error;
use super::KeyInfo;
use super::Machine;
use super::Paths;
use super::Progress;
use super::list;
use super::ssh::Key;
use super::ssh::on_disk;
use super::ssh::public_key;
use super::ssh::write_if_changed;
use super::vault::Item;

/// `signing` says yes.
fn truthy(v: &str) -> bool {
    matches!(v.to_ascii_lowercase().as_str(), "true" | "yes" | "1")
}

/// An address `allowed_signers` can take: no space, no quote, one `@` with something around it.
fn email_ok(e: &str) -> bool {
    !e.contains(|c: char| c.is_whitespace() || c == '"')
        && e.split_once('@').is_some_and(|(u, d)| !u.is_empty() && !d.is_empty())
}

/// git config as every repository sees it: system, `~/.config/git/config` and `~/.gitconfig`.
/// Not `--global`, which reads only `~/.gitconfig` once that exists.
fn git_config(m: &mut dyn Machine, args: &[&str]) -> String {
    let cmd = Cmd::new("git")
        .args(["-C", "/", "config"])
        .args(args.iter().copied())
        .env("GIT_DIR", "/nonexistent");
    m.run(&cmd).ok().filter(super::Ran::ok).map(|r| r.stdout).unwrap_or_default()
}

/// The key git signs with: the oldest with `signing=true` and a public key.
fn signer(keys: &[Key]) -> Option<&Key> {
    keys.iter().find(|k| truthy(k.item.field("signing")) && k.item.public_key.as_deref().is_some_and(|k| !k.is_empty()))
}

/// Marks the overview's signing key.
pub fn mark_signing(infos: &mut [KeyInfo], keys: &[Key]) {
    let Some(s) = signer(keys) else { return };
    if let Some(i) = infos.iter_mut().find(|i| i.file.as_deref() == Some(s.file.as_str())) {
        i.signing = true;
    }
}

/// Why git can't sign with the signing key's `.pub` as it is on disk, and the fix; `None` when
/// the file holds the vault's key.
fn pub_mismatch(m: &mut dyn Machine, paths: &Paths, sign: &Key) -> Option<String> {
    const LEFT: &str = "so signing is left as it was";
    const REFRESH: &str = "kubaos bitwarden refresh --only ssh,git";
    let file = &sign.file;
    let Some(want) = sign.item.public_key.as_deref().and_then(|k| public_key(k).ok()) else {
        return Some(format!(
            "its public key can't be used (see its warning), {LEFT}; fix the public key in Bitwarden, then run `{REFRESH}`"
        ));
    };
    let pub_path = paths.ssh.join(format!("{file}.pub"));
    let have = on_disk(m, &pub_path);
    match have {
        Some(have) if have == want => None,
        None => Some(format!("~/.ssh/{file}.pub isn't on disk, {LEFT}; run `{REFRESH}` to write it")),
        // The SSH step never replaces the `.pub` of a private key on disk, so a refresh alone
        // doesn't fix this one.
        Some(_) if m.exists(&paths.ssh.join(file)) => Some(format!(
            "~/.ssh/{file}.pub belongs to the private key ~/.ssh/{file} on disk and holds another key, {LEFT}; \
             in Bitwarden, add a text field `file` = `{file}-bitwarden` to {}, then run `{REFRESH}`",
            sign.item.label()
        )),
        Some(_) => Some(format!(
            "~/.ssh/{file}.pub holds another key, from an older pull, {LEFT}; run `{REFRESH}` to write the vault's key"
        )),
    }
}

/// Writes `signing.gitconfig` and `allowed_signers`, or removes the first when no key signs.
///
/// # Errors
///
/// A file can't be written.
pub fn apply(m: &mut dyn Machine, paths: &Paths, keys: &[Key], scope: &str, p: &mut Progress) -> Result<(), Error> {
    p.step("git: commit signing with an SSH key from the vault");
    let emails = git_config(m, &["--get", "user.email"]);
    let mut allowed = String::new();
    let mut marked: Vec<String> = vec![];
    for key in keys.iter().filter(|k| k.item.has_field("signing")) {
        let item: &Item = &key.item;
        let Some(public) = item.public_key.as_deref().and_then(|k| public_key(k).ok()) else {
            continue;
        };
        let v = item.field("signing");
        if truthy(v) {
            marked.push(item.label());
        } else if !v.eq_ignore_ascii_case("verify") {
            continue;
        }
        let extra = item.field("emails").replace(',', " ");
        for email in emails.split_whitespace().chain(extra.split_whitespace()).filter(|e| email_ok(e)) {
            allowed.push_str(&format!("{email} namespaces=\"git\" {public}\n"));
        }
    }
    let Some(sign) = signer(keys) else {
        p.warn(format!("no SSH key in {scope} has the field signing=true: commits stay unsigned"));
        m.remove(&paths.signing());
        p.summary("git    no signing key (add a field signing=true to one SSH key item)");
        return Ok(());
    };
    let name = sign.item.label();
    if marked.len() > 1 {
        p.warn(format!(
            "{} SSH keys have signing=true: {}. Signing with the oldest, {name}; set signing=verify on the others",
            marked.len(),
            list(&marked)
        ));
    }
    let pub_path = paths.ssh.join(format!("{}.pub", sign.file));
    if let Some(problem) = pub_mismatch(m, paths, sign) {
        p.warn(format!("{name} has signing=true, but {problem}"));
        return Ok(());
    }
    if allowed.is_empty() {
        p.warn("no email for allowed_signers: set user.email in config/git/config");
    }
    let signers = paths.git.join("allowed_signers");
    write_if_changed(m, &signers, &allowed, 0o644)?;
    let config = format!(
        "# Generated by `kubaos bitwarden` from the Bitwarden SSH key {name}. Do not edit.\n\
         # Included by config/git/config. git signs through the agent in SSH_AUTH_SOCK.\n\
         [user]\n\tsigningKey = {}\n[gpg]\n\tformat = ssh\n[gpg \"ssh\"]\n\tallowedSignersFile = {}\n\
         [commit]\n\tgpgSign = true\n",
        pub_path.display(),
        signers.display()
    );
    write_if_changed(m, &paths.signing(), &config, 0o644)?;
    if git_config(m, &["--get-all", "include.path"]).contains("git-bitwarden/signing.gitconfig") {
        p.ok(format!("signing with ~/.ssh/{}.pub ({name}); git config includes it", sign.file));
    } else {
        p.warn("config/git/config doesn't include ~/.config/git-bitwarden/signing.gitconfig");
    }
    p.ok(format!("allowed signers: {} line(s)", allowed.lines().count()));
    p.summary(format!("git    commits signed with {name} (~/.ssh/{}.pub)", sign.file));
    Ok(())
}
