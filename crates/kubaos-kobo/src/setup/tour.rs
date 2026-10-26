//! `--setup --demo --screenshot-tour DIR`: every card of the demo machine
//! with the keyboard only, a PNG per state, then exit. Like the install
//! screens' tour (crate::tour): key presses through `app::translate`, waits and
//! screenshots, nothing else. The test below runs the same script headless.

use std::collections::VecDeque;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use iced::keyboard;
use iced::{Task, window};
use kubaos_core::setup::demo::Machine;
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
        Tap(Down),
        Wait(SETTLE),
        Shot("11-phone-off"),
        Tap(Enter), // into Your phone: Set up a phone
        Tap(Enter),
        Until("pairing request", |s| {
            s.phone_link().is_some_and(|l| l.request().is_some())
        }),
        Wait(SETTLE),
        Shot("12-phone-pair-request"),
        Tap(Enter), // Accept the pairing
        Until("phone paired", |s| state(s, CardId::Phone) == Some(State::Done)),
        Tap(Enter), // Send a test copy
        Until("test sent", |s| {
            s.run
                .as_ref()
                .is_some_and(|r| matches!(&r.outcome, Some(Outcome::Done(t)) if t.contains("Paste it")))
        }),
        Wait(SETTLE),
        Shot("13-phone-paired"),
        Tap(Esc),
        Tap(Up),
        Tap(Up),
        Tap(Up),
        Tap(Up),
        Tap(Up),
        Wait(SETTLE),
        Shot("14-all-set"),
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

fn finished(s: &Setup) -> bool {
    s.run
        .as_ref()
        .is_some_and(|r| matches!(r.outcome, Some(Outcome::Done(_))))
}

