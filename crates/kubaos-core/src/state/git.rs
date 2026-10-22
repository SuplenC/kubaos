//! Keeping the state directory visible to Nix when the user flake is a git repository.
//!
//! Nix copies a flake that lives in a git work tree from git's index: a file git does not know is
//! not in the flake, however it sits on disk. So after every write to the state directory, each
//! new file gets an intent-to-add entry (`git add --intent-to-add`), which lists it without
//! staging its content, and each deleted file leaves the index (`git rm --cached`). Nothing is
//! committed, and a changed file's staged content stays as the user left it.
//!
//! # Owns
//!
//! - Finding out whether the user flake is in a git work tree, and the two git commands.
//!
//! # Not here
//!
//! - Writing the state directory: `packages.rs`, `suites.rs` and `state.rs`.

use std::ffi::OsStr;
use std::ffi::OsString;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;
use std::process::Stdio;

/// What [`super::StateDir::sync_git`] did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GitSync {
    /// The user flake is not in a git work tree (or git is not installed), so Nix reads the
    /// folder as it is and there was nothing to do.
    NotARepo,
    /// Git's index now lists every file of the state directory, and no deleted one.
    Synced,
}

/// A git command failed, so Nix may not see the state directory as it is on disk. The files
/// themselves are written.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum GitError {
    /// Git could not be run.
    #[error("running git in {}", flake.display())]
    Spawn {
        flake: PathBuf,
        #[source]
        source: io::Error,
    },
    /// Git ran and failed.
    #[error("`git {command}` in {} failed: {stderr}", flake.display())]
    Failed {
        flake: PathBuf,
        command: String,
        stderr: String,
    },
}

/// Records the state directory `dir`, inside the user flake `flake`, in git's index.
pub(super) fn sync(flake: &Path, dir: &Path) -> Result<GitSync, GitError> {
    if !is_work_tree(flake) {
        return Ok(GitSync::NotARepo);
    }
    let rel = dir.strip_prefix(flake).unwrap_or(dir).as_os_str();
    if dir.is_dir() {
        git(flake, &["add".as_ref(), "--intent-to-add".as_ref(), "--".as_ref(), rel])?;
    }
    let deleted = git(
        flake,
        &[
            "ls-files".as_ref(),
            "-z".as_ref(),
            "--deleted".as_ref(),
            "--".as_ref(),
            rel,
        ],
    )?;
    let deleted: Vec<OsString> = deleted
        .stdout
        .split(|b| *b == 0)
        .filter(|path| !path.is_empty())
        .map(|path| OsString::from(String::from_utf8_lossy(path).into_owned()))
        .collect();
    if !deleted.is_empty() {
        let mut args: Vec<&OsStr> = vec!["rm".as_ref(), "--cached".as_ref(), "--quiet".as_ref(), "--".as_ref()];
        args.extend(deleted.iter().map(OsString::as_os_str));
        git(flake, &args)?;
    }
    Ok(GitSync::Synced)
}

/// Whether `flake` is inside a git work tree. A missing folder, or no git, is not.
fn is_work_tree(flake: &Path) -> bool {
    if !flake.is_dir() {
        return false;
    }
    Command::new("git")
        .arg("-C")
        .arg(flake)
        .args(["rev-parse", "--is-inside-work-tree"])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .is_ok_and(|out| out.status.success() && out.stdout.starts_with(b"true"))
}

fn git(flake: &Path, args: &[&OsStr]) -> Result<Output, GitError> {
    let out = Command::new("git")
        .arg("-C")
        .arg(flake)
        .args(args)
        .stdin(Stdio::null())
        .output()
        .map_err(|source| GitError::Spawn {
            flake: flake.to_path_buf(),
            source,
        })?;
    if out.status.success() {
        return Ok(out);
    }
    Err(GitError::Failed {
        flake: flake.to_path_buf(),
        command: args.iter().map(|a| a.to_string_lossy()).collect::<Vec<_>>().join(" "),
        stderr: String::from_utf8_lossy(&out.stderr).trim().to_owned(),
    })
}

#[cfg(test)]
mod tests;
