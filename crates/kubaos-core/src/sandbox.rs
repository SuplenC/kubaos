//! The tests' wall around the processes they start: a program a test host runs (an app's `run`
//! action, a recorded script, a stand-in for a host command, the root helper) gets an
//! environment with no route to the live session or the machine.
//!
//! Its `PATH` is one folder, the sandbox's `bin`, and nothing else. In it, every command an app's
//! script or a host can call that changes the machine or the session (systemd, logind, apt, dpkg,
//! debconf, pkexec, sudo, T3, Docker, the group tools, the openers, nftables, NetworkManager) is a
//! stand-in that logs its call and changes nothing, and the harmless tools a script needs (a shell,
//! coreutils, jq, tar, sed, awk, grep, python) are links to the real ones. Any other command is not
//! found, so a script that reaches for one fails, loudly, rather than run the real one. The session
//! and system buses point at a socket that isn't there, and `XDG_RUNTIME_DIR` is a folder of the
//! sandbox's own, so even a program called by its full path finds no user manager, logind or polkit
//! to talk to. `HOME` and the `XDG_*` folders are the test's.
//!
//! # Owns
//!
//! - [`Sandbox`], the stand-ins ([`STUBBED`]) and the tools ([`TOOLS`]).
//! - Writing a program a test starts so that it starts at once ([`program`], [`settle`]).
//!
//! # Not here
//!
//! - The fake machine that runs its programs in one: `apps/host/fake.rs`.

use std::env;
use std::ffi::OsStr;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::fs::symlink;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;

/// A D-Bus address with no bus behind it, for the session's and the system's.
pub const NOWHERE: &str = "unix:path=/nonexistent";

/// The commands that change the machine or the session, or reach out of it, that an app's script
/// or a host may call: each is a stand-in that logs its call and exits 0, or 1 for those in
/// [`FINDS_NOTHING`].
pub const STUBBED: &[&str] = &[
    "systemctl",
    "systemd-run",
    "journalctl",
    "loginctl",
    "busctl",
    "dbus-send",
    "gdbus",
    "pgrep",
    "pkill",
    "pidof",
    "killall",
    "apt",
    "apt-get",
    "apt-mark",
    "apt-key",
    "dpkg",
    "dpkg-query",
    "dpkg-reconfigure",
    "debconf-set-selections",
    "debconf-communicate",
    "pkexec",
    "sudo",
    "su",
    "runuser",
    "t3",
    "t3code",
    "docker",
    "gpasswd",
    "usermod",
    "useradd",
    "userdel",
    "groupadd",
    "groupdel",
    "setsid",
    "nohup",
    "xdg-open",
    "gio",
    "notify-send",
    "gsettings",
    "hyprctl",
    "google-chrome",
    "google-chrome-stable",
    "google-chrome-beta",
    "chromium",
    "bitwarden",
    "claude",
    "curl",
    "wget",
    "nix",
    "home-manager",
    "reboot",
    "shutdown",
    "poweroff",
    "nft",
    "nmcli",
];

/// The stand-ins that answer as a machine where nothing matches: no process found.
pub const FINDS_NOTHING: &[&str] = &["pgrep", "pkill", "pidof", "killall"];

/// The harmless tools a script may use, linked from the real `PATH`: they read and write files
/// and text, and reach nothing else.
pub const TOOLS: &[&str] = &[
    "sh",
    "bash",
    "dash",
    "env",
    "true",
    "false",
    "test",
    "[",
    "echo",
    "printf",
    "cat",
    "cp",
    "mv",
    "rm",
    "ln",
    "ls",
    "mkdir",
    "rmdir",
    "touch",
    "chmod",
    "readlink",
    "realpath",
    "basename",
    "dirname",
    "head",
    "tail",
    "tr",
    "cut",
    "sort",
    "uniq",
    "wc",
    "tee",
    "sed",
    "awk",
    "gawk",
    "grep",
    "find",
    "xargs",
    "tar",
    "gzip",
    "zstd",
    "xz",
    "jq",
    "mktemp",
    "date",
    "sleep",
    "seq",
    "id",
    "whoami",
    "stat",
    "install",
    "cmp",
    "diff",
    "getent",
    "uname",
    "sha256sum",
    "base64",
    "od",
    "python3",
];

