use std::fs;
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::PermissionsExt;
use std::panic;
use std::panic::AssertUnwindSafe;
use std::path::PathBuf;

use super::Engine;
use super::EngineError;
use super::Keeper;
use crate::apps::action::Side;
use crate::apps::host::UnitState;
use crate::apps::host::fake::FakeHost;
use crate::apps::journal::Journal;
use crate::apps::root::Reply;
use crate::apps::root::Request;
use crate::apps::root::RootError;
use crate::apps::root::RootSide;
use crate::apps::root::Server;
use crate::update::event::ActionState;
use crate::update::event::Sink;
use crate::update::event::UpdateEvent;

/// Four steps, root and user in turn: a package, a file, a service, a setting.
const FIXTURE: &str = r#"
name = "Fixture"
summary = "An app for the tests"

[[action]]
name = "Install the tool"
kind = "apt"
repo = "fixture"
packages = ["fixture-tool"]

[[action]]
name = "Write its config"
kind = "file"
to = "~/.config/fixture/config.toml"
text = "user = '@USER@'\n"

[[action]]
name = "Start the service"
kind = "unit"
unit = "fixture.service"
scope = "system"
enable = true
start = true

[[action]]
name = "Turn it on in the settings"
kind = "setting"
file = "~/.config/other/settings.json"
key = "fixture.enabled"
value = true
"#;

/// A package and a group, as Docker has.
const GROUPED: &str = r#"
name = "Grouped"
summary = "An app with a group"

[[action]]
name = "Install the engine"
kind = "apt"
packages = ["grouped-engine", "shared-tool"]

[[action]]
name = "Let you use it without sudo"
kind = "group"
group = "grouped"
"#;

/// A vendor tool: a download, its own installer, lingering, and a pairing that opens a window.
const VENDOR: &str = r#"
name = "Vendor"
summary = "An app with a vendor's own tool"

[[action]]
name = "Download the tool"
kind = "download"
to = "~/.vendor/versions/@VERSION@/tool"
version = "1.2"
unpack = "file"
amd64 = { url = "https://example.com/tool-1.2", sha256 = "1111111111111111111111111111111111111111111111111111111111111111" }

[[action]]
name = "Install the tool"
kind = "run"
do = ["./install"]
check = ["./installed"]
undo = ["./uninstall"]

[[action]]
name = "Keep its service running"
kind = "linger"

[[action]]
name = "Pair the app"
kind = "run"
do = ["./pair"]
check = ["./paired"]
undo = ["./unpair"]
window = "Open Vendor and pair it"
"#;

/// The same path as the fixture's config, from another app.
const RIVAL: &str = r#"
name = "Rival"
summary = "An app that wants the fixture's config"

[[action]]
name = "Install the rival's tool"
kind = "apt"
packages = ["shared-tool"]

[[action]]
name = "Write the config"
kind = "file"
to = "~/.config/fixture/config.toml"
text = "rival\n"
"#;

struct World {
    _dir: tempfile::TempDir,
    host: FakeHost,
    catalog: PathBuf,
    user_journals: PathBuf,
    root_journals: PathBuf,
    lock: PathBuf,
}

#[derive(Default)]
struct Events(Vec<UpdateEvent>);

impl Sink for Events {
    fn event(&mut self, event: &UpdateEvent) {
        self.0.push(event.clone());
    }
}

impl Events {
    /// The `(step, state)` of each step event of the app.
    fn steps(&self) -> Vec<(usize, ActionState)> {
        self.0
            .iter()
            .filter_map(|e| match e {
                UpdateEvent::Action(a) => Some((a.step, a.state)),
                _ => None,
            })
            .collect()
    }
}

