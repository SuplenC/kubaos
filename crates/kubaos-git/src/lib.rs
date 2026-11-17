//! Git as KubaOS runs it: on the repository it names, never one its environment points at.
//!
//! Git obeys `GIT_DIR`, `GIT_WORK_TREE`, `GIT_INDEX_FILE` and the rest of its variables over
//! `-C`, and a process started from inside a git command (a hook, `git rebase --exec`, an editor
//! a commit opened) or from a shell that exported them carries them. A `git -C <flake> add` run
//! there stages in that other repository, and a `git init` re-initialises it. So every git
//! command KubaOS runs starts with none of git's repository variables, and finds its repository
//! from the folder it is given. The variables a user sets for every git (how it reaches a
//! remote, who commits) reach it as set.
//!
//! # Owns
//!
//! - [`at`] and [`anywhere`], the git commands, and [`clear`], which takes git's repository
//!   variables away from any command, for the programs that run git themselves.

use std::env;
use std::ffi::OsString;
use std::path::Path;
use std::process::Command;
use std::process::Stdio;

/// The variables that point git at a repository, an index or objects other than the one it would
/// find itself (`git rev-parse --local-env-vars`), and those that override its configuration
/// for one command (`git -c` and `GIT_CONFIG_COUNT`'s entries, which a parent git passes on).
/// [`clear`] takes these away even when this process hasn't got them, so a variable set on the
/// command before it is gone too.
///
/// Not the variables a user sets on purpose for every git they run: the transport's and the
/// credentials' (`GIT_SSH_COMMAND`, `GIT_ASKPASS`, `GIT_SSL_CAINFO`, `GIT_TERMINAL_PROMPT`), the
/// identity's (`GIT_AUTHOR_*`, `GIT_COMMITTER_*`), tracing (`GIT_TRACE*`), and the configuration
/// files they chose (`GIT_CONFIG_GLOBAL`, `GIT_CONFIG_SYSTEM`, `GIT_CONFIG_NOSYSTEM`). A private
/// flake input or an `ls-remote` needs the first, and none of them names a repository.
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
];

/// The numbered entries `GIT_CONFIG_COUNT` counts: `GIT_CONFIG_KEY_<n>` and `GIT_CONFIG_VALUE_<n>`.
const CONFIG_ENTRY: &[&str] = &["GIT_CONFIG_KEY_", "GIT_CONFIG_VALUE_"];

/// `git -C dir`, with none of git's repository variables and nothing on stdin: its repository is
/// the one `dir` is in, as git finds it from there.
#[must_use]
pub fn at(dir: &Path) -> Command {
    let mut command = anywhere();
    command.arg("-C").arg(dir);
    command
}

/// `git`, with none of git's repository variables and nothing on stdin, for a command that needs
/// no repository: `ls-remote` with a URL, `config` as every repository sees it.
#[must_use]
pub fn anywhere() -> Command {
    let mut command = Command::new("git");
    clear(&mut command).stdin(Stdio::null());
    command
}

/// `command` without git's repository variables: every one of [`REPOSITORY`], and every
/// numbered configuration entry this process or the command has. For the programs KubaOS starts
/// that may run git themselves, such as a migration or Nix reading a flake from a git checkout.
/// The rest of git's variables reach the program as the user set them.
pub fn clear(command: &mut Command) -> &mut Command {
    let inherited = env::vars_os().map(|(name, _)| name);
    let set = command.get_envs().map(|(name, _)| name.to_owned());
    let entries: Vec<OsString> = inherited.chain(set).filter(is_config_entry).collect();
    for name in REPOSITORY.iter().map(OsString::from).chain(entries) {
        command.env_remove(name);
    }
    command
}

fn is_config_entry(name: &OsString) -> bool {
    let name = name.as_encoded_bytes();
    CONFIG_ENTRY.iter().any(|prefix| name.starts_with(prefix.as_bytes()))
}

#[cfg(test)]
mod tests;