/// A test's walled-off environment, in a folder of its own: its stand-ins and tools in `bin`, the
/// stand-ins' log, its runtime and temporary folders.
#[derive(Clone, Debug)]
pub struct Sandbox {
    root: PathBuf,
    home: PathBuf,
    user: String,
}

impl Sandbox {
    /// A sandbox in `root` for `user`, whose home is `home`; `home` itself is left as it is.
    ///
    /// # Panics
    ///
    /// When its folders, stand-ins or links can't be made.
    #[must_use]
    pub fn new(root: &Path, home: &Path, user: impl Into<String>) -> Self {
        let sandbox = Self {
            root: root.to_path_buf(),
            home: home.to_path_buf(),
            user: user.into(),
        };
        for dir in [sandbox.bin(), sandbox.tmp(), sandbox.root.join("xdg")] {
            fs::create_dir_all(dir).expect("make the sandbox's folders");
        }
        fs::create_dir_all(sandbox.runtime()).expect("make the sandbox's runtime folder");
        fs::set_permissions(sandbox.runtime(), fs::Permissions::from_mode(0o700))
            .expect("make the runtime folder its owner's");
        fs::write(sandbox.log(), "").expect("start the stand-ins' log");
        for tool in TOOLS {
            if let Some(found) = real_tool(tool) {
                symlink(found, sandbox.bin().join(tool)).expect("link a tool into the sandbox");
            }
        }
        for name in STUBBED {
            let code = u8::from(FINDS_NOTHING.contains(name));
            sandbox.stub(name, &format!("exit {code}"));
        }
        sandbox
    }

    /// The folder that is the whole `PATH`.
    #[must_use]
    pub fn bin(&self) -> PathBuf {
        self.root.join("bin")
    }

    /// The stand-ins' log: one line a call, `<command> <arguments>`.
    #[must_use]
    pub fn log(&self) -> PathBuf {
        self.root.join("calls")
    }

    /// `XDG_RUNTIME_DIR`, where a user manager's and a session's sockets would be.
    #[must_use]
    pub fn runtime(&self) -> PathBuf {
        self.root.join("run")
    }

    fn tmp(&self) -> PathBuf {
        self.root.join("tmp")
    }

    /// `name`'s stand-in replaced by one that logs its call, then runs `body`, a POSIX shell
    /// script's, with the call's arguments: a test's own answer for a command.
    ///
    /// # Panics
    ///
    /// When it can't be written.
    pub fn stub(&self, name: &str, body: &str) {
        let path = self.bin().join(name);
        let _ = fs::remove_file(&path);
        let log = self.log();
        program(
            &path,
            &format!(
                "#!/bin/sh\nprintf '%s\\n' \"${{0##*/}}${{*:+ $*}}\" >>'{}'\n{body}\n",
                log.display()
            ),
        );
    }

    /// `name` taken out of the sandbox, not found, as on a machine without it.
    ///
    /// # Panics
    ///
    /// When it isn't there.
    pub fn without(&self, name: &str) {
        fs::remove_file(self.bin().join(name)).expect("take a command out of the sandbox");
    }

    /// The calls the stand-ins logged, in order.
    ///
    /// # Panics
    ///
    /// When the log can't be read.
    #[must_use]
    pub fn calls(&self) -> Vec<String> {
        fs::read_to_string(self.log())
            .expect("read the stand-ins' log")
            .lines()
            .map(str::to_owned)
            .collect()
    }

    /// `program`, to run in the sandbox.
    #[must_use]
    pub fn command(&self, program: impl AsRef<OsStr>) -> Command {
        let mut command = Command::new(program);
        self.confine(&mut command);
        command
    }