/// A sink that stops the run as it reaches the step named `at`, as a killed process does.
struct StopAt(&'static str);

impl Sink for StopAt {
    fn event(&mut self, event: &UpdateEvent) {
        if let UpdateEvent::Step { text } = event
            && text == self.0
        {
            panic!("the run is killed at {text}");
        }
    }
}

/// Root's side of the fake machine, its requests sent as JSON lines through the real server.
struct FakeRoot {
    host: FakeHost,
    catalog: PathBuf,
    journals: PathBuf,
    owner: u32,
    declined: bool,
    started: bool,
    requests: Vec<String>,
}

impl RootSide for FakeRoot {
    fn start(&mut self) -> Result<(), RootError> {
        if self.declined {
            return Err(RootError::Declined);
        }
        self.started = true;
        Ok(())
    }

    fn call(&mut self, request: &Request) -> Result<Reply, RootError> {
        assert!(self.started, "a request before the root helper started");
        let line = serde_json::to_string(request).unwrap();
        self.requests.push(line.clone());
        Ok(self.serve(&line))
    }
}

impl FakeRoot {
    fn serve(&mut self, line: &str) -> Reply {
        let keeper = Keeper::new(&mut self.host, Side::Root, &self.catalog, &self.journals);
        Server::new(keeper, &self.catalog, self.owner).handle(line)
    }
}

impl World {
    fn new(apps: &[(&str, &str)]) -> Self {
        let dir = tempfile::tempdir().unwrap();
        fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o755)).unwrap();
        let catalog = dir.path().join("catalog");
        for (id, text) in apps {
            fs::create_dir_all(catalog.join(id)).unwrap();
            fs::write(catalog.join(id).join("app.toml"), text).unwrap();
        }
        let host = FakeHost::new(dir.path());
        host.state.borrow_mut().units.insert(
            "system:fixture.service".to_owned(),
            UnitState {
                exists: true,
                ..UnitState::default()
            },
        );
        host.state.borrow_mut().downloads.insert(
            "https://example.com/tool-1.2".to_owned(),
            ("1".repeat(64), b"#!/bin/sh\n".to_vec()),
        );
        Self {
            user_journals: dir.path().join("user-journals"),
            root_journals: dir.path().join("root-journals"),
            lock: dir.path().join("lock"),
            catalog,
            host,
            _dir: dir,
        }
    }

    fn script(&self, app: &str, name: &str, body: &str) {
        let path = self.catalog.join(app).join(name);
        fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    }

    fn root(&self) -> FakeRoot {
        FakeRoot {
            host: self.host.clone(),
            catalog: self.catalog.clone(),
            journals: self.root_journals.clone(),
            owner: fs::metadata(&self.catalog).unwrap().uid(),
            declined: false,
            started: false,
            requests: Vec::new(),
        }
    }

    fn run(
        &self,
        root: &mut FakeRoot,
        sink: &mut dyn Sink,
        windows: bool,
        go: impl FnOnce(&mut Engine<'_>) -> Result<super::Finished, EngineError>,
    ) -> Result<super::Finished, EngineError> {
        let mut user_host = self.host.clone();
        let keeper = Keeper::new(&mut user_host, Side::User, &self.catalog, &self.user_journals);
        let mut engine = Engine::new(keeper, root, &self.root_journals, &self.lock, sink, windows);
        go(&mut engine)
    }

    fn install(&self, id: &str, windows: bool) -> (Result<super::Finished, EngineError>, Events) {
        let mut events = Events::default();
        let mut root = self.root();
        let result = self.run(&mut root, &mut events, windows, |e| e.install(id, None));
        (result, events)
    }

    fn remove(&self, id: &str) -> (Result<super::Finished, EngineError>, Events) {
        let mut events = Events::default();
        let mut root = self.root();
        let result = self.run(&mut root, &mut events, true, |e| e.remove(id));
        (result, events)
    }

    fn journals(&self, id: &str) -> (Option<Journal>, Option<Journal>) {
        (
            Journal::read(&self.user_journals, id).unwrap(),
            Journal::read(&self.root_journals, id).unwrap(),
        )
    }

    fn settings(&self) -> PathBuf {
        self.host_home().join(".config/other/settings.json")
    }

    fn host_home(&self) -> PathBuf {
        use crate::apps::host::Host;
        self.host.home().to_path_buf()
    }
}

#[test]
fn an_install_whose_step_3_of_4_fails_leaves_the_machine_as_it_was() {
    let world = World::new(&[("fixture", FIXTURE)]);
    fs::create_dir_all(world.settings().parent().unwrap()).unwrap();
    fs::write(world.settings(), "{\"theme\": \"dark\"}").unwrap();
    world
        .host
        .state
        .borrow_mut()
        .fail
        .insert("enable:fixture.service".to_owned());
    let before = world.host.snapshot();

    let (result, events) = world.install("fixture", true);

    let Err(EngineError::Step { name, .. }) = result else {
        panic!("the install went through: {result:?}");
    };
    assert_eq!(name, "Start the service");
    assert_eq!(world.host.snapshot(), before);
    assert_eq!(world.journals("fixture"), (None, None));
    assert_eq!(
        events.steps(),
        [
            (1, ActionState::Done),
            (2, ActionState::Done),
            (3, ActionState::Failed),
            (3, ActionState::Undoing),
            (3, ActionState::Undone),
            (2, ActionState::Undoing),
            (2, ActionState::Undone),
            (1, ActionState::Undoing),
            (1, ActionState::Undone),
        ]
    );
}

