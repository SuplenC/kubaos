//! Where setup's facts and actions come from: this machine (the core's
//! `setup` module, in-process) or the demo machine.

use std::process::Command;
use std::process::Stdio;

use kubaos_core::setup;
use kubaos_core::setup::CardId;
use kubaos_core::setup::CardStatus;
use kubaos_core::setup::Env;
use kubaos_core::setup::Inputs;
use kubaos_core::setup::RealSys;
use kubaos_core::setup::SetupEvent;
use kubaos_core::setup::Status;
use kubaos_core::setup::demo;

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
    /// The startup apps, once setup closes: the session holds them back
    /// while setup is pending (config/hypr/autostart.lua), so the first
    /// login shows setup alone.
    fn start_startup_apps(&self) {}
}

pub struct Real {
    pub env: Env,
}

impl Real {
    pub fn new() -> Self {
        Self { env: Env::current() }
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
        let wrapped = format!("{script}\necho\nread -rp 'Done. Enter closes this window, then Check again in setup. '");
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

    fn start_startup_apps(&self) {
        const TARGET: &str = "xdg-desktop-autostart.target";
        let systemctl = |args: &[&str]| {
            Command::new("systemctl")
                .arg("--user")
                .args(args)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
        };
        // HAZARD: starting the target while it is active starts again every startup app the
        // user has quit since login, as it pulls each one in afresh. Setup opened by hand,
        // after the first login, finds it active.
        if systemctl(&["is-active", "--quiet", TARGET]).is_ok_and(|s| s.success()) {
            return;
        }
        let started = systemctl(&["--no-block", "start", TARGET])
            .map_err(|e| e.to_string())
            .and_then(|s| if s.success() { Ok(()) } else { Err(s.to_string()) });
        if let Err(e) = started {
            log::warn!("can't start the startup apps: systemctl start {TARGET}: {e}");
        }
    }
}

pub struct Demo {
    pub demo: demo::Demo,
}

impl SetupBackend for Demo {
    fn status(&self) -> Status {
        // Detection takes a moment on the real thing too.
        // design-allow(duration): the demo's stand-in for a real detection's wait; not motion.
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
