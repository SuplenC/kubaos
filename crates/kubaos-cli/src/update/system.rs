//! The updater's real machine: programs run on this terminal, root ones through `sudo`, and the
//! switch through the job queue.

use std::ffi::OsString;
use std::io;
use std::io::BufRead;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::Child;
use std::process::Stdio;

use kubaos_core::update::machine::Cmd;
use kubaos_core::update::machine::Machine;
use kubaos_core::update::machine::MachineError;
use kubaos_core::update::machine::NIX_BIN;

/// The real machine: root commands through `sudo`, output on this terminal.
pub struct System {
    switch: Box<dyn FnMut() -> Result<(), String>>,
    is_root: bool,
    /// `PATH` for user commands, with Nix's tools, when this process's lacks `nix`.
    path: Option<OsString>,
}

impl std::fmt::Debug for System {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("System").field("is_root", &self.is_root).finish_non_exhaustive()
    }
}

impl System {
    /// The machine, switching through `switch`. User commands get Nix's own tools on `PATH` when
    /// it lacks `nix`, as a shell that didn't source Nix's profile has.
    #[must_use]
    pub fn new(switch: impl FnMut() -> Result<(), String> + 'static) -> Self {
        let path = std::env::var_os("PATH").unwrap_or_default();
        let has_nix = std::env::split_paths(&path).any(|dir| dir.join("nix").is_file());
        let path = (!has_nix).then(|| {
            let mut dirs = vec![PathBuf::from(NIX_BIN)];
            dirs.extend(std::env::split_paths(&path));
            std::env::join_paths(dirs).unwrap_or(path)
        });
        Self {
            switch: Box::new(switch),
            is_root: rustix::process::geteuid().is_root(),
            path,
        }
    }

    /// Starts `cmd` with stdin piped when it has input, else `stdin`, and feeds the input.
    fn spawn(&self, cmd: &Cmd, stdin: Stdio, capture: bool) -> Result<Child, MachineError> {
        let mut command = cmd.command(self.is_root, self.path.as_deref());
        command.stdin(if cmd.input().is_some() { Stdio::piped() } else { stdin });
        if capture {
            command.stdout(Stdio::piped()).stderr(Stdio::piped());
        }
        let spawn = |source| MachineError::Spawn {
            program: cmd.program(),
            source,
        };
        let mut child = command.spawn().map_err(spawn)?;
        if let (Some(bytes), Some(mut pipe)) = (cmd.input(), child.stdin.take()) {
            pipe.write_all(bytes).map_err(spawn)?;
        }
        Ok(child)
    }
}

impl Machine for System {
    fn output(&mut self, cmd: &Cmd) -> Result<String, MachineError> {
        let child = self.spawn(cmd, Stdio::null(), true)?;
        let out = child.wait_with_output().map_err(|source| MachineError::Spawn {
            program: cmd.program(),
            source,
        })?;
        if out.status.success() {
            return Ok(String::from_utf8_lossy(&out.stdout).into_owned());
        }
        Err(MachineError::exited(cmd, out.status.code(), &String::from_utf8_lossy(&out.stderr)))
    }

    fn stream(&mut self, cmd: &Cmd) -> Result<(), MachineError> {
        let mut child = self.spawn(cmd, Stdio::inherit(), false)?;
        let status = child.wait().map_err(|source| MachineError::Spawn {
            program: cmd.program(),
            source,
        })?;
        if status.success() {
            Ok(())
        } else {
            Err(MachineError::exited(cmd, status.code(), ""))
        }
    }

    fn switch(&mut self) -> Result<(), MachineError> {
        (self.switch)().map_err(MachineError::Switch)
    }

    fn confirm(&mut self, question: &str) -> bool {
        print!("{question} [Y/n] ");
        let _ = io::stdout().flush();
        let mut answer = String::new();
        // No terminal to answer on (stdin closed) is no, not the default yes.
        if !matches!(io::stdin().lock().read_line(&mut answer), Ok(n) if n > 0) {
            return false;
        }
        matches!(answer.trim(), "" | "y" | "Y" | "yes" | "Yes")
    }

    fn free_bytes(&self, path: &Path) -> io::Result<u64> {
        let stat = rustix::fs::statvfs(path)?;
        Ok(stat.f_bavail.saturating_mul(stat.f_frsize))
    }

    fn say(&mut self, line: &str) {
        println!("{line}");
    }

    fn warn(&mut self, line: &str) {
        eprintln!("kubaos: warning: {line}");
    }
}