#[test]
fn removing_an_app_takes_what_it_made_and_keeps_what_it_found() {
    let world = World::new(&[("fixture", FIXTURE)]);
    fs::create_dir_all(world.settings().parent().unwrap()).unwrap();
    fs::write(
        world.settings(),
        "{\"theme\": \"dark\", \"fixture\": {\"enabled\": false}}",
    )
    .unwrap();
    let before = world.host.snapshot();

    let (installed, _) = world.install("fixture", true);
    installed.unwrap();
    let during = world.host.snapshot();
    let (removed, _) = world.remove("fixture");
    removed.unwrap();

    assert_ne!(during, before);
    assert!(during.contains("fixture-tool"), "{during}");
    assert_eq!(world.host.snapshot(), before);
    assert_eq!(world.journals("fixture"), (None, None));
}

#[test]
fn removing_an_app_leaves_a_group_membership_the_user_had() {
    let world = World::new(&[("grouped", GROUPED)]);
    world
        .host
        .state
        .borrow_mut()
        .groups
        .insert("grouped".to_owned(), ["kuba".to_owned()].into());
    let before = world.host.snapshot();

    world.install("grouped", true).0.unwrap();
    world.remove("grouped").0.unwrap();

    assert_eq!(world.host.snapshot(), before);
}

#[test]
fn removing_an_app_takes_the_group_it_made_and_the_membership_it_gave() {
    let world = World::new(&[("grouped", GROUPED)]);
    let before = world.host.snapshot();

    world.install("grouped", true).0.unwrap();
    assert!(world.host.state.borrow().groups["grouped"].contains("kuba"));
    world.remove("grouped").0.unwrap();

    assert_eq!(world.host.snapshot(), before);
}

#[test]
fn a_setting_the_user_changed_after_the_install_stays_as_they_set_it() {
    let world = World::new(&[("fixture", FIXTURE)]);
    world.install("fixture", true).0.unwrap();
    fs::write(world.settings(), "{\"fixture\": {\"enabled\": \"mine\"}}").unwrap();

    world.remove("fixture").0.unwrap();

    assert_eq!(
        fs::read_to_string(world.settings()).unwrap(),
        "{\"fixture\": {\"enabled\": \"mine\"}}"
    );
}

#[test]
fn a_run_stopped_after_step_2_of_4_goes_on_from_step_3() {
    let world = World::new(&[("fixture", FIXTURE)]);
    let mut root = world.root();
    let stopped = panic::catch_unwind(AssertUnwindSafe(|| {
        world.run(&mut root, &mut StopAt("Start the service"), true, |e| {
            e.install("fixture", None)
        })
    }));
    assert!(stopped.is_err());
    let log_before = world.host.state.borrow().log.len();

    let (result, events) = world.install("fixture", true);

    result.unwrap();
    assert_eq!(
        events.steps(),
        [
            (1, ActionState::Done),
            (2, ActionState::Done),
            (3, ActionState::Done),
            (4, ActionState::Done),
        ]
    );
    let log = world.host.state.borrow().log[log_before..].to_vec();
    assert_eq!(log, ["enable fixture.service true"], "steps 1 and 2 ran again");
    assert!(world.journals("fixture").0.unwrap().complete);
}

#[test]
fn a_declined_password_changes_nothing() {
    let world = World::new(&[("fixture", FIXTURE)]);
    let before = world.host.snapshot();
    let mut events = Events::default();
    let mut root = world.root();
    root.declined = true;

    let result = world.run(&mut root, &mut events, true, |e| e.install("fixture", None));

    assert!(matches!(result, Err(EngineError::Declined)), "{result:?}");
    assert!(events.0.contains(&UpdateEvent::Declined));
    assert_eq!(world.host.snapshot(), before);
    assert_eq!(world.journals("fixture"), (None, None));
}

