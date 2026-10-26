//! Where setup's facts and actions come from: this machine (the core's
//! `setup` module, in-process) or the demo machine.

use std::process::{Command, Stdio};

use kubaos_core::setup::{self, CardId, CardStatus, Env, Inputs, RealSys, SetupEvent, Status, demo};

pub trait SetupBackend: Send + Sync {
    /// Detects every card. Slow-ish (bw status takes a second or three):
    /// the app calls it on a worker thread.
    fn status(&self) -> Status;
    /// Detects one card again.
    fn card(&self, id: CardId) -> CardStatus;
    fn run(&self, card: CardId, inputs: &Inputs, emit: &mut dyn FnMut(SetupEvent));
    /// Writes the marker, so setup stays closed at the next login.
    fn finish(&self) -> Result<(), String>;
    fn marker_exists(&self) -> bool;
    fn reboot(&self) -> Result<(), String>;
    /// Opens a terminal running `script` (bash), for the few things that
    /// need one (the YubiKey's own PIN and fingerprints).
    fn terminal(&self, script: &str) -> Result<(), String>;
    fn is_demo(&self) -> bool;
    /// Bitwarden desktop, once setup closes: Hyprland's autostart skips it
    /// while setup is pending (config/hypr/autostart.lua), so the first
    /// login shows setup alone.
    fn start_bitwarden(&self) {}
}

pub struct Real {
    pub env: Env,
}

impl Real {
    pub fn new() -> Self {
        Self {
            env: Env::current(),
        }
    }
}

impl SetupBackend for Real {
    fn status(&self) -> Status {
        setup::status(&mut RealSys::new(), &self.env)
    }

    fn card(&self, id: CardId) -> CardStatus {
        let sys = &mut RealSys::new();
        // The phone card polls every few seconds and needs no internet: no nmcli for it.
        let online = !id.needs_network() || setup::online(sys);
        setup::card_status(sys, &self.env, id, online)
    }

    fn run(&self, card: CardId, inputs: &Inputs, emit: &mut dyn FnMut(SetupEvent)) {
        setup::run(&mut RealSys::new(), &self.env, card, inputs, emit);
    }

    fn finish(&self) -> Result<(), String> {
        setup::mark_done(&mut RealSys::new(), &self.env)
    }

    fn marker_exists(&self) -> bool {
        self.env.marker().exists()
    }

    fn reboot(&self) -> Result<(), String> {
        let s = Command::new("systemctl")
            .arg("reboot")
            .status()
            .map_err(|e| e.to_string())?;
        if s.success() {
            Ok(())
        } else {
            Err(format!("systemctl reboot: {s}"))
        }
    }

    fn terminal(&self, script: &str) -> Result<(), String> {
        let wrapped = format!(
            "{script}\necho\nread -rp 'Done. Enter closes this window, then Check again in setup. '"
        );
        Command::new("ghostty")
            .args(["--title=KubaOS setup", "-e", "bash", "-c", &wrapped])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map(|_| ())
            .map_err(|e| format!("couldn't start ghostty: {e}"))
    }

    fn is_demo(&self) -> bool {
        false
    }

    fn start_bitwarden(&self) {
        const BITWARDEN: &str = "/opt/Bitwarden/bitwarden";
        // amd64 only; already running when setup was opened by hand.
        let installed = std::path::Path::new(BITWARDEN).exists();
        let running = Command::new("pgrep")
            .args(["-f", BITWARDEN])
            .stdout(Stdio::null())
            .status()
            .is_ok_and(|s| s.success());
        if !installed || running {
            return;
        }
        // Its own user unit, which has the session's environment
        // (config/hypr/autostart.lua imports it before setup starts): a
        // child of ours would die with setup's unit. Not hyprctl: the Lua
        // config rejects `dispatch exec` and hyprctl still exits 0.
        let started = Command::new("systemd-run")
            .args([
                "--user",
                "--collect",
                "--property=PartOf=graphical-session.target",
                BITWARDEN,
            ])
            .stdout(Stdio::null())
            .status()
            .map_err(|e| e.to_string())
            .and_then(|s| {
                if s.success() {
                    Ok(())
                } else {
                    Err(s.to_string())
                }
            });
        if let Err(e) = started {
            eprintln!("kobo: can't start Bitwarden: systemd-run: {e}");
        }
    }
}

pub struct Demo {
    pub demo: demo::Demo,
}

impl SetupBackend for Demo {
    fn status(&self) -> Status {
        // Detection takes a moment on the real thing too.
        std::thread::sleep(std::time::Duration::from_millis(300).div_f32(self.demo.speed));
        self.demo.status()
    }

    fn card(&self, id: CardId) -> CardStatus {
        let status = self.demo.status();
        status.card(id).cloned().expect("the demo has every card")
    }

    fn run(&self, card: CardId, inputs: &Inputs, emit: &mut dyn FnMut(SetupEvent)) {
        self.demo.run(card, inputs, emit);
    }

    fn finish(&self) -> Result<(), String> {
        Ok(())
    }

    fn marker_exists(&self) -> bool {
        false
    }

    fn reboot(&self) -> Result<(), String> {
        Err("it's a demo, there's nothing to reboot".into())
    }

    fn terminal(&self, _script: &str) -> Result<(), String> {
        Err("it's a demo, no terminal".into())
    }

    fn is_demo(&self) -> bool {
        true
    }
}