/// After first boot, the Bitwarden card from the Command menu: narrow the SSH keys to a
/// folder, pull again, log out, and log in to a self-hosted server.
pub fn script_bitwarden() -> Vec<Step> {
    use K::*;
    use Step::*;
    vec![
        Until("status", |s| s.status.is_some()),
        Tap(Home),
        Tap(Down),
        Tap(Down),
        Tap(Down),
        Wait(SETTLE),
        Shot("bw-01-installed"),
        Tap(Enter), // into the card: the folders
        Tap(Right), // Work
        Tap(Space),
        Tap(Tab), // Master password
        Type("correct horse battery staple"),
        Wait(SETTLE),
        Shot("bw-02-folders"),
        Tap(Enter), // last field, no main button: Pull again
        Until("pulled", finished),
        Wait(SETTLE),
        Shot("bw-03-pulled-work"),
        Tap(ShiftTab), // from Next to Log out
        Tap(Enter),
        Until("logged out", |s| finished(s) && !s.bw_logged_in()),
        Wait(SETTLE),
        Shot("bw-04-logged-out"),
        Tap(Enter), // into the card: Server
        Tap(Right), // bitwarden.eu to Self-hosted
        Tap(Tab),   // Server address
        Type("https://vault.example.com"),
        Tap(Tab), // Two-step login
        Tap(Tab), // Email, kept
        Tap(Tab), // Master password
        Type("correct horse battery staple"),
        Tap(Tab),
        Type("424242"),
        Wait(SETTLE),
        Shot("bw-05-self-hosted"),
        Tap(Enter),
        Until("logged in", |s| finished(s) && s.bw_logged_in()),
        Wait(SETTLE),
        Shot("bw-06-self-hosted-done"),
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
    pub fn new(dir: PathBuf, pace: f32, machine: Machine) -> Self {
        let steps = match machine {
            Machine::Tuxedo => script(),
            Machine::Vm => script_vm(),
            Machine::Installed => script_bitwarden(),
        };
        Self {
            dir,
            steps: steps.into(),
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
    use kubaos_core::setup::Facts;
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
        let phone = st.card(CardId::Phone).unwrap();
        assert_eq!(phone.state, State::Done);
        assert!(phone.summary.contains("Paste it on iPhone"), "{}", phone.summary);
    }

    fn on_the_phone_card(machine: demo::Machine) -> Setup {
        let mut s = headless(machine);
        s.start = Some(CardId::Phone);
        play(&mut s, vec![Step::Until("status", |s| s.status.is_some())]);
        s
    }

    #[test]
    fn turning_the_phone_link_off_is_done_and_leaves_the_card_to_set_up_again() {
        let mut s = on_the_phone_card(demo::Machine::Installed);
        assert!(s.focus_order().contains(&SF::TurnOff));

        let _ = s.update_inner(Message::Setup(crate::setup::Msg::Click(SF::TurnOff)));
        play(&mut s, vec![Step::Until("off", |s| s.run.as_ref().is_some_and(|r| !r.running()))]);

        let run = s.run.as_ref().unwrap();
        assert!(matches!(&run.outcome, Some(Outcome::Done(t)) if t.starts_with("Phone link off")), "{:?}", run.outcome);
        assert_eq!(s.card(CardId::Phone).unwrap().state, State::Needed);
        assert_eq!(s.primary(), Some(("Set up a phone", crate::setup::Act::Run)));
        assert!(!s.focus_order().contains(&SF::TurnOff));
    }

    #[test]
    fn a_pairing_request_draws_its_key_the_accept_button_and_the_way_out() {
        use iced_test::simulator::Simulator;
        let mut s = on_the_phone_card(demo::Machine::Tuxedo);
        let _ = s.update_inner(Message::Setup(crate::setup::Msg::Click(SF::Primary)));
        play(
            &mut s,
            vec![Step::Until("pairing request", |s| {
                s.phone_link().is_some_and(|l| l.request().is_some()) && !s.running()
            })],
        );

        let mut ui = Simulator::with_size(iced::Settings::default(), iced::Size::new(1280.0, 800.0), crate::setup::view(&s));

        ui.find("The phone shows a key too. It must be the same: 807704F2.").expect("the key");
        ui.find("Also on this network: site24. Not yours? Leave them be.").expect("the stranger");
        ui.click("Accept the pairing").expect("the accept button");
        ui.click("Turn the phone link off").expect("the way out");
        let clicks: Vec<SF> = ui
            .into_messages()
            .filter_map(|m| match m {
                Message::Setup(crate::setup::Msg::Click(f)) => Some(f),
                _ => None,
            })
            .collect();
        assert_eq!(clicks, [SF::Primary, SF::TurnOff]);
    }

    /// The phone card as it waits for a phone, with the link on and nothing seen yet.
    fn waiting_phone(s: &mut Setup) {
        let link = kubaos_core::phone::Status {
            on: true,
            running: true,
            name: Some("kubaos".into()),
            addresses: vec!["192.168.1.23".into()],
            devices: vec![],
        };
        let st = s.status.as_mut().unwrap();
        let card = st.cards.iter_mut().find(|c| c.id == CardId::Phone).unwrap();
        card.state = State::Waiting;
        card.facts = Facts::Phone { link };
    }

    #[test]
    fn the_phone_card_checks_again_on_its_own_while_it_waits() {
        let mut s = on_the_phone_card(demo::Machine::Vm);
        waiting_phone(&mut s);

        // The demo's own phone card is off: the poll brings that back in place of the edit.
        play(&mut s, vec![Step::Until("polled", |s| s.card(CardId::Phone).unwrap().state == State::Needed)]);

        assert!(s.phone_link().is_some_and(|l| !l.on));
    }

    #[test]
    fn a_phone_card_waiting_a_while_explains_adding_the_laptop_by_address() {
        let mut s = on_the_phone_card(demo::Machine::Vm);
        waiting_phone(&mut s);
        let now = Instant::now();
        s.now = now;

        s.phone_waiting_since = Some(now - Duration::from_secs(5));
        assert!(!s.phone_by_address());

        s.phone_waiting_since = Some(now - Duration::from_secs(31));
        assert!(s.phone_by_address());
    }

    #[test]
    fn a_run_that_leaves_its_card_to_do_shows_an_error_and_keeps_the_password() {
        let mut s = headless(demo::Machine::Tuxedo);
        play(
            &mut s,
            vec![Step::Until("detection", |s| s.status.is_some())],
        );
        let mut still = s.card(CardId::Tuxedo).unwrap().clone();
        still.facts = kubaos_core::setup::Facts::Tuxedo {
            vendor: "TUXEDO".into(),
            product: String::new(),
            secure_boot: Some(true),
            drivers: true,
            modules: vec![],
            mok: Some(kubaos_core::setup::MokState::NotEnrolled),
        };
        s.form.mok = "kuba2026".into();
        s.form.mok_again = "kuba2026".into();
        let _ = s.start();

        s.on_event(kubaos_core::setup::SetupEvent::Finished {
            card: CardId::Tuxedo,
            status: still,
        });

        let run = s.run.as_ref().unwrap();
        assert!(
            matches!(run.outcome, Some(crate::setup::Outcome::Failed { .. })),
            "{:?}",
            run.outcome
        );
        assert_eq!(s.form.mok, "kuba2026");
        assert!(s.log_open);
    }

    #[test]
    fn a_card_waiting_on_a_reboot_puts_reboot_to_finish_before_done_for_now() {
        let mut s = headless(demo::Machine::Tuxedo);
        let script = script();
        let upto = script.iter().position(|st| matches!(st, Step::Shot("04-tuxedo-reboot"))).unwrap();
        play(&mut s, script.into_iter().take(upto).collect());

        let order = s.focus_order();
        let reboot = order.iter().position(|f| *f == SF::RebootAll);
        let finish = order.iter().position(|f| *f == SF::Finish);

        assert!(reboot.is_some() && reboot < finish, "{order:?}");
        assert_eq!(s.reboot_nudge().as_deref(), Some("The TUXEDO drivers load after a reboot."));
    }

    #[test]
    fn reboot_to_finish_reboots_through_the_card_path_and_toasts_a_refusal() {
        let mut s = headless(demo::Machine::Tuxedo);
        let script = script();
        let upto = script.iter().position(|st| matches!(st, Step::Shot("04-tuxedo-reboot"))).unwrap();
        play(&mut s, script.into_iter().take(upto).collect());

        let _ = s.update_inner(Message::Setup(crate::setup::Msg::Click(SF::RebootAll)));

        let toast = s.toast.as_ref().map(|(t, _)| t.as_str());
        assert_eq!(toast, Some("Reboot refused: it's a demo, there's nothing to reboot."));
    }

    #[test]
    fn with_nothing_waiting_on_a_reboot_the_footer_has_no_reboot_button() {
        let mut s = headless(demo::Machine::Vm);
        play(&mut s, script_vm());

        assert!(!s.focus_order().contains(&SF::RebootAll));
        assert_eq!(s.reboot_nudge(), None);
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

    #[test]
    fn the_installed_bitwarden_card_narrows_its_keys_logs_out_and_logs_in_to_another_server() {
        let mut s = headless(demo::Machine::Installed);
        let script = script_bitwarden();
        let upto = |name| script.iter().position(|st| matches!(st, Step::Shot(n) if *n == name)).unwrap();
        let (folders, pulled) = (upto("bw-02-folders"), upto("bw-04-logged-out"));
        let mut steps = script.into_iter();

        play(&mut s, steps.by_ref().take(folders).collect());
        // The env/ folder holds project variables, so it isn't offered for SSH keys.
        assert_eq!(s.bw_folders(), ["Personal", "Work"]);
        assert_eq!(s.form.folders, ["Work"]);
        assert!(s.focus_order().contains(&SF::LogOut));

        play(&mut s, steps.by_ref().take(pulled - folders).collect());
        let keys = |s: &Setup| match &s.card(CardId::Bitwarden).unwrap().facts {
            Facts::Bitwarden { overview, .. } => overview.keys.iter().map(|k| k.name.clone()).collect::<Vec<_>>(),
            f => panic!("{f:?}"),
        };
        assert_eq!(keys(&s), ["Work GitHub", "GitHub"]);
        assert_eq!(s.card(CardId::Bitwarden).unwrap().state, State::Needed);
        assert!(!s.focus_order().contains(&SF::LogOut));
        assert!(s.form.password.is_empty());

        play(&mut s, steps.collect());
        let c = s.card(CardId::Bitwarden).unwrap();
        assert_eq!(c.state, State::Done);
        match &c.facts {
            Facts::Bitwarden { server, folders, .. } => {
                assert_eq!(server.as_deref(), Some("https://vault.example.com"));
                assert_eq!(folders, &["Work"]);
            }
            f => panic!("{f:?}"),
        }
        assert!(s.form.password.is_empty() && s.form.code.is_empty());
    }

    #[test]
    fn opening_setup_on_a_card_selects_it_once_detection_is_in() {
        let mut s = headless(demo::Machine::Installed);
        s.start = Some(CardId::Bitwarden);

        play(&mut s, vec![Step::Until("status", |s| s.status.is_some())]);

        assert_eq!(s.sel, CardId::Bitwarden);
    }
}