#[test]
fn a_removal_declined_at_the_password_keeps_the_app() {
    let world = World::new(&[("fixture", FIXTURE)]);
    world.install("fixture", true).0.unwrap();
    let before = world.host.snapshot();
    let mut root = world.root();
    root.declined = true;

    let result = world.run(&mut root, &mut Events::default(), true, |e| e.remove("fixture"));

    assert!(matches!(result, Err(EngineError::Declined)), "{result:?}");
    assert_eq!(world.host.snapshot(), before);
    assert!(world.journals("fixture").0.unwrap().complete);
}

#[test]
fn an_app_whose_path_another_app_owns_is_refused_before_anything_runs() {
    let world = World::new(&[("fixture", FIXTURE), ("rival", RIVAL)]);
    world.install("fixture", true).0.unwrap();
    let before = world.host.snapshot();

    let (result, _) = world.install("rival", true);

    let Err(EngineError::Refused { why, .. }) = result else {
        panic!("the rival was installed: {result:?}");
    };
    assert!(why[0].contains("config.toml belongs to fixture"), "{why:?}");
    assert_eq!(world.host.snapshot(), before);
    assert_eq!(world.journals("rival"), (None, None));
}

#[test]
fn a_file_of_the_users_own_is_never_written_over() {
    let world = World::new(&[("rival", RIVAL)]);
    let config = world.host_home().join(".config/fixture/config.toml");
    fs::create_dir_all(config.parent().unwrap()).unwrap();
    fs::write(&config, "mine\n").unwrap();
    let before = world.host.snapshot();

    let (result, _) = world.install("rival", true);

    let Err(EngineError::Refused { why, .. }) = result else {
        panic!("the rival was installed: {result:?}");
    };
    assert!(why[0].contains("is yours"), "{why:?}");
    assert_eq!(world.host.snapshot(), before);
}

#[test]
fn a_package_two_apps_need_goes_with_the_second() {
    let world = World::new(&[("grouped", GROUPED), ("rival", RIVAL)]);
    world.install("grouped", true).0.unwrap();
    world.install("rival", true).0.unwrap();

    world.remove("grouped").0.unwrap();
    assert!(world.host.state.borrow().dpkg.contains_key("shared-tool"));
    assert!(!world.host.state.borrow().dpkg.contains_key("grouped-engine"));
    world.remove("rival").0.unwrap();

    assert!(!world.host.state.borrow().dpkg.contains_key("shared-tool"));
}

#[test]
fn a_step_that_opens_a_window_is_a_note_in_a_run_that_may_not_and_runs_in_one_that_may() {
    let world = World::new(&[("vendor", VENDOR)]);
    world.script(
        "vendor",
        "install",
        "mkdir -p \"$HOME/.vendor\" && touch \"$HOME/.vendor/installed\"",
    );
    world.script("vendor", "installed", "test -e \"$HOME/.vendor/installed\"");
    world.script("vendor", "uninstall", "rm -f \"$HOME/.vendor/installed\"");
    world.script("vendor", "pair", "touch \"$HOME/.vendor/paired\"");
    world.script("vendor", "paired", "test -e \"$HOME/.vendor/paired\"");
    world.script("vendor", "unpair", "rm -f \"$HOME/.vendor/paired\"");

    let (unattended, events) = world.install("vendor", false);
    unattended.unwrap();
    assert!(events.0.contains(&UpdateEvent::Note {
        text: "Open Vendor and pair it".to_owned()
    }));
    assert!(!world.host_home().join(".vendor/paired").exists());
    assert!(world.journals("vendor").0.unwrap().complete);

    let (attended, events) = world.install("vendor", true);
    attended.unwrap();
    assert!(world.host_home().join(".vendor/paired").exists());
    assert_eq!(
        events.steps(),
        [
            (1, ActionState::Done),
            (2, ActionState::Done),
            (3, ActionState::Done),
            (4, ActionState::Done),
        ]
    );
}

#[test]
fn removing_a_vendor_app_undoes_its_runs_and_restores_lingering_only_when_it_was_off() {
    let world = World::new(&[("vendor", VENDOR)]);
    world.script(
        "vendor",
        "install",
        "mkdir -p \"$HOME/.vendor\" && touch \"$HOME/.vendor/installed\"",
    );
    world.script("vendor", "installed", "test -e \"$HOME/.vendor/installed\"");
    world.script("vendor", "uninstall", "rm -rf \"$HOME/.vendor/installed\"");
    world.script("vendor", "pair", "touch \"$HOME/.vendor/paired\"");
    world.script("vendor", "paired", "test -e \"$HOME/.vendor/paired\"");
    world.script("vendor", "unpair", "rm -f \"$HOME/.vendor/paired\"");
    let before = world.host.snapshot();

    world.install("vendor", true).0.unwrap();
    assert!(world.host.state.borrow().linger);
    world.remove("vendor").0.unwrap();
    assert_eq!(world.host.snapshot(), before);

    world.host.state.borrow_mut().linger = true;
    world.install("vendor", true).0.unwrap();
    world.remove("vendor").0.unwrap();
    assert!(world.host.state.borrow().linger);
}

