//! Real git repositories for the updater's tests: an `origin`, a checkout cloned from it, and a
//! throwaway SSH signing key with its `allowed_signers`.

use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

/// Runs git in `dir` with a test identity, and returns its trimmed output.
pub fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args([
            "-c",
            "user.name=t",
            "-c",
            "user.email=t@kubaos",
            "-c",
            "commit.gpgsign=false",
        ])
        .args(["-c", "tag.gpgsign=false", "-c", "init.defaultBranch=main"])
        .args(args)
        .output()
        .expect("run git");
    assert!(
        out.status.success(),
        "git {args:?} in {}: {}",
        dir.display(),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_owned()
}

/// A throwaway SSH key and an `allowed_signers` that trusts it.
pub struct Key {
    pub private: PathBuf,
    pub allowed_signers: PathBuf,
}

/// Makes a key in `dir`.
pub fn key(dir: &Path) -> Key {
    fs::create_dir_all(dir).expect("make the key folder");
    let private = dir.join("key");
    let out = Command::new("ssh-keygen")
        .args(["-q", "-t", "ed25519", "-N", "", "-C", "test", "-f"])
        .arg(&private)
        .output()
        .expect("run ssh-keygen");
    assert!(
        out.status.success(),
        "ssh-keygen: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let public = fs::read_to_string(dir.join("key.pub")).expect("read the public key");
    let allowed_signers = dir.join("allowed_signers");
    fs::write(&allowed_signers, format!("t@kubaos namespaces=\"git\" {public}")).expect("write allowed_signers");
    Key {
        private,
        allowed_signers,
    }
}

/// The `-c` options that sign with `key`.
fn signing(key: &Key) -> Vec<String> {
    vec![
        "-c".into(),
        "gpg.format=ssh".into(),
        "-c".into(),
        format!("user.signingkey={}", key.private.display()),
    ]
}

/// A bare `origin` in `dir` with one commit on main: `VERSION`, `CODENAME` and `files`.
pub fn origin(dir: &Path, version: &str, files: &[(&str, &str)]) -> PathBuf {
    let work = dir.join("work");
    fs::create_dir_all(&work).expect("make the work tree");
    git(&work, &["init", "-q"]);
    commit(
        &work,
        "first",
        &[&[("VERSION", version), ("CODENAME", "shinjuku")], files].concat(),
        None,
    );
    let bare = dir.join("origin.git");
    git(
        dir,
        &[
            "clone",
            "-q",
            "--bare",
            work.to_str().expect("a UTF-8 path"),
            "origin.git",
        ],
    );
    git(
        &work,
        &["remote", "add", "origin", bare.to_str().expect("a UTF-8 path")],
    );
    bare
}

/// Commits `files` in the work tree `work`, signed with `key` when given, and returns the commit.
pub fn commit(work: &Path, message: &str, files: &[(&str, &str)], key: Option<&Key>) -> String {
    for (path, text) in files {
        let path = work.join(path);
        fs::create_dir_all(path.parent().expect("a parent")).expect("make the folder");
        fs::write(&path, text).expect("write the file");
    }
    git(work, &["add", "-A"]);
    let mut args: Vec<String> = key.map(signing).unwrap_or_default();
    args.extend(["commit".into(), "-q".into(), "-m".into(), message.into()]);
    if key.is_some() {
        args.push("-S".into());
    }
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    git(work, &refs);
    git(work, &["rev-parse", "HEAD"])
}

/// Tags HEAD of `work` as `name`, annotated, signed with `key` when given.
pub fn tag(work: &Path, name: &str, key: Option<&Key>) {
    let mut args: Vec<String> = key.map(signing).unwrap_or_default();
    args.extend(["tag".into(), "-m".into(), name.into()]);
    if key.is_some() {
        args.push("-s".into());
    }
    args.push(name.into());
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    git(work, &refs);
}

/// A checkout of `origin` at `path`, on a `release` branch at origin's main.
pub fn checkout(origin: &Path, path: &Path) {
    fs::create_dir_all(path.parent().expect("a parent")).expect("make the parent");
    let parent = path.parent().expect("a parent");
    git(
        parent,
        &[
            "clone",
            "-q",
            origin.to_str().expect("a UTF-8 path"),
            path.to_str().expect("a UTF-8 path"),
        ],
    );
    git(path, &["checkout", "-q", "-B", "release"]);
}
