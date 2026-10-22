//! `--setup --demo --screenshot-tour DIR`: every card of the demo machine
//! with the keyboard only, a PNG per state, then exit. Like the installer's
//! tour (crate::tour): key presses through `app::translate`, waits and
//! screenshots, nothing else. The test below runs the same script headless.

use std::collections::VecDeque;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use iced::keyboard;
use iced::{Task, window};
use kubaos_core::setup::{CardId, PromptKind, State};

use super::{Outcome, Setup};
use crate::app::Message;
use crate::tour::K;

pub enum Step {
    Tap(K),
    Type(&'static str),
    Wait(u64),
    Until(&'static str, fn(&Setup) -> bool),
    Shot(&'static str),
    Exit,
}

const SETTLE: u64 = 700;

fn prompt(s: &Setup, kind: PromptKind) -> bool {
    s.run
        .as_ref()
        .and_then(|r| r.prompt.as_ref())
        .is_some_and(|(k, _, _)| *k == kind)
}

fn state(s: &Setup, c: CardId) -> Option<State> {
    s.card(c).map(|c| c.state)
}

/// The demo TUXEDO: every card, from nothing to all set.
pub fn script() -> Vec<Step> {
    use K::*;
    use Step::*;
    vec![
        Until("status", |s| s.status.is_some()),
        Wait(900),
        Shot("01-overview"),
        Tap(Enter), // into TUXEDO: the MOK password
        Type("kuba2026"),
        Tap(Tab),
        Type("kuba2026"),
        Wait(SETTLE),
        Shot("02-tuxedo-secure-boot"),
        Tap(Enter), // last field: the card's button
        Until("tuxedo busy", |s| {
            s.run.as_ref().is_some_and(|r| r.log.len() >= 4)
        }),
        Tap(Tab), // Skip (the card's button waits while it runs)
        Tap(Tab), // Show the log
        Tap(Enter),
        Wait(400),
        Shot("03-tuxedo-installing"),
        Until("tuxedo done", |s| {
            state(s, CardId::Tuxedo) == Some(State::Reboot)
        }),
        Wait(SETTLE),
        Shot("04-tuxedo-reboot"),
        Tap(Esc),
        Tap(Down),
        Tap(Enter), // into Fingerprint: its button
        Tap(Enter),
        Until("third touch", |s| {
            s.run
                .as_ref()
                .and_then(|r| r.prompt.as_ref())
                .is_some_and(|(k, _, c)| *k == PromptKind::Finger && c.is_some_and(|(k, _)| k >= 3))
        }),
        Wait(150),
        Shot("05-fingerprint-touch"),
        Until("fingerprint done", |s| {
            state(s, CardId::Fingerprint) == Some(State::Done)
        }),
        Tap(Esc),
        Tap(Down),
        Tap(Enter),
        Tap(Enter),
        Until("key touch", |s| prompt(s, PromptKind::Key)),
        Wait(500),
        Shot("06-yubikey-touch"),
        Until("yubikey done", |s| {
            state(s, CardId::Yubikey) == Some(State::Done)
        }),
        Tap(Esc),
        Tap(Down),
        Tap(Enter), // Server (bitwarden.eu, from the machine)
        Tap(Tab),   // Two-step login
        Tap(Tab),   // Email, prefilled
        Tap(Tab),   // Master password
        Type("correct horse battery staple"),
        Tap(Tab),
        Type("000000"),
        Wait(SETTLE),
        Shot("07-bitwarden-form"),
        Tap(Enter),
        Until("wrong code", |s| {
            s.run
                .as_ref()
                .is_some_and(|r| matches!(r.outcome, Some(Outcome::Failed { .. })))
        }),
        Wait(SETTLE),
        Shot("08-bitwarden-wrong-code"),
        Tap(ShiftTab), // Show what I typed
        Tap(ShiftTab), // the code field
        Type("424242"),
        Tap(Enter),
        Until("bitwarden done", |s| {
            state(s, CardId::Bitwarden) == Some(State::Done)
        }),
        Tap(Tab), // from Next to the log
        Tap(Enter),
        Wait(SETTLE),
        Shot("09-bitwarden-done"),
        Tap(Esc),
        Tap(Down),
        Tap(Enter), // Name, from the account's full name
        Tap(Tab),   // Email
        Type("kuba@git.example"),
        Wait(SETTLE),
        Shot("10-git"),
        Tap(Enter), // last field: Save
        Until("git done", |s| state(s, CardId::Git) == Some(State::Done)),
        Tap(Esc),
        Tap(Up),
        Tap(Up),
        Tap(Up),
        Tap(Up),
        Wait(SETTLE),
        Shot("11-all-set"),
        Exit,
    ]
}

/// The bench VM: no TUXEDO, no reader, no key plugged in.
pub fn script_vm() -> Vec<Step> {
    use K::*;
    use Step::*;
    vec![
        Until("status", |s| s.status.is_some()),
        Wait(900),
        Shot("vm-01-overview"),
        Tap(Up),
        Tap(Up),
        Wait(SETTLE),
        Shot("vm-02-not-a-tuxedo"),
        Tap(Down),
        Tap(Down),
        Tap(Down),
        Tap(Enter),
        Wait(SETTLE),
        Shot("vm-03-bitwarden"),
        Exit,
    ]
}

pub struct Tour {
    dir: PathBuf,
    steps: VecDeque<Step>,
    resume_at: Option<Instant>,
    pace: f32,
    pub waiting_for_shot: bool,
}

impl Tour {
    pub fn new(dir: PathBuf, pace: f32, vm: bool) -> Self {
        Self {
            dir,
            steps: if vm { script_vm() } else { script() }.into(),
            resume_at: None,
            pace,
            waiting_for_shot: false,
        }
    }
}

pub fn events(step: &Step) -> Vec<keyboard::Event> {
    match step {
        Step::Tap(k) => crate::tour::events(&crate::tour::Step::Tap(*k)),
        Step::Type(t) => crate::tour::events(&crate::tour::Step::Type(t)),
        _ => vec![],
    }
}

pub fn send(s: &mut Setup, evs: Vec<keyboard::Event>) -> Task<Message> {
    let mut tasks = Vec::new();
    for ev in evs {
        if let Some(k) = crate::app::translate(&ev) {
            tasks.push(s.update_inner(Message::Key(k)));
        }
    }
    Task::batch(tasks)
}

pub fn drive(s: &mut Setup) -> Task<Message> {
    let mut tasks = Vec::new();
    while let Some(tour) = &mut s.tour {
        if tour.waiting_for_shot {
            break;
        }
        let now = Instant::now();
        if let Some(t) = tour.resume_at {
            if now < t {
                break;
            }
            tour.resume_at = None;
        }
        let Some(step) = tour.steps.pop_front() else {
            break;
        };
        match step {
            Step::Wait(ms) => {
                tour.resume_at = Some(now + Duration::from_millis(ms).mul_f32(tour.pace))
            }
            Step::Until(name, f) => {
                let ready = s.window.is_some();
                if !(ready && f(s)) {
                    if let Some(t) = &mut s.tour {
                        t.steps.push_front(Step::Until(name, f));
                    }
                    break;
                }
            }
            Step::Shot(name) => {
                let Some(id) = s.window else { break };
                tour.waiting_for_shot = true;
                tasks.push(
                    window::screenshot(id).map(move |sh| Message::TourShot(name.to_string(), sh)),
                );
                break;
            }
            Step::Exit => {
                tasks.push(iced::exit());
                break;
            }
            keys => {
                let evs = events(&keys);
                tasks.push(send(s, evs));
            }
        }
    }
    Task::batch(tasks)
}

pub fn save(s: &Setup, name: &str, shot: &window::Screenshot) {
    let Some(tour) = &s.tour else { return };
    let _ = std::fs::create_dir_all(&tour.dir);
    let path = tour.dir.join(format!("{name}.png"));
    match image::save_buffer(
        &path,
        &shot.rgba,
        shot.size.width,
        shot.size.height,
        image::ExtendedColorType::Rgba8,
    ) {
        Ok(()) => eprintln!("tour: {}", path.display()),
        Err(e) => eprintln!("tour: {}: {e}", path.display()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::setup::SF;
    use crate::setup::backend::Demo;
    use kubaos_core::setup::demo;
    use std::sync::Arc;

    fn headless(machine: demo::Machine) -> Setup {
        Setup::headless(Arc::new(Demo {
            demo: demo::Demo::new(machine, 200.0),
        }))
    }

    fn play(s: &mut Setup, script: Vec<Step>) {
        let tick = |s: &mut Setup| {
            let _ = s.update_inner(Message::Tick(Instant::now()));
        };
        for step in script {
            match step {
                Step::Until(name, f) => {
                    let deadline = Instant::now() + Duration::from_secs(20);
                    while !f(s) {
                        assert!(
                            Instant::now() < deadline,
                            "stuck waiting for {name} on {:?}, focus {:?}",
                            s.sel,
                            s.focus
                        );
                        tick(s);
                        std::thread::sleep(Duration::from_millis(2));
                    }
                }
                Step::Wait(_) | Step::Shot(_) => tick(s),
                Step::Exit => break,
                keys => {
                    let _ = send(s, events(&keys));
                    tick(s);
                }
            }
        }
    }

    /// Every card of the demo TUXEDO, keys only, headless: no clicks, no
    /// messages but keys and frame ticks.
    #[test]
    fn keyboard_only_setup() {
        let mut s = headless(demo::Machine::Tuxedo);
        play(&mut s, script());
        let st = s.status.as_ref().unwrap();
        assert!(st.settled(), "{st:?}");
        assert_eq!(st.card(CardId::Tuxedo).unwrap().state, State::Reboot);
        // Secrets are gone once their run worked.
        assert!(s.form.password.is_empty() && s.form.code.is_empty() && s.form.mok.is_empty());
        assert_eq!(s.form.email, "kuba@example.com");
        // The account's full name stood in for the git name.
        match &st.card(CardId::Git).unwrap().facts {
            kubaos_core::setup::Facts::Git { name, email, .. } => {
                assert_eq!(name.as_deref(), Some("Kuba Example"));
                assert_eq!(email.as_deref(), Some("kuba@git.example"));
            }
            f => panic!("{f:?}"),
        }
    }

    #[test]
    fn vm_shows_absent_cards_and_tab_reaches_everything() {
        let mut s = headless(demo::Machine::Vm);
        play(&mut s, script_vm());
        assert_eq!(s.sel, CardId::Bitwarden);
        let order = s.focus_order();
        for want in [
            SF::Cards,
            SF::Server,
            SF::Method,
            SF::Email,
            SF::Password,
            SF::Code,
            SF::Primary,
            SF::Skip,
            SF::Finish,
        ] {
            assert!(order.contains(&want), "{want:?} not in {order:?}");
        }
        let start = s.focus;
        for _ in 0..order.len() {
            let _ = send(&mut s, events(&Step::Tap(K::Tab)));
        }
        assert_eq!(s.focus, start);
        assert_eq!(s.card(CardId::Tuxedo).unwrap().state, State::Absent);
    }
}