#[test]
fn the_journal_records_every_path_an_install_changed() {
    let world = World::new(&[("fixture", FIXTURE)]);
    world.install("fixture", true).0.unwrap();

    let (user, root) = world.journals("fixture");
    let claims: Vec<String> = user
        .unwrap()
        .entries
        .iter()
        .chain(&root.unwrap().entries)
        .flat_map(|e| e.claims.iter().filter(|c| c.owned).map(|c| c.claim.clone()))
        .collect();
    let home = world.host_home();
    for expected in [
        "package:fixture-tool".to_owned(),
        format!("dir:{}", home.join(".config/fixture").display()),
        format!("path:{}", home.join(".config/fixture/config.toml").display()),
        "unit:system:fixture.service".to_owned(),
        format!("key:{}#fixture.enabled", world.settings().display()),
    ] {
        assert!(claims.contains(&expected), "{expected} not in {claims:?}");
    }
}

#[test]
fn the_root_helper_refuses_anything_but_a_root_step_of_a_catalog_app() {
    let world = World::new(&[("fixture", FIXTURE)]);
    let mut root = world.root();
    let refused = |reply: Reply| matches!(reply, Reply::Refused { .. });

    for line in [
        "not json",
        r#"{"op":"run","argv":["rm","-rf","/"]}"#,
        r#"{"op":"step","app":"../etc","step":1,"windows":false}"#,
        r#"{"op":"step","app":"nothing","step":1,"windows":false}"#,
        r#"{"op":"step","app":"fixture","step":2,"windows":false}"#,
        r#"{"op":"step","app":"fixture","step":9,"windows":false}"#,
        r#"{"op":"step","app":"fixture","setup":"other","step":1,"windows":false}"#,
        r#"{"op":"step","app":"fixture","step":1,"windows":false,"packages":["sl"]}"#,
    ] {
        let reply = root.serve(line);
        assert!(refused(reply.clone()), "{line} was taken: {reply:?}");
    }
    assert_eq!(world.host.state.borrow().log, Vec::<String>::new());

    fs::set_permissions(world.catalog.join("fixture"), fs::Permissions::from_mode(0o777)).unwrap();
    let reply = root.serve(r#"{"op":"step","app":"fixture","step":1,"windows":false}"#);
    assert!(refused(reply.clone()), "a folder others may write was run: {reply:?}");
    fs::set_permissions(world.catalog.join("fixture"), fs::Permissions::from_mode(0o755)).unwrap();

    let reply = root.serve(r#"{"op":"step","app":"fixture","step":1,"windows":false}"#);
    assert!(matches!(reply, Reply::Step { .. }), "{reply:?}");
}

#[test]
fn an_app_in_another_setup_loses_that_setups_own_steps_first() {
    let switching = r#"
name = "Two ways"
summary = "An app with setups"

[[action]]
name = "Install the app"
kind = "apt"
packages = ["two-ways"]

[setup.plain]
name = "Two ways"
summary = "The app alone"

[setup.service]
name = "Two ways as a service"
summary = "With a service"

[[setup.service.action]]
name = "Keep its service running"
kind = "linger"
"#;
    let world = World::new(&[("two-ways", switching)]);
    let mut root = world.root();
    world
        .run(&mut root, &mut Events::default(), true, |e| {
            e.install("two-ways", Some("service"))
        })
        .unwrap();
    assert!(world.host.state.borrow().linger);

    let mut root = world.root();
    world
        .run(&mut root, &mut Events::default(), true, |e| {
            e.install("two-ways", Some("plain"))
        })
        .unwrap();

    assert!(!world.host.state.borrow().linger);
    assert!(world.host.state.borrow().dpkg.contains_key("two-ways"));
    let (user, _) = world.journals("two-ways");
    assert_eq!(user.unwrap().setup.as_deref(), Some("plain"));
}
