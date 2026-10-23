//! The updater's real machine: programs run with their output captured, root ones through `sudo`,
//! and the switch through the job queue.

use std::ffi::OsString;
use std::io;
use std::io::BufRead;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::Child;
use std::process::Command;
use std::process::Stdio;
use std::sync::mpsc;
use std::thread;

use kubaos_core::update::machine::Cmd;
use kubaos_core::update::machine::Lines;
use kubaos_core::update::machine::Machine;
use kubaos_core::update::machine::MachineError;
use kubaos_core::update::machine::NIX_BIN;

/// The switch, handing each chunk of its output to the callback.
type Switch = Box<dyn FnMut(&mut dyn FnMut(&str)) -> Result<(), String>>;

/// The real machine: root commands through `sudo`, output captured.
pub struct System {
    switch: Switch,
    is_root: bool,
    /// `PATH` for user commands, with Nix's tools, when this process's lacks `nix`.
    path: Option<OsString>,
}

impl std::fmt::Debug for System {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("System")
            .field("is_root", &self.is_root)
            .finish_non_exhaustive()
    }
}

impl System {
    /// The machine, switching through `switch`. User commands get Nix's own tools on `PATH` when
    /// it lacks `nix`, as a shell that didn't source Nix's profile has.
    #[must_use]
    pub fn new(switch: impl FnMut(&mut dyn FnMut(&str)) -> Result<(), String> + 'static) -> Self {
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

    /// Starts `cmd` with stdin piped when it has input, else none, stdout and stderr piped, and
    /// feeds the input.
    fn spawn(&self, cmd: &Cmd) -> Result<Child, MachineError> {
        let mut command = cmd.command(self.is_root, self.path.as_deref());
        command.stdin(if cmd.input().is_some() { Stdio::piped() } else { Stdio::null() });
        command.stdout(Stdio::piped()).stderr(Stdio::piped());
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

/// Reads `pipe` to its end on a thread of its own, sending each line on `lines`.
fn forward(pipe: impl Read + Send + 'static, lines: mpsc::Sender<String>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut reader = io::BufReader::new(pipe);
        let mut cut = Lines::default();
        loop {
            let read = match reader.fill_buf() {
                Ok([]) | Err(_) => break,
                Ok(bytes) => {
                    cut.push(bytes, |line| {
                        let _ = lines.send(line.to_owned());
                    });
                    bytes.len()
                }
            };
            reader.consume(read);
        }
        cut.finish(|line| {
            let _ = lines.send(line.to_owned());
        });
    })
}

impl Machine for System {
    fn output(&mut self, cmd: &Cmd) -> Result<String, MachineError> {
        let child = self.spawn(cmd)?;
        let out = child.wait_with_output().map_err(|source| MachineError::Spawn {
            program: cmd.program(),
            source,
        })?;
        if out.status.success() {
            return Ok(String::from_utf8_lossy(&out.stdout).into_owned());
        }
        Err(MachineError::exited(
            cmd,
            out.status.code(),
            &String::from_utf8_lossy(&out.stderr),
        ))
    }

    fn run(&mut self, cmd: &Cmd, on_line: &mut dyn FnMut(&str)) -> Result<(), MachineError> {
        let mut child = self.spawn(cmd)?;
        let (send, lines) = mpsc::channel();
        // Two readers, so a program that fills its stderr pipe while stdout is read never blocks.
        let readers: Vec<_> = [
            child.stdout.take().map(|out| forward(out, send.clone())),
            child.stderr.take().map(|err| forward(err, send.clone())),
        ]
        .into_iter()
        .flatten()
        .collect();
        drop(send);
        for line in lines {
            on_line(&line);
        }
        for reader in readers {
            reader.join().expect("an output reader thread panicked");
        }
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

    fn switch(&mut self, on_line: &mut dyn FnMut(&str)) -> Result<(), MachineError> {
        let mut cut = Lines::default();
        let switched = (self.switch)(&mut |chunk| cut.push(chunk.as_bytes(), &mut *on_line));
        cut.finish(&mut *on_line);
        switched.map_err(MachineError::Switch)
    }

    fn confirm(&mut self, _question: &str) -> bool {
        let mut answer = String::new();
        // No terminal to answer on (stdin closed) is no, not the default yes.
        if !matches!(io::stdin().lock().read_line(&mut answer), Ok(n) if n > 0) {
            return false;
        }
        matches!(answer.trim(), "" | "y" | "Y" | "yes" | "Yes")
    }

    fn needs_password(&mut self) -> bool {
        if self.is_root {
            return false;
        }
        let cached = Command::new("sudo")
            .args(["-n", "true"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        !cached.is_ok_and(|status| status.success())
    }

    fn authenticate(&mut self) -> Result<(), MachineError> {
        let cmd = Cmd::user("sudo").arg("-v");
        let status = Command::new("sudo")
            .arg("-v")
            .status()
            .map_err(|source| MachineError::Spawn {
                program: "sudo".to_owned(),
                source,
            })?;
        if status.success() {
            Ok(())
        } else {
            Err(MachineError::exited(&cmd, status.code(), "the password was not taken"))
        }
    }

    fn free_bytes(&self, path: &Path) -> io::Result<u64> {
        let stat = rustix::fs::statvfs(path)?;
        Ok(stat.f_bavail.saturating_mul(stat.f_frsize))
    }
}
