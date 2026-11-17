//! Git as KubaOS runs it: on the repository it names, never one its environment points at.
//!
//! Git obeys `GIT_DIR`, `GIT_WORK_TREE`, `GIT_INDEX_FILE` and the rest of its variables over
//! `-C`, and a process started from inside a git command (a hook, `git rebase --exec`, an editor
//! a commit opened) or from a shell that exported them carries them. A `git -C <flake> add` run
//! there stages in that other repository, and a `git init` re-initialises it. So every git
//! command KubaOS runs starts with none of git's variables, and finds its repository from the
//! folder it is given.
//!
//! # Owns
//!
//! - [`at`] and [`anywhere`], the git commands, and [`clear`], which takes git's variables away
//!   from any command, for the programs that run git themselves.

use std::env;
use std::ffi::OsString;
use std::path::Path;
use std::process::Command;
use std::process::Stdio;

/// The variables that point git at a repository, an index, objects or configuration other than
/// the one it would find itself: `git rev-parse --local-env-vars`, and those that name the
/// configuration files. [`clear`] takes these away even when this process hasn't got them, so
/// a variable set on the command before it is gone too.
pub const REPOSITORY: &[&str] = &[
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_IMPLICIT_WORK_TREE",
    "GIT_COMMON_DIR",
    "GIT_INDEX_FILE",
    "GIT_OBJECT_DIRECTORY",
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    "GIT_GRAFT_FILE",
    "GIT_SHALLOW_FILE",
    "GIT_NO_REPLACE_OBJECTS",
    "GIT_REPLACE_REF_BASE",
    "GIT_NAMESPACE",
    "GIT_PREFIX",
    "GIT_INTERNAL_SUPER_PREFIX",
    "GIT_CEILING_DIRECTORIES",
    "GIT_DISCOVERY_ACROSS_FILESYSTEM",
    "GIT_CONFIG",
    "GIT_CONFIG_PARAMETERS",
    "GIT_CONFIG_COUNT",
    "GIT_CONFIG_GLOBAL",
    "GIT_CONFIG_SYSTEM",
    "GIT_CONFIG_NOSYSTEM",
];

/// `git -C dir`, with none of git's variables and nothing on stdin: its repository is the one
/// `dir` is in, as git finds it from there.
#[must_use]
pub fn at(dir: &Path) -> Command {
    let mut command = anywhere();
    command.arg("-C").arg(dir);
    command
}

/// `git`, with none of git's variables and nothing on stdin, for a command that needs no
/// repository: `ls-remote` with a URL, `config` as every repository sees it.
#[must_use]
pub fn anywhere() -> Command {
    let mut command = Command::new("git");
    clear(&mut command).stdin(Stdio::null());
    command
}

/// `command` without git's variables: every `GIT_*` this process has, and every one of
/// [`REPOSITORY`]. For the programs KubaOS starts that may run git themselves, such as a
/// migration or Nix reading a flake from a git checkout.
///
/// All of this process's, not only [`REPOSITORY`]: git adds variables with its releases, and
/// KubaOS sets the ones it needs on the command itself, after this.
pub fn clear(command: &mut Command) -> &mut Command {
    let inherited = env::vars_os().map(|(name, _)| name).filter(is_git);
    let names: Vec<OsString> = inherited.chain(REPOSITORY.iter().map(OsString::from)).collect();
    for name in names {
        command.env_remove(name);
    }
    command
}

fn is_git(name: &OsString) -> bool {
    name.as_encoded_bytes().starts_with(b"GIT_")
}

#[cfg(test)]
mod tests;