    /// `command` given the sandbox's environment, and only that, with nothing on stdin.
    pub fn confine<'a>(&self, command: &'a mut Command) -> &'a mut Command {
        let xdg = self.root.join("xdg");
        command
            .env_clear()
            .env("PATH", self.bin())
            .env("HOME", &self.home)
            .env("USER", &self.user)
            .env("LOGNAME", &self.user)
            .env("LANG", "C.UTF-8")
            .env("TMPDIR", self.tmp())
            .env("XDG_RUNTIME_DIR", self.runtime())
            .env("XDG_CONFIG_HOME", self.home.join(".config"))
            .env("XDG_DATA_HOME", self.home.join(".local/share"))
            .env("XDG_STATE_HOME", self.home.join(".local/state"))
            .env("XDG_CACHE_HOME", self.home.join(".cache"))
            .env("XDG_CONFIG_DIRS", &xdg)
            .env("XDG_DATA_DIRS", &xdg)
            .env("DBUS_SESSION_BUS_ADDRESS", NOWHERE)
            .env("DBUS_SYSTEM_BUS_ADDRESS", NOWHERE)
            .stdin(Stdio::null())
    }
}

/// The shell a script's `#!/bin/sh` runs, which the sandbox's `sh` is too: Ubuntu's is dash,
/// a dev shell's `PATH` puts bash first, and the two differ (a `command -v` that finds nothing
/// exits 127 in dash, 1 in bash), so a test's `sh -c` would depend on the shell it ran from.
const SYSTEM_SHELL: &str = "/bin/sh";

/// `tool` as the real `PATH` finds it; `sh` is [`SYSTEM_SHELL`] where the machine has one.
fn real_tool(tool: &str) -> Option<PathBuf> {
    if tool == "sh" && Path::new(SYSTEM_SHELL).is_file() {
        return Some(PathBuf::from(SYSTEM_SHELL));
    }
    let real = env::var_os("PATH").unwrap_or_default();
    env::split_paths(&real).map(|dir| dir.join(tool)).find(|p| p.is_file())
}

/// Writes `text` to `path` as a program anyone may run and its owner alone write, and
/// [`settle`]s it, so a test can start it at once.
///
/// # Panics
///
/// When it can't be written.
pub fn program(path: &Path, text: &str) {
    fs::write(path, text).expect("write a program");
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("make a program run");
    settle(path);
}

/// `path`, a program this process wrote, replaced by a copy `cp` makes, of the same mode, which
/// no process holds open for writing.
///
/// A process forked on another thread holds every file this process has open, from its fork
/// until its own exec closes them (they are all close-on-exec). A program written on one thread
/// while another starts a process is, for that moment, open for writing in the new process, and
/// starting it then fails with "Text file busy" (`ETXTBSY`). `cargo test` runs a crate's tests
/// as threads of one process, and they write scripts and start processes all the time. Writing
/// under another name and renaming it into place doesn't help: the forked process holds the same
/// file. A copy `cp` makes is a file only `cp` ever opened, and `cp` has exited.
///
/// # Panics
///
/// When it can't be copied.
pub fn settle(path: &Path) {
    let name = path.file_name().expect("a program's name").to_string_lossy();
    let copy = path.with_file_name(format!(".{name}.settled"));
    let cp = real_tool("cp").expect("cp on PATH");
    let out = Command::new(cp)
        .env_clear()
        .arg("--")
        .arg(path)
        .arg(&copy)
        .stdin(Stdio::null())
        .output()
        .expect("run cp");
    assert!(
        out.status.success(),
        "cp {}: {}",
        path.display(),
        String::from_utf8_lossy(&out.stderr)
    );
    let mode = fs::metadata(path).expect("read a program's mode").permissions().mode();
    fs::set_permissions(&copy, fs::Permissions::from_mode(mode)).expect("give the copy its mode");
    fs::rename(&copy, path).expect("put the copy in place");
}

#[cfg(test)]
mod tests;
