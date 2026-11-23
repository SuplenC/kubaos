//! How kubaos-core starts a program that may run git: without git's repository variables, and,
//! in kubaos-core's own tests, in the test's sandbox.
//!
//! A scaffolder runs `git init`, Nix reads a flake from its git checkout, and the state
//! directory's index sync runs git itself; each would act on the repository a `GIT_DIR` it
//! inherited names. In a test build, each also runs in a sandbox of the test thread's own
//! ([`crate::sandbox`]), so a test that drives this code starts nothing with the developer's
//! environment, home or `~/.gitconfig`: the test's own seam, with no switch for a test to forget.
//!
//! # Owns
//!
//! - [`command`], [`git`] and [`prepare`], and the test thread's sandbox ([`testing`]).
//!
//! # Not here
//!
//! - Which of git's variables are taken away: `kubaos_git`.

use std::path::Path;
use std::process::Command;

/// `program`, ready to start: [`prepare`]d.
pub(crate) fn command(program: impl AsRef<std::ffi::OsStr>) -> Command {
    let mut command = Command::new(program);
    prepare(&mut command);
    command
}

/// `git -C dir`, ready to start: [`prepare`]d.
pub(crate) fn git(dir: &Path) -> Command {
    let mut command = kubaos_git::at(dir);
    prepare(&mut command);
    command
}

/// `command` without git's repository variables, and in a test build, in the test thread's
/// sandbox first, so a variable a test hands it ([`testing::pass`]) meets the same clearing an
/// inherited one does.
pub(crate) fn prepare(command: &mut Command) -> &mut Command {
    #[cfg(test)]
    testing::confine(command);
    kubaos_git::clear(command)
}

#[cfg(test)]
pub(crate) mod testing {
    //! The test thread's sandbox, made the first time the code under test starts a program, with
    //! git and `ssh-keygen` linked in for the fixture repositories' git.

    use std::cell::RefCell;
    use std::ffi::OsString;
    use std::fs;
    use std::process::Command;

    use crate::sandbox::Sandbox;

    struct Thread {
        _dir: crate::sandbox::TempDir,
        sandbox: Sandbox,
        passed: Vec<(OsString, OsString)>,
    }

    thread_local! {
        static THREAD: RefCell<Option<Thread>> = const { RefCell::new(None) };
    }

    fn with<T>(f: impl FnOnce(&mut Thread) -> T) -> T {
        THREAD.with(|cell| {
            let mut cell = cell.borrow_mut();
            let thread = cell.get_or_insert_with(|| {
                let dir = crate::sandbox::tempdir().expect("a folder for the thread's sandbox");
                let home = dir.path().join("home");
                fs::create_dir_all(&home).expect("make the sandbox's home");
                let sandbox = Sandbox::new(&dir.path().join("sandbox"), &home, "tester");
                sandbox.link("git");
                sandbox.link("ssh-keygen");
                Thread {
                    _dir: dir,
                    sandbox,
                    passed: Vec::new(),
                }
            });
            f(thread)
        })
    }

    /// `command` given the test thread's sandbox, and the variables the test passes on.
    pub(super) fn confine(command: &mut Command) {
        with(|thread| {
            thread.sandbox.confine(command);
            for (name, value) in &thread.passed {
                command.env(name, value);
            }
        });
    }

    /// `name=value` handed to every program the code under test starts on this thread from now
    /// on, as a variable inherited from where KubaOS ran would be.
    pub(crate) fn pass(name: &str, value: impl Into<OsString>) {
        with(|thread| thread.passed.push((name.into(), value.into())));
    }
}
