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
use crate::apps::host::fake::FakeHost;
use crate::apps::host::fake::FakeRoot;
use crate::apps::host::fake::Unit;
use crate::apps::journal;
use crate::apps::journal::Journal;
use crate::apps::root::Reply;
use crate::sandbox::Sandbox;
use crate::unfree::Unfree;
use crate::update::event::ActionState;
use crate::update::event::Sink;
use crate::update::event::UpdateEvent;

/// Four steps, root and user in turn: a package, a file, a service, a setting.
const FIXTURE: &str = r#"
name = "Fixture"
summary = "An app for the tests"
category = "system"
tags = ["fixture"]
description = "An app the tests install."
description_from = "https://example.com/fixture"
icon = "fixture"

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
category = "system"
tags = ["fixture"]
description = "An app the tests install."
description_from = "https://example.com/fixture"
icon = "fixture"

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
category = "system"
tags = ["fixture"]
description = "An app the tests install."
description_from = "https://example.com/fixture"
icon = "fixture"

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
category = "system"
tags = ["fixture"]
description = "An app the tests install."
description_from = "https://example.com/fixture"
icon = "fixture"

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

impl World {
    fn new(apps: &[(&str, &str)]) -> Self {
        let dir = tempfile::tempdir().unwrap();
        let checkout = dir.path().join("checkout");
        let catalog = checkout.join("apps");
        let system = checkout.join("system");
        fs::create_dir_all(&system).unwrap();
        fs::write(system.join("apt-repo.sh"), "#!/bin/sh\n").unwrap();
        for (id, text) in apps {
            fs::create_dir_all(catalog.join(id)).unwrap();
            fs::write(catalog.join(id).join("app.toml"), text).unwrap();
        }
        // The root helper refuses a checkout others may write, whatever the umask made it.
        let modes = [(checkout, 0o755), (catalog.clone(), 0o755), (system.clone(), 0o755)];
        for (path, mode) in modes.into_iter().chain(
            apps.iter()
                .flat_map(|(id, _)| [(catalog.join(id), 0o755), (catalog.join(id).join("app.toml"), 0o644)]),
        ) {
            fs::set_permissions(path, fs::Permissions::from_mode(mode)).unwrap();
        }
        fs::set_permissions(system.join("apt-repo.sh"), fs::Permissions::from_mode(0o755)).unwrap();
        let host = FakeHost::new(dir.path());
        host.state.borrow_mut().units.insert(
            "system:fixture.service".to_owned(),
            Unit {
                exists: true,
                ..Unit::default()
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
            calls: std::rc::Rc::default(),
            undone: Vec::new(),
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
fn a_panic_during_a_run_leaves_the_engine_holding_no_lock() {
    let world = World::new(&[("fixture", FIXTURE)]);
    let mut root = world.root();
    let mut events = Events::default();
    let mut user_host = world.host.clone();
    let keeper = Keeper::new(&mut user_host, Side::User, &world.catalog, &world.user_journals);
    let mut engine = Engine::new(keeper, &mut root, &world.root_journals, &world.lock, &mut events, true);

    let stopped = panic::catch_unwind(AssertUnwindSafe(|| {
        engine.locked(|_| -> Result<(), EngineError> { panic!("a step stopped the run") })
    }));

    assert!(stopped.is_err());
    assert!(!engine.locked, "the engine thinks it still holds the lock");
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

    let Err(EngineError::Refused { why, yours, .. }) = result else {
        panic!("the rival was installed: {result:?}");
    };
    assert!(why[0].contains("config.toml belongs to fixture"), "{why:?}");
    assert!(!yours, "another app's path isn't the user's to settle");
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

    let Err(EngineError::Refused { why, yours, .. }) = result else {
        panic!("the rival was installed: {result:?}");
    };
    assert!(why[0].contains("is yours"), "{why:?}");
    assert!(yours);
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
        r#"{"op":"adopt","app":"nothing","owned":true}"#,
        r#"{"op":"adopt","app":"../etc","owned":true}"#,
        r#"{"op":"adopt","app":"fixture","owned":true,"step":1}"#,
        r#"{"op":"redo","app":"../etc","step":1}"#,
        r#"{"op":"redo","app":"nothing","step":1}"#,
        r#"{"op":"redo","app":"fixture","setup":"../x","step":1}"#,
        r#"{"op":"redo","app":"fixture","setup":"other","step":1}"#,
        r#"{"op":"redo","app":"fixture","step":2}"#,
        r#"{"op":"redo","app":"fixture","step":1,"entry":{}}"#,
    ] {
        let reply = root.serve(line);
        assert!(refused(reply.clone()), "{line} was taken: {reply:?}");
    }
    assert_eq!(world.host.state.borrow().log, Vec::<String>::new());

    fs::set_permissions(world.catalog.join("fixture"), fs::Permissions::from_mode(0o777)).unwrap();
    let reply = root.serve(r#"{"op":"step","app":"fixture","step":1,"windows":false}"#);
    assert!(refused(reply.clone()), "a folder others may write was run: {reply:?}");
    let reply = root.serve(r#"{"op":"redo","app":"fixture","step":1}"#);
    assert!(
        refused(reply.clone()),
        "a folder others may write was done again: {reply:?}"
    );
    fs::set_permissions(world.catalog.join("fixture"), fs::Permissions::from_mode(0o755)).unwrap();

    let repo = world.catalog.parent().unwrap().join("system/apt-repo.sh");
    fs::set_permissions(&repo, fs::Permissions::from_mode(0o777)).unwrap();
    let reply = root.serve(r#"{"op":"step","app":"fixture","step":1,"windows":false}"#);
    assert!(
        refused(reply.clone()),
        "a repository setup others may write was run: {reply:?}"
    );
    fs::set_permissions(&repo, fs::Permissions::from_mode(0o755)).unwrap();

    let reply = root.serve(r#"{"op":"step","app":"fixture","step":1,"windows":false}"#);
    assert!(matches!(reply, Reply::Step { .. }), "{reply:?}");
}

#[test]
fn root_takes_in_its_steps_of_an_app_the_machine_had_running_nothing() {
    let world = World::new(&[("fixture", FIXTURE)]);
    world
        .host
        .state
        .borrow_mut()
        .dpkg
        .insert("fixture-tool".to_owned(), "1.0".to_owned());
    let before = world.host.snapshot();
    let mut root = world.root();

    let owned = root.serve(r#"{"op":"adopt","app":"fixture","owned":true}"#);

    assert_eq!(owned, Reply::Ready);
    assert_eq!(world.host.snapshot(), before);
    let journal = Journal::read(&world.root_journals, "fixture").unwrap().unwrap();
    // The package is there, and owned; the service isn't started, so it is the next run's.
    assert_eq!(journal.entries.len(), 1);
    assert_eq!(journal.entries[0].step, 1);
    assert!(journal.entries[0].claims.iter().all(|c| c.owned));
    assert!(
        world.host.state.borrow().log.is_empty(),
        "{:?}",
        world.host.state.borrow().log
    );
}

#[test]
fn an_app_in_another_setup_loses_that_setups_own_steps_first() {
    let switching = r#"
name = "Two ways"
summary = "An app with setups"
category = "system"
tags = ["fixture"]
description = "An app the tests install."
description_from = "https://example.com/fixture"
icon = "fixture"

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

#[test]
fn an_installed_app_whose_new_step_fails_keeps_what_it_had() {
    let world = World::new(&[("grouped", GROUPED)]);
    world.install("grouped", true).0.unwrap();
    let installed = world.host.snapshot();
    let grown = format!(
        "{GROUPED}\n[[action]]\nname = \"Start the service\"\nkind = \"unit\"\nunit = \"fixture.service\"\nscope = \"system\"\nenable = true\n"
    );
    fs::write(world.catalog.join("grouped/app.toml"), grown).unwrap();
    world
        .host
        .state
        .borrow_mut()
        .fail
        .insert("enable:fixture.service".to_owned());

    let (result, _) = world.install("grouped", true);

    assert!(matches!(result, Err(EngineError::Step { .. })), "{result:?}");
    assert_eq!(world.host.snapshot(), installed);
    assert!(world.journals("grouped").0.unwrap().complete);
}

/// An app named `Test` whose actions are `actions`.
fn app(actions: &str) -> String {
    format!(
        "name = \"Test\"\nsummary = \"An app for the tests\"\ncategory = \"system\"\ntags = [\"test\"]\n\
         description = \"An app the tests install.\"\ndescription_from = \"https://example.com/test\"\nicon = \"fixture\"\n{actions}"
    )
}

impl World {
    /// The app `id`'s folder given `text`, as a release changes it.
    fn write_app(&self, id: &str, text: &str) {
        fs::write(self.catalog.join(id).join("app.toml"), text).unwrap();
    }

    /// A file of the app `id`'s folder.
    fn folder_file(&self, id: &str, name: &str, text: impl AsRef<[u8]>) {
        let path = self.catalog.join(id).join(name);
        fs::write(&path, text).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
    }

    fn declined(
        &self,
        go: impl FnOnce(&mut Engine<'_>) -> Result<super::Finished, EngineError>,
    ) -> Result<super::Finished, EngineError> {
        let mut root = self.root();
        root.declined = true;
        self.run(&mut root, &mut Events::default(), true, go)
    }

    fn system_path(&self, path: &str) -> PathBuf {
        use crate::apps::host::Host;
        self.host.place(std::path::Path::new(path))
    }

    fn log_since(&self, from: usize) -> Vec<String> {
        self.host.state.borrow().log[from..].to_vec()
    }
}

#[test]
fn a_settings_file_the_setting_cant_read_is_refused_and_left_as_it_is() {
    let world = World::new(&[("fixture", FIXTURE)]);
    fs::create_dir_all(world.settings().parent().unwrap()).unwrap();
    fs::write(world.settings(), "{ \"theme\": \"dark\", // the user's own\n").unwrap();
    let before = world.host.snapshot();

    let (result, _) = world.install("fixture", true);

    let Err(EngineError::Refused { why, .. }) = result else {
        panic!("the install went through: {result:?}");
    };
    assert!(
        why.iter().any(|w| w.contains("isn't settings this action can read")),
        "{why:?}"
    );
    assert_eq!(world.host.snapshot(), before);
}

#[test]
fn a_settings_key_under_a_value_that_isnt_a_table_is_refused() {
    let world = World::new(&[("fixture", FIXTURE)]);
    fs::create_dir_all(world.settings().parent().unwrap()).unwrap();
    fs::write(world.settings(), "{\"fixture\": \"on\"}").unwrap();
    let before = world.host.snapshot();

    let (result, _) = world.install("fixture", true);

    let Err(EngineError::Refused { why, .. }) = result else {
        panic!("the install went through: {result:?}");
    };
    assert!(why.iter().any(|w| w.contains("fixture isn't a table")), "{why:?}");
    assert_eq!(world.host.snapshot(), before);
}

#[test]
fn a_json_setting_keeps_the_files_key_order() {
    let world = World::new(&[("fixture", FIXTURE)]);
    fs::create_dir_all(world.settings().parent().unwrap()).unwrap();
    fs::write(world.settings(), "{\"zeta\": 1, \"alpha\": 2}").unwrap();

    world.install("fixture", true).0.unwrap();

    let text = fs::read_to_string(world.settings()).unwrap();
    let at = |key: &str| text.find(key).unwrap();
    assert!(at("zeta") < at("alpha") && at("alpha") < at("fixture"), "{text}");
}

#[test]
fn a_json_setting_changes_only_its_key_and_keeps_the_files_indentation_and_numbers() {
    let world = World::new(&[("fixture", FIXTURE)]);
    fs::create_dir_all(world.settings().parent().unwrap()).unwrap();
    let mine = "{\n    \"zoom\": 1.50,\n    \"limit\": 1e3,\n    \"big\": 123456789012345678901234567890,\n    \
                \"list\": [1, 2.0],\n    \"fixture\": {\n        \"mode\": \"dark\"\n    }\n}\n";
    fs::write(world.settings(), mine).unwrap();

    world.install("fixture", true).0.unwrap();
    let set = fs::read_to_string(world.settings()).unwrap();
    fs::write(world.settings(), set.replace("dark", "light")).unwrap();
    world.remove("fixture").0.unwrap();

    assert_eq!(
        set,
        mine.replace(
            "\"mode\": \"dark\"\n",
            "\"mode\": \"dark\",\n        \"enabled\": true\n"
        )
    );
    assert_eq!(
        fs::read_to_string(world.settings()).unwrap(),
        mine.replace("dark", "light")
    );
}

#[test]
fn a_json_setting_in_a_new_table_is_laid_out_as_the_file_is() {
    let world = World::new(&[("fixture", FIXTURE)]);
    fs::create_dir_all(world.settings().parent().unwrap()).unwrap();
    fs::write(world.settings(), "{\n\t\"zoom\": 1.50\n}").unwrap();

    world.install("fixture", true).0.unwrap();
    let set = fs::read_to_string(world.settings()).unwrap();
    fs::write(world.settings(), set.replace("1.50", "2.50")).unwrap();
    world.remove("fixture").0.unwrap();

    assert_eq!(
        set,
        "{\n\t\"zoom\": 1.50,\n\t\"fixture\": {\n\t\t\"enabled\": true\n\t}\n}"
    );
    assert_eq!(fs::read_to_string(world.settings()).unwrap(), "{\n\t\"zoom\": 2.50\n}");
}

#[test]
fn a_toml_setting_keeps_the_files_comments_and_types_and_its_undo_takes_only_the_key() {
    let settings = app(r#"
[[action]]
name = "Turn it on"
kind = "setting"
file = "~/.config/tool/config.toml"
key = "ui.enabled"
value = true
"#);
    let world = World::new(&[("tool", &settings)]);
    let config = world.host_home().join(".config/tool/config.toml");
    fs::create_dir_all(config.parent().unwrap()).unwrap();
    let mine = "# The user's own comment.\nupdated = 1979-05-27T07:32:00Z\n\n[ui]\ntheme = \"dark\"\n";
    fs::write(&config, mine).unwrap();

    world.install("tool", true).0.unwrap();
    let set = fs::read_to_string(&config).unwrap();
    fs::write(&config, set.replace("dark", "light")).unwrap();
    world.remove("tool").0.unwrap();

    assert_eq!(
        set,
        "# The user's own comment.\nupdated = 1979-05-27T07:32:00Z\n\n[ui]\ntheme = \"dark\"\nenabled = true\n"
    );
    assert_eq!(fs::read_to_string(&config).unwrap(), mine.replace("dark", "light"));
}

#[test]
fn a_run_checks_a_settled_step_and_does_it_again_when_its_check_fails() {
    let world = World::new(&[("fixture", FIXTURE)]);
    world.install("fixture", true).0.unwrap();
    world.host.state.borrow_mut().dpkg.remove("fixture-tool");
    let from = world.host.state.borrow().log.len();

    let (result, events) = world.install("fixture", true);

    result.unwrap();
    assert!(world.host.state.borrow().dpkg.contains_key("fixture-tool"));
    assert_eq!(world.log_since(from), ["apt install fixture-tool"]);
    assert!(events.steps().iter().all(|(_, state)| *state == ActionState::Done));
}

/// A vendor's `.deb` at `version`, then its service kept off, as Docker's are.
fn deb_app(version: &str) -> String {
    app(&format!(
        r#"
[[action]]
name = "Install the app"
kind = "deb"
package = "vendor-app"
version = "{version}"
amd64 = {{ url = "https://example.com/vendor-app-{version}.deb", sha256 = "{sha}" }}

[[action]]
name = "Keep its service off"
kind = "unit"
unit = "vendor-app.service"
scope = "system"
enable = false
"#,
        sha = version.chars().next().unwrap().to_string().repeat(64),
    ))
}

#[test]
fn a_release_that_bumps_a_deb_pin_upgrades_it_in_place_and_checks_the_steps_after_it() {
    let world = World::new(&[("vendor-app", &deb_app("1.0"))]);
    {
        let mut state = world.host.state.borrow_mut();
        for version in ["1.0", "2.0"] {
            let sha = version[..1].repeat(64);
            let deb = format!("deb vendor-app {version}").into_bytes();
            state
                .downloads
                .insert(format!("https://example.com/vendor-app-{version}.deb"), (sha, deb));
        }
        let unit = Unit {
            exists: true,
            enabled: true,
            ..Unit::default()
        };
        state.units.insert("system:vendor-app.service".to_owned(), unit);
    }
    world.install("vendor-app", true).0.unwrap();
    world.write_app("vendor-app", &deb_app("2.0"));
    // The new version's package enables its service again, as Docker's does.
    world
        .host
        .state
        .borrow_mut()
        .units
        .get_mut("system:vendor-app.service")
        .unwrap()
        .enabled = true;
    let from = world.host.state.borrow().log.len();

    world.install("vendor-app", true).0.unwrap();

    let state = world.host.state.borrow();
    assert_eq!(state.dpkg["vendor-app"], "2.0");
    assert!(!state.units["system:vendor-app.service"].enabled);
    assert_eq!(
        state.log[from..],
        [
            "fetch https://example.com/vendor-app-2.0.deb",
            "deb install vendor-app 2.0",
            "enable vendor-app.service false"
        ]
    );
    drop(state);
    let root = world.journals("vendor-app").1.unwrap();
    assert!(root.entries[0].claims.iter().all(|c| c.owned), "{root:?}");
}

/// A download to a folder named for its version.
fn runtime_app(version: &str) -> String {
    app(&format!(
        r#"
[[action]]
name = "Download the runtime"
kind = "download"
to = "~/.vendor/versions/@VERSION@/tool"
version = "{version}"
unpack = "file"
amd64 = {{ url = "https://example.com/tool-{version}", sha256 = "{sha}" }}
"#,
        sha = version[2..3].repeat(64),
    ))
}

#[test]
fn a_new_download_version_replaces_the_old_one_and_goes_whole_with_the_app() {
    let world = World::new(&[("runtime", &runtime_app("1.1"))]);
    {
        let mut state = world.host.state.borrow_mut();
        state.downloads.insert(
            "https://example.com/tool-1.1".to_owned(),
            ("1".repeat(64), b"one".to_vec()),
        );
        state.downloads.insert(
            "https://example.com/tool-1.3".to_owned(),
            ("3".repeat(64), b"three".to_vec()),
        );
    }
    let before = world.host.snapshot();
    let versions = world.host_home().join(".vendor/versions");
    world.install("runtime", true).0.unwrap();
    world.write_app("runtime", &runtime_app("1.3"));

    world.install("runtime", true).0.unwrap();

    assert_eq!(fs::read_to_string(versions.join("1.3/tool")).unwrap(), "three");
    assert!(!versions.join("1.1").exists());
    world.remove("runtime").0.unwrap();
    assert_eq!(world.host.snapshot(), before);
}

#[test]
fn a_root_undo_that_stopped_partway_is_done_again_by_the_removals_rollback() {
    let world = World::new(&[("grouped", GROUPED)]);
    world.install("grouped", true).0.unwrap();
    world.host.state.borrow_mut().fail.insert("groupdel:grouped".to_owned());

    let (removed, _) = world.remove("grouped");

    assert!(
        matches!(removed, Err(EngineError::RemovalRolledBack { .. })),
        "{removed:?}"
    );
    assert!(world.host.state.borrow().groups["grouped"].contains("kuba"));
    let root = world.journals("grouped").1.unwrap();
    assert_eq!(root.entry(2).unwrap().state, crate::apps::journal::State::Done);
}

#[test]
fn a_removal_that_fails_partway_is_rolled_back_and_leaves_the_app_as_it_was() {
    let world = World::new(&[("fixture", FIXTURE)]);
    fs::create_dir_all(world.settings().parent().unwrap()).unwrap();
    fs::write(world.settings(), "{\"theme\": \"dark\"}").unwrap();
    world.install("fixture", true).0.unwrap();
    let installed = world.host.snapshot();
    let (user, root) = world.journals("fixture");
    // apt can't remove the package, as with dpkg's lock held: the last step undone fails.
    world
        .host
        .state
        .borrow_mut()
        .fail
        .insert("apt-remove:fixture-tool".to_owned());

    let (removed, events) = world.remove("fixture");

    let Err(EngineError::RemovalRolledBack { name, .. }) = &removed else {
        panic!("not rolled back: {removed:?}");
    };
    assert_eq!(name, "Install the tool");
    assert_eq!(world.host.snapshot(), installed, "the app is as it was");
    let (user_after, root_after) = world.journals("fixture");
    let user_after = user_after.unwrap();
    assert!(user_after.complete);
    let mark = journal::rolled_back(&world.user_journals, "fixture");
    assert!(
        mark.as_ref()
            .is_some_and(|m| m.whole && m.why.starts_with("Install the tool: ")),
        "{mark:?}"
    );
    let steps = |j: &Journal| j.entries.iter().map(|e| (e.step, e.state)).collect::<Vec<_>>();
    assert_eq!(steps(&user_after), steps(&user.unwrap()));
    assert_eq!(steps(&root_after.unwrap()), steps(&root.unwrap()));
    // Undone newest first, the failed one last; done again oldest first.
    let order: Vec<(usize, ActionState)> = events.steps().into_iter().collect();
    assert_eq!(
        order,
        [
            (4, ActionState::Undoing),
            (4, ActionState::Undone),
            (3, ActionState::Undoing),
            (3, ActionState::Undone),
            (2, ActionState::Undoing),
            (2, ActionState::Undone),
            (1, ActionState::Undoing),
            (1, ActionState::Failed),
            (1, ActionState::Done),
            (2, ActionState::Done),
            (3, ActionState::Done),
            (4, ActionState::Done),
        ]
    );

    // Once apt can, the removal goes through, and an install no longer tells of the rollback.
    world.host.state.borrow_mut().fail.clear();
    world.install("fixture", true).0.unwrap();
    assert_eq!(journal::rolled_back(&world.user_journals, "fixture"), None);
    world.remove("fixture").0.unwrap();
    assert_eq!(world.journals("fixture"), (None, None));
}

#[test]
fn a_step_a_rollback_cant_do_again_is_a_warning_and_the_next_install_finishes_it() {
    let world = World::new(&[("fixture", FIXTURE)]);
    world.install("fixture", true).0.unwrap();
    let installed = world.host.snapshot();
    {
        let mut state = world.host.state.borrow_mut();
        state.fail.insert("apt-remove:fixture-tool".to_owned());
        state.fail.insert("enable:fixture.service".to_owned());
    }

    let (removed, events) = world.remove("fixture");

    assert!(
        matches!(removed, Err(EngineError::RemovalHalfRolledBack { .. })),
        "{removed:?}"
    );
    let warned = events.0.iter().any(|e| {
        matches!(e, UpdateEvent::Warning { text }
            if text.starts_with("Start the service: rolling the removal back, it couldn't be done again: "))
    });
    assert!(warned, "{:?}", events.0);
    let user = world.journals("fixture").0.unwrap();
    assert!(!user.complete, "{user:?}");
    assert!(journal::rolled_back(&world.user_journals, "fixture").is_some_and(|m| !m.whole));
    // The steps after the one it couldn't do again were still done again.
    assert!(world.settings().is_file());

    world.host.state.borrow_mut().fail.clear();
    world.install("fixture", true).0.unwrap();

    assert_eq!(world.host.snapshot(), installed);
    let user = world.journals("fixture").0.unwrap();
    assert!(user.complete);
    assert_eq!(journal::rolled_back(&world.user_journals, "fixture"), None);
}

#[test]
fn a_root_step_changed_under_the_same_name_asks_for_root_before_anything_runs() {
    let world = World::new(&[("grouped", GROUPED)]);
    world.install("grouped", true).0.unwrap();
    // Every package left on the list is there, so only the action's words say it changed.
    world.write_app("grouped", &GROUPED.replace(", \"shared-tool\"]", "]"));
    let before = world.host.snapshot();

    let result = world.declined(|e| e.install("grouped", None));

    assert!(matches!(result, Err(EngineError::Declined)), "{result:?}");
    assert_eq!(world.host.snapshot(), before);
}

#[test]
fn an_installed_app_whose_root_steps_check_done_needs_no_root() {
    let world = World::new(&[("fixture", FIXTURE)]);
    world.install("fixture", true).0.unwrap();

    let result = world.declined(|e| e.install("fixture", None));

    result.unwrap();
}

#[test]
fn an_install_that_would_remove_a_package_the_app_didnt_install_is_refused() {
    let docker = app(r#"
[[action]]
name = "Install Docker"
kind = "apt"
packages = ["docker-ce"]
"#);
    let world = World::new(&[("docker", &docker)]);
    {
        let mut state = world.host.state.borrow_mut();
        state.dpkg.insert("docker.io".to_owned(), "24.0".to_owned());
        state
            .conflicts
            .insert("docker-ce".to_owned(), ["docker.io".to_owned()].into());
    }
    let before = world.host.snapshot();

    let (result, _) = world.install("docker", true);

    let Err(EngineError::Step { source, .. }) = result else {
        panic!("the install went through: {result:?}");
    };
    assert!(source.to_string().contains("would remove docker.io"), "{source}");
    assert_eq!(world.host.snapshot(), before);
}

#[test]
fn a_removal_that_would_take_a_package_the_user_installed_since_is_refused() {
    let world = World::new(&[("grouped", GROUPED)]);
    world.install("grouped", true).0.unwrap();
    {
        let mut state = world.host.state.borrow_mut();
        state.dpkg.insert("user-app".to_owned(), "1.0".to_owned());
        state
            .depends
            .insert("user-app".to_owned(), ["shared-tool".to_owned()].into());
    }

    let (result, _) = world.remove("grouped");

    let Err(EngineError::RemovalRolledBack { source, .. }) = result else {
        panic!("the removal went through: {result:?}");
    };
    assert!(source.to_string().contains("also remove user-app"), "{source}");
    let state = world.host.state.borrow();
    assert!(state.dpkg.contains_key("user-app") && state.dpkg.contains_key("shared-tool"));
}

#[test]
fn a_setup_switch_refused_by_its_preflight_keeps_the_setup_the_machine_has() {
    let two_ways = app(r#"
[[action]]
name = "Install the app"
kind = "apt"
packages = ["two-ways"]

[setup.plain]
name = "Two ways"
summary = "The app alone"

[[setup.plain.action]]
name = "Write its config"
kind = "file"
to = "~/.config/two-ways/config"
text = "plain\n"

[setup.service]
name = "Two ways as a service"
summary = "With a service"

[[setup.service.action]]
name = "Keep its service running"
kind = "linger"
"#);
    let world = World::new(&[("two-ways", &two_ways)]);
    let mut root = world.root();
    world
        .run(&mut root, &mut Events::default(), true, |e| {
            e.install("two-ways", Some("service"))
        })
        .unwrap();
    let config = world.host_home().join(".config/two-ways/config");
    fs::create_dir_all(config.parent().unwrap()).unwrap();
    fs::write(&config, "mine\n").unwrap();

    let mut root = world.root();
    let result = world.run(&mut root, &mut Events::default(), true, |e| {
        e.install("two-ways", Some("plain"))
    });

    assert!(matches!(result, Err(EngineError::Refused { .. })), "{result:?}");
    assert!(world.host.state.borrow().linger);
    assert_eq!(world.journals("two-ways").0.unwrap().setup.as_deref(), Some("service"));
}

#[test]
fn a_tarball_unpacks_owned_by_whoever_unpacks_it_and_writable_by_no_one_else() {
    // tar gives what it unpacks the archive's modes less the umask: under a umask that lets the
    // group write, as Ubuntu's 002 does, only the action itself keeps the files the owner's. The
    // test runs again in a child under that umask, so no other test's thread sees it changed.
    const UNDER_UMASK: &str = "KUBAOS_TEST_UNDER_UMASK_002";
    if std::env::var_os(UNDER_UMASK).is_none() {
        let name = module_path!().split_once("::").map_or("", |(_, rest)| rest);
        let dir = tempfile::tempdir().unwrap();
        let sandbox = crate::sandbox::Sandbox::new(&dir.path().join("sandbox"), &dir.path().join("home"), "tester");
        let ran = sandbox
            .command("sh")
            .args(["-c", "umask 002 && exec \"$0\" \"$@\""])
            .arg(std::env::current_exe().unwrap())
            .arg(format!(
                "{name}::a_tarball_unpacks_owned_by_whoever_unpacks_it_and_writable_by_no_one_else"
            ))
            .args(["--exact", "--nocapture", "--test-threads=1"])
            .env(UNDER_UMASK, "1")
            .output()
            .unwrap();
        let out = String::from_utf8_lossy(&ran.stdout);
        assert!(
            ran.status.success() && out.contains("1 passed"),
            "{out}{}",
            String::from_utf8_lossy(&ran.stderr)
        );
        return;
    }
    let tarball = app(r#"
[[action]]
name = "Download the runtime"
kind = "download"
to = "~/.vendor/runtime"
version = "1.0"
unpack = "tar-gz"
strip = 1
amd64 = { url = "https://example.com/runtime.tar.gz", sha256 = "4444444444444444444444444444444444444444444444444444444444444444" }
"#);
    let world = World::new(&[("runtime", &tarball)]);
    let src = tempfile::tempdir().unwrap();
    fs::create_dir_all(src.path().join("pkg/bin")).unwrap();
    fs::write(src.path().join("pkg/bin/tool"), "#!/bin/sh\n").unwrap();
    for (path, mode) in [("pkg", 0o777), ("pkg/bin", 0o777), ("pkg/bin/tool", 0o777)] {
        fs::set_permissions(src.path().join(path), fs::Permissions::from_mode(mode)).unwrap();
    }
    let archive = src.path().join("runtime.tar.gz");
    let made = crate::sandbox::output(
        "tar",
        [
            std::ffi::OsStr::new("-czf"),
            archive.as_os_str(),
            std::ffi::OsStr::new("-C"),
            src.path().as_os_str(),
            std::ffi::OsStr::new("pkg"),
        ],
    );
    assert!(made.status.success());
    world.host.state.borrow_mut().downloads.insert(
        "https://example.com/runtime.tar.gz".to_owned(),
        ("4".repeat(64), fs::read(&archive).unwrap()),
    );
    let before = world.host.snapshot();

    world.install("runtime", true).0.unwrap();

    let runtime = world.host_home().join(".vendor/runtime");
    for path in [runtime.clone(), runtime.join("bin"), runtime.join("bin/tool")] {
        let meta = fs::metadata(&path).unwrap();
        assert_eq!(meta.mode() & 0o022, 0, "{} is writable by others", path.display());
        assert_eq!(meta.uid(), fs::metadata(world.host_home()).unwrap().uid());
    }
    assert_eq!(fs::read_to_string(runtime.join("bin/tool")).unwrap(), "#!/bin/sh\n");
    world.remove("runtime").0.unwrap();
    assert_eq!(world.host.snapshot(), before);
}

/// A system unit enabled, from an app named by `name`.
fn unit_app(name: &str) -> String {
    app(r#"
[[action]]
name = "Enable the shared service"
kind = "unit"
unit = "shared.service"
scope = "system"
enable = true
"#)
    .replace("name = \"Test\"", &format!("name = \"{name}\""))
}

#[test]
fn a_unit_two_apps_enable_goes_back_to_how_it_was_whichever_goes_first() {
    for first in ["a", "b"] {
        let world = World::new(&[("a", &unit_app("A")), ("b", &unit_app("B"))]);
        world.host.state.borrow_mut().units.insert(
            "system:shared.service".to_owned(),
            Unit {
                exists: true,
                ..Unit::default()
            },
        );
        world.install("a", true).0.unwrap();
        world.install("b", true).0.unwrap();
        let second = if first == "a" { "b" } else { "a" };

        world.remove(first).0.unwrap();
        assert!(
            world.host.state.borrow().units["system:shared.service"].enabled,
            "{first} first"
        );
        world.remove(second).0.unwrap();

        assert!(
            !world.host.state.borrow().units["system:shared.service"].enabled,
            "{first} first"
        );
    }
}

const OUTPUT: &str = r#"
[[action]]
name = "Let the helper write the policy"
kind = "output"
path = "/etc/opt/fixture/policies/managed/theme.json"
"#;

#[test]
fn an_output_the_app_made_room_for_goes_with_it_and_one_found_stays() {
    let world = World::new(&[("helper", &app(OUTPUT))]);
    let policy = world.system_path("/etc/opt/fixture/policies/managed/theme.json");
    let before = world.host.snapshot();

    world.install("helper", true).0.unwrap();
    assert!(policy.parent().unwrap().is_dir());
    fs::write(&policy, "{}").unwrap();
    world.install("helper", true).0.unwrap();
    world.remove("helper").0.unwrap();
    assert_eq!(world.host.snapshot(), before);

    fs::create_dir_all(policy.parent().unwrap()).unwrap();
    fs::write(&policy, "{\"left\": \"by an older install\"}").unwrap();
    let (found, events) = world.install("helper", true);
    found.unwrap();
    assert_eq!(events.steps(), [(1, ActionState::Found)]);
    world.remove("helper").0.unwrap();
    assert!(policy.exists());
}

#[test]
fn a_stopped_long_running_service_is_started_and_stopped_again_on_removal() {
    let service = app(r#"
[[action]]
name = "Start the service"
kind = "unit"
unit = "daemon.service"
scope = "system"
start = true
"#);
    let world = World::new(&[("daemon", &service)]);
    // It ran earlier in this boot and was stopped: systemd keeps a clean exit for it.
    let stopped = Unit {
        exists: true,
        exited: true,
        ..Unit::default()
    };
    world
        .host
        .state
        .borrow_mut()
        .units
        .insert("system:daemon.service".to_owned(), stopped);

    let (installed, events) = world.install("daemon", true);
    installed.unwrap();
    assert_eq!(events.steps(), [(1, ActionState::Done)]);
    assert!(world.host.state.borrow().units["system:daemon.service"].active);

    world.remove("daemon").0.unwrap();

    assert!(!world.host.state.borrow().units["system:daemon.service"].active);
}

#[test]
fn a_started_oneshot_that_ran_is_done_and_needs_no_root_again() {
    let theme = app(r#"
[[action]]
name = "Theme it once"
kind = "unit"
unit = "theme.service"
scope = "system"
start = true
"#);
    let world = World::new(&[("theme", &theme)]);
    {
        let mut state = world.host.state.borrow_mut();
        let unit = Unit {
            exists: true,
            oneshot: true,
            ..Unit::default()
        };
        state.units.insert("system:theme.service".to_owned(), unit);
    }
    world.install("theme", true).0.unwrap();
    assert!(world.host.state.borrow().units["system:theme.service"].exited);

    let result = world.declined(|e| e.install("theme", None));

    result.unwrap();
}

#[test]
fn a_removal_runs_the_recorded_undo_script_after_the_app_left_the_catalog() {
    let tool = app(r#"
[[action]]
name = "Install its runtime"
kind = "apt"
packages = ["tool-runtime"]

[[action]]
name = "Install the tool"
kind = "run"
do = ["./install"]
check = ["./installed"]
undo = ["./uninstall"]
"#);
    let world = World::new(&[("tool", &tool)]);
    world.script("tool", "install", "touch \"$HOME/.tool-installed\"");
    world.script("tool", "installed", "test -e \"$HOME/.tool-installed\"");
    world.script("tool", "uninstall", "rm -f \"$HOME/.tool-installed\"");
    world.install("tool", true).0.unwrap();
    assert!(world.host_home().join(".tool-installed").exists());
    world.script("tool", "uninstall", "exit 1");
    fs::remove_dir_all(world.catalog.join("tool")).unwrap();

    world.remove("tool").0.unwrap();

    assert!(!world.host_home().join(".tool-installed").exists());
}

#[test]
fn a_ready_check_gets_its_words_filled_in() {
    let ready = app(r#"
[[ready]]
check = ["test", "-d", "@HOME@"]
says = "The home isn't there"

[[action]]
name = "Keep its service running"
kind = "linger"
"#);
    let world = World::new(&[("ready", &ready)]);

    world.install("ready", true).0.unwrap();
}

#[test]
fn a_step_left_doing_is_finished_by_the_next_run_and_the_rest_is_kept() {
    let world = World::new(&[("fixture", FIXTURE)]);
    world.install("fixture", true).0.unwrap();
    let mut journal = world.journals("fixture").0.unwrap();
    journal.complete = false;
    let config = journal.entries.iter_mut().find(|e| e.step == 2).unwrap();
    config.state = crate::apps::journal::State::Doing;
    journal.save(&world.user_journals).unwrap();
    let config = world.host_home().join(".config/fixture/config.toml");
    fs::remove_file(&config).unwrap();
    let from = world.host.state.borrow().log.len();

    let (result, events) = world.install("fixture", true);

    result.unwrap();
    assert_eq!(fs::read_to_string(&config).unwrap(), "user = '@USER@'\n");
    assert!(world.log_since(from).is_empty(), "{:?}", world.log_since(from));
    assert!(events.steps().iter().all(|(_, state)| *state == ActionState::Done));
    let journal = world.journals("fixture").0.unwrap();
    assert!(journal.complete);
    assert_eq!(journal.entry(2).unwrap().state, crate::apps::journal::State::Done);
    assert!(
        journal
            .entry(2)
            .unwrap()
            .claims
            .iter()
            .any(|c| c.owned && c.path().is_some())
    );
}

#[test]
fn every_kind_of_the_user_side_and_a_deb_go_whole_with_the_app() {
    let kinds = app(r#"
[[action]]
name = "Install the app"
kind = "deb"
package = "sketch"
version = "1.0"
amd64 = { url = "https://example.com/sketch.deb", sha256 = "5555555555555555555555555555555555555555555555555555555555555555" }

[[action]]
name = "Add its tools"
kind = "nix"
packages = ["ripgrep", "fd"]

[[action]]
name = "Link its config"
kind = "symlink"
to = "~/.config/sketch/current"
target = "/opt/sketch/config"

[[action]]
name = "Start its user service"
kind = "unit"
unit = "sketch.service"
scope = "user"
enable = true
start = true

[[action]]
name = "Open it in a window of its own"
kind = "web_app"
id = "sketch"
title = "Sketch"
url = "https://sketch.example/"
icon = "sketch.svg"
"#);
    let world = World::new(&[("sketch", &kinds)]);
    world.folder_file("sketch", "sketch.svg", "<svg/>\n");
    {
        let mut state = world.host.state.borrow_mut();
        state.downloads.insert(
            "https://example.com/sketch.deb".to_owned(),
            ("5".repeat(64), b"deb sketch 1.0".to_vec()),
        );
        state.nix.insert("fd".to_owned());
        let unit = Unit {
            exists: true,
            ..Unit::default()
        };
        state.units.insert("user:sketch.service".to_owned(), unit);
    }
    let before = world.host.snapshot();

    world.install("sketch", true).0.unwrap();

    let home = world.host_home();
    {
        let state = world.host.state.borrow();
        assert_eq!(state.dpkg["sketch"], "1.0");
        assert!(state.nix.contains("ripgrep"));
        let unit = state.units["user:sketch.service"];
        assert!(unit.enabled && unit.active);
    }
    assert_eq!(
        fs::read_link(home.join(".config/sketch/current")).unwrap(),
        PathBuf::from("/opt/sketch/config")
    );
    let entry = fs::read_to_string(home.join(".local/share/applications/kubaos-web-sketch.desktop")).unwrap();
    let profile = home.join(".local/share/kubaos/web-apps/sketch");
    assert!(
        entry.contains(&format!(
            "Exec=google-chrome --user-data-dir={} --profile-directory=kubaos-web-sketch --no-first-run \
             --no-default-browser-check --app=https://sketch.example/\n",
            profile.display()
        )),
        "{entry}"
    );
    assert!(
        entry.contains("StartupWMClass=chrome-sketch.example__-kubaos-web-sketch\n"),
        "{entry}"
    );
    let icon = home.join(".local/share/icons/hicolor/scalable/apps/kubaos-web-sketch.svg");
    assert!(entry.contains(&format!("Icon={}\n", icon.display())), "{entry}");
    assert_eq!(fs::read_to_string(icon).unwrap(), "<svg/>\n");

    world.remove("sketch").0.unwrap();

    assert_eq!(world.host.snapshot(), before);
    assert!(world.host.state.borrow().nix.contains("fd"));
}

#[test]
fn an_unfree_nix_step_is_refused_while_the_users_nixpkgs_refuses_unfree_and_lists_nothing() {
    let chat = app(r#"
[[action]]
name = "Install Chat"
kind = "nix"
packages = ["discord"]
unfree = true
"#);
    let world = World::new(&[("chat", &chat)]);
    world.host.state.borrow_mut().unfree = Unfree::Refused;
    let before = world.host.snapshot();

    let err = world.install("chat", true).0.unwrap_err();

    assert!(err.to_string().contains("Install Chat"), "{err}");
    assert!(world.host.state.borrow().nix.is_empty());
    assert_eq!(world.host.snapshot(), before);

    world.host.state.borrow_mut().unfree = Unfree::Allowed;
    world.install("chat", true).0.unwrap();
    assert!(world.host.state.borrow().nix.contains("discord"));
}

#[test]
fn a_nix_step_for_another_architecture_is_refused_and_lists_nothing() {
    let zoom = app(r#"
[[action]]
name = "Install Zoom"
kind = "nix"
arch = "arm64"
packages = ["zoom-us"]
"#);
    let world = World::new(&[("zoom", &zoom)]);

    world.install("zoom", true).0.unwrap_err();

    assert!(world.host.state.borrow().nix.is_empty());
}

#[test]
fn a_web_app_naming_a_theme_icon_points_its_entry_at_it_and_copies_nothing() {
    let web = app(r#"
[[action]]
name = "Open it in a window of its own"
kind = "web_app"
id = "chat"
title = "Chat"
url = "https://chat.example/"
icon = "whatsapp"
"#);
    let world = World::new(&[("chat", &web)]);
    let before = world.host.snapshot();

    world.install("chat", true).0.unwrap();

    let home = world.host_home();
    let entry = fs::read_to_string(home.join(".local/share/applications/kubaos-web-chat.desktop")).unwrap();
    assert!(entry.contains("\nIcon=whatsapp\n"), "{entry}");
    assert!(!home.join(".local/share/icons").exists());

    world.remove("chat").0.unwrap();

    assert_eq!(world.host.snapshot(), before);
}

#[test]
fn a_web_app_moving_from_its_svg_to_a_theme_icon_takes_the_copied_icon_away() {
    let svg = app(r#"
[[action]]
name = "Open it in a window of its own"
kind = "web_app"
id = "chat"
title = "Chat"
url = "https://chat.example/"
icon = "chat.svg"
"#);
    let world = World::new(&[("chat", &svg)]);
    world.folder_file("chat", "chat.svg", "<svg/>\n");
    world.install("chat", true).0.unwrap();
    let icon = world
        .host_home()
        .join(".local/share/icons/hicolor/scalable/apps/kubaos-web-chat.svg");
    assert!(icon.is_file());

    world.write_app("chat", &svg.replace("icon = \"chat.svg\"", "icon = \"whatsapp\""));
    world.install("chat", true).0.unwrap();

    assert!(!icon.exists());
    let entry = fs::read_to_string(
        world
            .host_home()
            .join(".local/share/applications/kubaos-web-chat.desktop"),
    )
    .unwrap();
    assert!(entry.contains("\nIcon=whatsapp\n"), "{entry}");
}

#[test]
fn a_web_apps_browser_folder_is_the_users_and_outlasts_a_removal_and_a_reinstall() {
    let web = app(r#"
[[action]]
name = "Open it in a window of its own"
kind = "web_app"
id = "sketch"
title = "Sketch"
url = "https://sketch.example/"
icon = "sketch.svg"
"#);
    let world = World::new(&[("sketch", &web)]);
    world.folder_file("sketch", "sketch.svg", "<svg/>\n");
    world.install("sketch", true).0.unwrap();
    let profile = world.host_home().join(".local/share/kubaos/web-apps/sketch");
    fs::create_dir_all(profile.join("kubaos-web-sketch")).unwrap();
    fs::write(profile.join("kubaos-web-sketch/Cookies"), "signed in").unwrap();

    world.remove("sketch").0.unwrap();

    assert_eq!(
        fs::read_to_string(profile.join("kubaos-web-sketch/Cookies")).unwrap(),
        "signed in"
    );
    assert!(
        !world
            .host_home()
            .join(".local/share/applications/kubaos-web-sketch.desktop")
            .exists()
    );
    world.install("sketch", true).0.unwrap();
    assert_eq!(
        fs::read_to_string(profile.join("kubaos-web-sketch/Cookies")).unwrap(),
        "signed in"
    );
    world.remove("sketch").0.unwrap();
    assert!(profile.join("kubaos-web-sketch/Cookies").is_file());
}

#[test]
fn a_web_apps_url_reaches_the_browser_whole_through_its_desktop_entry() {
    let penpot = app(r#"
[[action]]
name = "Put Penpot in a window of its own"
kind = "web_app"
id = "penpot"
title = "Penpot"
url = "https://design.example/view?team=a&file=b#page%20one"
icon = "penpot.svg"
"#);
    let world = World::new(&[("penpot", &penpot)]);
    world.folder_file("penpot", "penpot.svg", "<svg/>\n");

    world.install("penpot", true).0.unwrap();

    let entry = fs::read_to_string(
        world
            .host_home()
            .join(".local/share/applications/kubaos-web-penpot.desktop"),
    )
    .unwrap();
    assert!(
        entry.contains("--no-default-browser-check \"--app=https://design.example/view?team=a&file=b#page%%20one\"\n"),
        "{entry}"
    );
    assert!(
        entry.contains("\nStartupWMClass=chrome-design.example__view-kubaos-web-penpot\n"),
        "{entry}"
    );
}

#[test]
fn a_web_apps_window_is_named_as_chrome_names_an_app_window_on_wayland() {
    use crate::apps::action::perform::web_app_window;

    for (url, window) in [
        ("https://penpot.app/", "chrome-penpot.app__-kubaos-web-penpot"),
        ("https://penpot.app", "chrome-penpot.app__-kubaos-web-penpot"),
        ("http://localhost:9001/", "chrome-localhost__-kubaos-web-penpot"),
        (
            "http://127.0.0.1:9001/#/dashboard",
            "chrome-127.0.0.1__-kubaos-web-penpot",
        ),
        ("http://[::1]:9001/", "chrome-[__1]__-kubaos-web-penpot"),
        (
            "https://Design.Example/a/b?x=1",
            "chrome-design.example__a_b-kubaos-web-penpot",
        ),
    ] {
        assert_eq!(web_app_window(url, "kubaos-web-penpot"), window, "{url}");
    }
}

/// A download to a path without its version, `unpack` as given, pinned to `version`.
fn fixed_download(version: &str, unpack: &str) -> String {
    let sha = version.replace('.', "").repeat(32);
    let zstd = if unpack == "tar-zst" {
        "[[action]]\nname = \"Install zstd\"\nkind = \"apt\"\npackages = [\"zstd\"]\n"
    } else {
        ""
    };
    app(&format!(
        r#"
{zstd}
[[action]]
name = "Download the tool"
kind = "download"
to = "~/.vendor/tool"
version = "{version}"
unpack = "{unpack}"
amd64 = {{ url = "https://example.com/tool-{version}", sha256 = "{sha}" }}
"#
    ))
}

/// A gzipped tarball holding `bin/tool` with `text`.
fn tarball(text: &str) -> Vec<u8> {
    packed(text, "tar-gz")
}

/// An archive of `unpack` (`tar-gz`, `tar-zst` or `tar-xz`) holding `bin/tool` with `text`, made by
/// the tar of a sandbox.
fn packed(text: &str, unpack: &str) -> Vec<u8> {
    let src = tempfile::tempdir().unwrap();
    fs::create_dir_all(src.path().join("bin")).unwrap();
    fs::write(src.path().join("bin/tool"), text).unwrap();
    let sandbox = Sandbox::new(&src.path().join("sandbox"), src.path(), "kuba");
    let archive = src.path().join("tool.tar");
    let compress = match unpack {
        "tar-zst" => "--zstd",
        "tar-xz" => "--xz",
        _ => "--gzip",
    };
    let made = sandbox
        .command("tar")
        .args(["-c", compress, "-f"])
        .arg(&archive)
        .arg("-C")
        .arg(src.path())
        .arg("bin")
        .status()
        .unwrap();
    assert!(made.success());
    fs::read(archive).unwrap()
}

#[test]
fn a_release_that_bumps_a_download_at_the_same_path_downloads_it_again() {
    for unpack in ["file", "tar-gz", "tar-zst", "tar-xz"] {
        let world = World::new(&[("tool", &fixed_download("1.1", unpack))]);
        for version in ["1.1", "1.3"] {
            let bytes = match unpack {
                "file" => format!("tool {version}").into_bytes(),
                _ => packed(&format!("tool {version}"), unpack),
            };
            let sha = version.replace('.', "").repeat(32);
            world
                .host
                .state
                .borrow_mut()
                .downloads
                .insert(format!("https://example.com/tool-{version}"), (sha, bytes));
        }
        let tool = match unpack {
            "file" => world.host_home().join(".vendor/tool"),
            _ => world.host_home().join(".vendor/tool/bin/tool"),
        };
        let before = world.host.snapshot();
        world.install("tool", true).0.unwrap();
        world.write_app("tool", &fixed_download("1.3", unpack));

        world.install("tool", true).0.unwrap();

        assert_eq!(fs::read_to_string(&tool).unwrap(), "tool 1.3", "{unpack}");
        let journal = world.journals("tool").0.unwrap();
        assert!(
            journal.entries.iter().any(
                |e| matches!(&e.action.kind, crate::apps::action::Kind::Download { version, .. } if version == "1.3")
            ),
            "{unpack}"
        );
        world.remove("tool").0.unwrap();
        assert_eq!(world.host.snapshot(), before, "{unpack}");
    }
}

#[test]
fn a_self_updating_download_its_updater_rewrites_in_place_is_never_downloaded_over() {
    let self_updating =
        |version: &str| fixed_download(version, "file").replace("unpack =", "self_updates = true\nunpack =");
    let world = World::new(&[("tool", &self_updating("1.1"))]);
    for version in ["1.1", "1.3"] {
        let sha = version.replace('.', "").repeat(32);
        world.host.state.borrow_mut().downloads.insert(
            format!("https://example.com/tool-{version}"),
            (sha, format!("tool {version}").into_bytes()),
        );
    }
    world.install("tool", true).0.unwrap();
    let tool = world.host_home().join(".vendor/tool");
    fs::write(&tool, "tool 2.0, from its own updater").unwrap();
    world.write_app("tool", &self_updating("1.3"));

    world.install("tool", true).0.unwrap();

    assert_eq!(fs::read_to_string(&tool).unwrap(), "tool 2.0, from its own updater");
    let fetched = world.host.state.borrow().log.iter().any(|l| l.ends_with("tool-1.3"));
    assert!(!fetched);
    world.remove("tool").0.unwrap();
    assert!(!tool.exists());
}

#[test]
fn a_record_that_words_the_same_download_differently_is_rewritten_while_the_app_is_open() {
    let open_check = "[[ready]]\ncheck = [\"test\", \"!\", \"-e\", \"@HOME@/tool-open\"]\nsays = \"Tool is running\"\n";
    let tool = format!(
        "{}{open_check}",
        fixed_download("1.1", "file").replace("unpack =", "self_updates = true\nunpack =")
    );
    let world = World::new(&[("tool", &tool)]);
    world.host.state.borrow_mut().downloads.insert(
        "https://example.com/tool-1.1".to_owned(),
        ("11".repeat(32), b"tool 1.1".to_vec()),
    );
    world.install("tool", true).0.unwrap();
    // A journal from before the record said the updater rewrites the download in place.
    let mut journal = world.journals("tool").0.unwrap();
    for entry in &mut journal.entries {
        if let crate::apps::action::Kind::Download { in_place, .. } = &mut entry.action.kind {
            *in_place = false;
        }
    }
    journal.save(&world.user_journals).unwrap();
    fs::write(world.host_home().join("tool-open"), "").unwrap();
    let mut root = world.root();
    let mut events = Events::default();

    world
        .run(&mut root, &mut events, true, |e| e.install("tool", None))
        .unwrap();

    assert!(!root.started, "root was asked for");
    let journal = world.journals("tool").0.unwrap();
    assert!(journal.entries.iter().any(|e| matches!(
        e.action.kind,
        crate::apps::action::Kind::Download { in_place: true, .. }
    )));
}

#[test]
fn a_self_updating_download_of_a_folder_its_version_is_part_of_is_downloaded_again_when_its_pin_changes() {
    // `tool-@VERSION@` names the version in a folder of its own, not in place: a new build of
    // the same version replaces the old one, as for any download.
    let pinned = |url: &str, sha: &str| {
        fixed_download("1.1", "file")
            .replace(
                "to = \"~/.vendor/tool\"",
                "to = \"~/.vendor/tool-@VERSION@\"\nself_updates = true",
            )
            .replace("https://example.com/tool-1.1", url)
            .replace(&"11".repeat(32), sha)
    };
    let (first, second) = ("a".repeat(64), "b".repeat(64));
    let world = World::new(&[("tool", &pinned("https://example.com/tool-1.1", &first))]);
    for (url, sha, text) in [
        ("https://example.com/tool-1.1", &first, "tool 1.1"),
        ("https://example.com/tool-1.1-rebuilt", &second, "tool 1.1, rebuilt"),
    ] {
        world
            .host
            .state
            .borrow_mut()
            .downloads
            .insert(url.to_owned(), (sha.clone(), text.as_bytes().to_vec()));
    }
    world.install("tool", true).0.unwrap();
    world.write_app("tool", &pinned("https://example.com/tool-1.1-rebuilt", &second));

    world.install("tool", true).0.unwrap();

    let tool = world.host_home().join(".vendor/tool-1.1");
    assert_eq!(fs::read_to_string(tool).unwrap(), "tool 1.1, rebuilt");
}

#[test]
fn a_download_a_run_stopped_during_is_downloaded_again() {
    let world = World::new(&[("tool", &fixed_download("1.1", "file"))]);
    for version in ["1.1", "1.3"] {
        let sha = version.replace('.', "").repeat(32);
        world.host.state.borrow_mut().downloads.insert(
            format!("https://example.com/tool-{version}"),
            (sha, format!("tool {version}").into_bytes()),
        );
    }
    world.install("tool", true).0.unwrap();
    world.write_app("tool", &fixed_download("1.3", "file"));
    world
        .host
        .state
        .borrow_mut()
        .fail
        .insert("fetch:https://example.com/tool-1.3".to_owned());
    world.install("tool", true).0.unwrap_err();
    world.host.state.borrow_mut().fail.clear();
    let journal = world.journals("tool").0.unwrap();
    assert_eq!(journal.entry(1).unwrap().state, crate::apps::journal::State::Doing);
    assert_eq!(
        fs::read_to_string(world.host_home().join(".vendor/tool")).unwrap(),
        "tool 1.1"
    );

    world.install("tool", true).0.unwrap();

    assert_eq!(
        fs::read_to_string(world.host_home().join(".vendor/tool")).unwrap(),
        "tool 1.3"
    );
}

/// An app of one apt package, `packages`, named by `name`.
fn apt_app(name: &str, packages: &str) -> String {
    app(&format!(
        r#"
[[action]]
name = "Install it"
kind = "apt"
packages = [{packages}]
"#
    ))
    .replace("name = \"Test\"", &format!("name = \"{name}\""))
}

/// A machine where `tool` needs `libtool`, which isn't there, and `libc`, which is.
fn world_with_dependencies(apps: &[(&str, &str)]) -> World {
    let world = World::new(apps);
    {
        let mut state = world.host.state.borrow_mut();
        state.dpkg.insert("libc".to_owned(), "2.39".to_owned());
        state
            .depends
            .insert("tool".to_owned(), ["libtool".to_owned(), "libc".to_owned()].into());
    }
    world
}

#[test]
fn an_apps_svg_icon_is_installed_into_the_icon_theme_journaled_and_goes_with_it() {
    let tool = app("[[action]]\nname = \"Keep it running\"\nkind = \"linger\"\n")
        .replace("icon = \"fixture\"", "icon = \"tool.svg\"");
    let world = World::new(&[("tool", &tool)]);
    let svg = "<!-- The tool's logo, MIT. -->\n<svg/>\n";
    world.folder_file("tool", "tool.svg", svg);
    let before = world.host.snapshot();

    world.install("tool", true).0.unwrap();

    let icon = world
        .host_home()
        .join(".local/share/icons/hicolor/scalable/apps/tool.svg");
    assert_eq!(fs::read_to_string(&icon).unwrap(), svg);
    let journal = world.journals("tool").0.unwrap();
    let entry = journal.entries.iter().find(|e| e.step == 2).unwrap();
    assert!(
        matches!(&entry.action.kind, crate::apps::action::Kind::File { text: Some(t), .. } if t == svg),
        "{entry:?}"
    );
    world.remove("tool").0.unwrap();
    assert_eq!(world.host.snapshot(), before);
}

/// A machine whose `tool` has the picture `bytes` as its icon `name`.
fn world_with_icon(name: &str, bytes: &[u8]) -> World {
    let tool = app("[[action]]\nname = \"Keep it running\"\nkind = \"linger\"\n")
        .replace("icon = \"fixture\"", &format!("icon = \"{name}\""));
    let world = World::new(&[("tool", &tool)]);
    world.folder_file("tool", name, bytes);
    world
}

#[test]
fn an_apps_png_icon_is_installed_into_the_themes_folder_of_its_size_and_journaled_as_its_bytes() {
    let png = crate::apps::folder::picture::png(128, 128);
    let world = world_with_icon("tool.png", &png);

    world.install("tool", true).0.unwrap();

    let icon = world
        .host_home()
        .join(".local/share/icons/hicolor/128x128/apps/tool.png");
    assert_eq!(fs::read(&icon).unwrap(), png);
    assert_eq!(fs::metadata(&icon).unwrap().permissions().mode() & 0o777, 0o644);
    let journal = world.journals("tool").0.unwrap();
    let entry = journal.entries.iter().find(|e| e.step == 2).unwrap();
    assert!(
        matches!(
            &entry.action.kind,
            crate::apps::action::Kind::File {
                binary: true,
                text: Some(_),
                ..
            }
        ),
        "{entry:?}"
    );
    // Installed again, the icon is there as recorded, so nothing is done: the file isn't written
    // again and its entry records the same bytes.
    let written = fs::metadata(&icon).unwrap().modified().unwrap();
    world.install("tool", true).0.unwrap();
    assert_eq!(fs::read(&icon).unwrap(), png);
    assert_eq!(fs::metadata(&icon).unwrap().modified().unwrap(), written);
    let again = world.journals("tool").0.unwrap();
    assert_eq!(again.entries[1].action.kind, journal.entries[1].action.kind);
}

#[test]
fn a_binary_icon_whose_journal_holds_no_base64_is_written_again_from_the_catalog_and_goes_with_its_app() {
    let world = world_with_icon("tool.png", &crate::apps::folder::picture::png(128, 128));
    let before = world.host.snapshot();
    world.install("tool", true).0.unwrap();
    let icon = world
        .host_home()
        .join(".local/share/icons/hicolor/128x128/apps/tool.png");
    let file = Journal::file(&world.user_journals, "tool");
    let journal = fs::read_to_string(&file).unwrap();
    let Some(crate::apps::action::Kind::File { text: Some(base64), .. }) = world
        .journals("tool")
        .0
        .unwrap()
        .entries
        .iter()
        .find(|e| e.step == 2)
        .map(|e| e.action.kind.clone())
    else {
        panic!("no icon step");
    };
    assert!(journal.contains(&base64));
    fs::write(&file, journal.replace(&base64, "not base64!")).unwrap();
    fs::remove_file(&icon).unwrap();

    world.install("tool", true).0.unwrap();
    assert_eq!(fs::read(&icon).unwrap(), crate::apps::folder::picture::png(128, 128));
    assert!(fs::read_to_string(&file).unwrap().contains(&base64));

    world.remove("tool").0.unwrap();
    assert_eq!(world.host.snapshot(), before);
}

#[test]
fn an_apps_png_icon_goes_with_it_and_leaves_the_machine_as_before() {
    let world = world_with_icon("tool.png", &crate::apps::folder::picture::png(128, 128));
    let before = world.host.snapshot();

    world.install("tool", true).0.unwrap();
    world.remove("tool").0.unwrap();

    assert_eq!(world.host.snapshot(), before);
}

#[test]
fn an_apps_jpeg_icon_puts_nothing_in_the_icon_theme_which_reads_no_jpeg() {
    let world = world_with_icon("tool.jpg", &crate::apps::folder::picture::jpeg(256, 256));
    let before = world.host.snapshot();

    world.install("tool", true).0.unwrap();

    let journal = world.journals("tool").0.unwrap();
    assert_eq!(journal.entries.len(), 1, "{journal:?}");
    assert!(!world.host_home().join(".local/share/icons").exists());
    world.remove("tool").0.unwrap();
    assert_eq!(world.host.snapshot(), before);
}

#[test]
fn an_apps_jpeg_icon_goes_with_it_and_leaves_the_machine_as_before() {
    let world = world_with_icon("tool.jpeg", &crate::apps::folder::picture::jpeg(256, 256));
    let before = world.host.snapshot();

    world.install("tool", true).0.unwrap();
    world.remove("tool").0.unwrap();

    assert_eq!(world.host.snapshot(), before);
    assert_eq!(world.journals("tool"), (None, None));
}

/// A machine with Ubuntu's snap `firefox` (`1:1snap1`), which sorts above the vendor's 160.0, and
/// `libnss3`, which the vendor's firefox needs, at 3.110 from the user's own pin, where apt would
/// take it down to 3.98.
fn world_with_a_pinned_dependency() -> World {
    let firefox = app(r#"
[[action]]
name = "Install Firefox"
kind = "apt"
packages = ["firefox"]
versions = "*~build*"
"#);
    let world = World::new(&[("firefox", &firefox)]);
    {
        let mut state = world.host.state.borrow_mut();
        state
            .dpkg
            .insert("firefox".to_owned(), "1:1snap1-0ubuntu9.1".to_owned());
        state.candidates.insert("firefox".to_owned(), "160.0~build1".to_owned());
        state.dpkg.insert("libnss3".to_owned(), "2:3.110-1".to_owned());
        state.candidates.insert("libnss3".to_owned(), "2:3.98-1".to_owned());
        state
            .depends
            .insert("firefox".to_owned(), ["libnss3".to_owned()].into());
    }
    world
}

#[test]
fn an_install_that_would_move_down_a_package_other_than_its_own_is_refused_naming_it() {
    let world = world_with_a_pinned_dependency();
    let before = world.host.snapshot();

    let error = world.install("firefox", true).0.unwrap_err();

    let shown = crate::update::event::chain(&error);
    assert!(shown.contains("would move libnss3 down"), "{shown}");
    assert!(!shown.contains("firefox down"), "{shown}");
    assert_eq!(world.host.snapshot(), before);
}

#[test]
fn an_install_moves_its_own_package_down_to_replace_another_build_of_it() {
    let world = world_with_a_pinned_dependency();
    world.host.state.borrow_mut().depends.clear();

    world.install("firefox", true).0.unwrap();

    assert_eq!(
        world.host.state.borrow().dpkg.get("firefox").map(String::as_str),
        Some("160.0~build1")
    );
}

#[test]
fn removing_an_app_removes_what_its_packages_pulled_in_and_keeps_what_was_there() {
    let world = world_with_dependencies(&[("tool", &apt_app("Tool", "\"tool\""))]);
    let before = world.host.snapshot();

    world.install("tool", true).0.unwrap();
    assert!(world.host.state.borrow().dpkg.contains_key("libtool"));
    let (_, root) = world.journals("tool");
    assert!(
        root.unwrap().entries[0]
            .claims
            .iter()
            .any(|c| c.claim == "dep:libtool" && c.owned)
    );
    world.remove("tool").0.unwrap();

    assert_eq!(world.host.snapshot(), before);
}

#[test]
fn a_pulled_in_package_something_else_needs_stays_when_the_app_goes() {
    let world = world_with_dependencies(&[("tool", &apt_app("Tool", "\"tool\""))]);
    world.install("tool", true).0.unwrap();
    {
        let mut state = world.host.state.borrow_mut();
        state.dpkg.insert("user-app".to_owned(), "1.0".to_owned());
        state
            .depends
            .insert("user-app".to_owned(), ["libtool".to_owned()].into());
    }

    world.remove("tool").0.unwrap();

    let state = world.host.state.borrow();
    assert!(!state.dpkg.contains_key("tool"));
    assert!(state.dpkg.contains_key("libtool") && state.dpkg.contains_key("user-app"));
}

#[test]
fn a_pulled_in_package_another_app_installs_passes_to_it() {
    let world = world_with_dependencies(&[
        ("tool", &apt_app("Tool", "\"tool\"")),
        ("lib", &apt_app("Lib", "\"libtool\"")),
    ]);
    world.install("tool", true).0.unwrap();
    world.install("lib", true).0.unwrap();

    world.remove("tool").0.unwrap();
    assert!(world.host.state.borrow().dpkg.contains_key("libtool"));
    world.remove("lib").0.unwrap();

    assert!(!world.host.state.borrow().dpkg.contains_key("libtool"));
}

#[test]
fn a_release_that_turns_an_unpacked_download_into_a_file_at_the_same_path_replaces_the_folder() {
    let world = World::new(&[("tool", &fixed_download("1.1", "tar-gz"))]);
    {
        let mut state = world.host.state.borrow_mut();
        state.downloads.insert(
            "https://example.com/tool-1.1".to_owned(),
            ("11".repeat(32), tarball("tool 1.1")),
        );
        state.downloads.insert(
            "https://example.com/tool-1.3".to_owned(),
            ("13".repeat(32), b"tool 1.3".to_vec()),
        );
    }
    let before = world.host.snapshot();
    world.install("tool", true).0.unwrap();
    world.write_app("tool", &fixed_download("1.3", "file"));

    world.install("tool", true).0.unwrap();

    assert_eq!(
        fs::read_to_string(world.host_home().join(".vendor/tool")).unwrap(),
        "tool 1.3"
    );
    world.remove("tool").0.unwrap();
    assert_eq!(world.host.snapshot(), before);
}

#[test]
fn a_folder_a_stopped_run_left_aside_goes_with_the_next_download() {
    let world = World::new(&[("tool", &fixed_download("1.1", "tar-gz"))]);
    for version in ["1.1", "1.3"] {
        world.host.state.borrow_mut().downloads.insert(
            format!("https://example.com/tool-{version}"),
            (version.replace('.', "").repeat(32), tarball(&format!("tool {version}"))),
        );
    }
    let before = world.host.snapshot();
    world.install("tool", true).0.unwrap();
    // The run stopped after moving the old folder aside and before moving the new one in.
    let vendor = world.host_home().join(".vendor");
    fs::rename(vendor.join("tool"), vendor.join(".tool.old")).unwrap();
    world.write_app("tool", &fixed_download("1.3", "tar-gz"));

    world.install("tool", true).0.unwrap();

    assert!(!vendor.join(".tool.old").exists());
    assert_eq!(fs::read_to_string(vendor.join("tool/bin/tool")).unwrap(), "tool 1.3");
    world.remove("tool").0.unwrap();
    assert_eq!(world.host.snapshot(), before);
}

#[test]
fn a_pulled_in_package_the_user_installed_by_name_since_stays_when_the_app_goes() {
    let world = world_with_dependencies(&[("tool", &apt_app("Tool", "\"tool\""))]);
    world.install("tool", true).0.unwrap();
    world.host.state.borrow_mut().manual.insert("libtool".to_owned());

    world.remove("tool").0.unwrap();

    let state = world.host.state.borrow();
    assert!(!state.dpkg.contains_key("tool"));
    assert!(state.dpkg.contains_key("libtool"));
}

#[test]
fn a_json_setting_there_twice_goes_whole_with_the_app() {
    let world = World::new(&[("fixture", FIXTURE)]);
    fs::create_dir_all(world.settings().parent().unwrap()).unwrap();
    fs::write(world.settings(), "{\"zoom\": 1}").unwrap();
    world.install("fixture", true).0.unwrap();
    fs::write(
        world.settings(),
        "{\"zoom\": 1, \"fixture\": {\"enabled\": true, \"enabled\": true}}",
    )
    .unwrap();

    world.remove("fixture").0.unwrap();

    assert_eq!(fs::read_to_string(world.settings()).unwrap(), "{\"zoom\": 1}");
}

#[test]
fn a_json_setting_in_a_new_table_keeps_the_files_crlf_line_endings() {
    let world = World::new(&[("fixture", FIXTURE)]);
    fs::create_dir_all(world.settings().parent().unwrap()).unwrap();
    fs::write(world.settings(), "{\r\n\t\"zoom\": 1.50\r\n}").unwrap();

    world.install("fixture", true).0.unwrap();

    assert_eq!(
        fs::read_to_string(world.settings()).unwrap(),
        "{\r\n\t\"zoom\": 1.50,\r\n\t\"fixture\": {\r\n\t\t\"enabled\": true\r\n\t}\r\n}"
    );
}

/// A runtime its vendor's updater moves on, in a folder named for its version.
fn self_updating_runtime(version: &str) -> String {
    app(&format!(
        r#"
[[action]]
name = "Download the runtime"
kind = "download"
to = "~/.vendor/versions/@VERSION@"
version = "{version}"
unpack = "tar-gz"
self_updates = true
amd64 = {{ url = "https://example.com/tool-{version}", sha256 = "{sha}" }}
"#,
        sha = version.replace('.', "").repeat(32),
    ))
}

#[test]
fn a_self_updating_download_its_vendor_moved_on_is_not_downloaded_again() {
    let world = World::new(&[("runtime", &self_updating_runtime("1.1"))]);
    world.host.state.borrow_mut().downloads.insert(
        "https://example.com/tool-1.1".to_owned(),
        ("11".repeat(32), tarball("tool 1.1")),
    );
    world.install("runtime", true).0.unwrap();
    let versions = world.host_home().join(".vendor/versions");
    fs::rename(versions.join("1.1"), versions.join("1.10")).unwrap();
    let from = world.host.state.borrow().log.len();

    let (result, events) = world.install("runtime", true);

    result.unwrap();
    assert!(
        !world.log_since(from).iter().any(|l| l.starts_with("fetch")),
        "{:?}",
        world.log_since(from)
    );
    assert_eq!(events.steps(), [(1, ActionState::Done)]);
    assert!(!versions.join("1.1").exists());
}

#[test]
fn a_removal_shows_each_step_it_undoes_by_name() {
    let world = World::new(&[("fixture", FIXTURE)]);
    world.install("fixture", true).0.unwrap();

    let (result, events) = world.remove("fixture");

    result.unwrap();
    let shown: Vec<&str> = events
        .0
        .iter()
        .filter_map(|e| match e {
            UpdateEvent::Step { text } => Some(text.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(
        shown,
        [
            "Turn it on in the settings",
            "Start the service",
            "Write its config",
            "Install the tool"
        ]
    );
    assert!(events.0.contains(&UpdateEvent::Progress { done: 4, total: 4 }));
}

mod ai;
mod architecture;
mod browsers;
mod cloud;
mod communication;
mod devtools;
mod dkms;
mod editors;
mod games;
mod graphics;
mod iac;
mod keys;
mod kubernetes;
mod media;
mod office;
mod requires;
mod security;
mod system;
mod undo_only;
mod versions;

/// The claim `claim` of the app `id`'s journals: whether it owns it, when a step claims it.
fn owns(world: &World, id: &str, claim: &str) -> Option<bool> {
    let (user, root) = world.journals(id);
    user.into_iter()
        .chain(root)
        .flat_map(|j| j.entries)
        .flat_map(|e| e.claims)
        .find(|c| c.claim == claim)
        .map(|c| c.owned)
}

#[test]
fn a_shared_package_stays_the_removed_apps_while_its_undo_fails_and_the_other_app_never_takes_it() {
    let world = World::new(&[("grouped", GROUPED), ("rival", RIVAL)]);
    world.install("grouped", true).0.unwrap();
    world.install("rival", true).0.unwrap();
    world
        .host
        .state
        .borrow_mut()
        .fail
        .insert("apt-remove:grouped-engine".to_owned());

    let (removed, _) = world.remove("grouped");

    assert!(
        matches!(removed, Err(EngineError::RemovalRolledBack { .. })),
        "{removed:?}"
    );
    assert_eq!(owns(&world, "grouped", "package:shared-tool"), Some(true));
    assert_eq!(
        owns(&world, "rival", "package:shared-tool"),
        Some(false),
        "never handed over"
    );
    // The rival's removal leaves the package the other app still needs.
    world.host.state.borrow_mut().fail.clear();
    world.remove("rival").0.unwrap();
    assert!(world.host.state.borrow().dpkg.contains_key("shared-tool"));
    world.remove("grouped").0.unwrap();
    assert!(!world.host.state.borrow().dpkg.contains_key("shared-tool"));
}

#[test]
fn a_rollback_takes_back_a_claim_its_removal_handed_over_and_the_step_owns_it_again() {
    let late = app(r#"
[[action]]
name = "Install the engine"
kind = "apt"
packages = ["late-engine"]

[[action]]
name = "Install the shared tool"
kind = "apt"
packages = ["shared-tool"]
"#);
    let world = World::new(&[("late", &late), ("rival", RIVAL)]);
    world.install("late", true).0.unwrap();
    world.install("rival", true).0.unwrap();
    let (_, root) = world.journals("late");
    // The shared tool's step goes first, handing the package to the rival; then the engine's
    // removal fails.
    world
        .host
        .state
        .borrow_mut()
        .fail
        .insert("apt-remove:late-engine".to_owned());

    let (removed, _) = world.remove("late");

    assert!(
        matches!(removed, Err(EngineError::RemovalRolledBack { .. })),
        "{removed:?}"
    );
    assert_eq!(owns(&world, "late", "package:shared-tool"), Some(true));
    assert_eq!(owns(&world, "rival", "package:shared-tool"), Some(false));
    let states = |j: Journal| j.entries.iter().map(|e| (e.step, e.state)).collect::<Vec<_>>();
    assert_eq!(states(world.journals("late").1.unwrap()), states(root.unwrap()));
}

#[test]
fn a_rollback_leaves_a_step_that_opens_a_window_to_a_note_and_says_a_run_ran_again() {
    let paired = app(r#"
[[action]]
name = "Install the engine"
kind = "apt"
packages = ["paired-engine"]

[[action]]
name = "Install the tool"
kind = "run"
do = ["./install"]
check = ["./installed"]
undo = ["./uninstall"]

[[action]]
name = "Pair the app"
kind = "run"
do = ["./pair"]
check = ["./paired"]
undo = ["./unpair"]
window = "Open the app and pair it"
"#);
    let world = World::new(&[("paired", &paired)]);
    world.script("paired", "install", "touch \"$HOME/installed\"");
    world.script("paired", "installed", "test -e \"$HOME/installed\"");
    world.script("paired", "uninstall", "rm -f \"$HOME/installed\"");
    world.script("paired", "pair", "touch \"$HOME/paired\"");
    world.script("paired", "paired", "test -e \"$HOME/paired\"");
    world.script("paired", "unpair", "rm -f \"$HOME/paired\"");
    world.install("paired", true).0.unwrap();
    world
        .host
        .state
        .borrow_mut()
        .fail
        .insert("apt-remove:paired-engine".to_owned());

    let (removed, events) = world.remove("paired");

    assert!(
        matches!(removed, Err(EngineError::RemovalRolledBack { .. })),
        "{removed:?}"
    );
    assert!(!world.host_home().join("paired").exists(), "no window opened");
    assert!(world.host_home().join("installed").exists());
    assert!(events.0.contains(&UpdateEvent::Note {
        text: "Open the app and pair it".to_owned()
    }));
    assert!(events.0.contains(&UpdateEvent::Note {
        text: "Install the tool: I ran it again to put it back, so what it makes is new.".to_owned()
    }));
    let user = world.journals("paired").0.unwrap();
    assert_eq!(
        user.entry(3).unwrap().state,
        crate::apps::journal::State::Undoing,
        "left for the user"
    );
}

#[test]
fn a_stopped_runs_undoing_entry_is_done_again_by_the_next_install() {
    let world = World::new(&[("grouped", GROUPED)]);
    world.install("grouped", true).0.unwrap();
    // A removal stopped during the group's undo: the entry marked, the membership gone.
    let mut root = world.journals("grouped").1.unwrap();
    let mut entry = root.entry(2).unwrap().clone();
    entry.state = crate::apps::journal::State::Undoing;
    root.put(entry);
    root.save(&world.root_journals).unwrap();
    world
        .host
        .state
        .borrow_mut()
        .groups
        .get_mut("grouped")
        .unwrap()
        .remove("kuba");

    world.install("grouped", true).0.unwrap();

    assert!(world.host.state.borrow().groups["grouped"].contains("kuba"));
    let root = world.journals("grouped").1.unwrap();
    assert_eq!(root.entry(2).unwrap().state, crate::apps::journal::State::Done);
}

/// A sink that reads the rollback mark of `app` as the first step a rollback does again is
/// reported, after the step that failed.
struct MarkAtFirstRedo {
    journals: PathBuf,
    app: &'static str,
    failed: bool,
    redone: bool,
    seen: Option<journal::RolledBack>,
}

impl Sink for MarkAtFirstRedo {
    fn event(&mut self, event: &UpdateEvent) {
        let UpdateEvent::Action(action) = event else {
            return;
        };
        match action.state {
            ActionState::Failed => self.failed = true,
            ActionState::Done if self.failed && !self.redone => {
                self.redone = true;
                self.seen = journal::rolled_back(&self.journals, self.app);
            }
            _ => {}
        }
    }
}

#[test]
fn the_rollback_is_marked_as_not_whole_before_its_first_step_is_done_again() {
    let world = World::new(&[("fixture", FIXTURE)]);
    world.install("fixture", true).0.unwrap();
    world
        .host
        .state
        .borrow_mut()
        .fail
        .insert("apt-remove:fixture-tool".to_owned());
    let mut sink = MarkAtFirstRedo {
        journals: world.user_journals.clone(),
        app: "fixture",
        failed: false,
        redone: false,
        seen: None,
    };
    let mut root = world.root();

    let removed = world.run(&mut root, &mut sink, true, |e| e.remove("fixture"));

    assert!(
        matches!(removed, Err(EngineError::RemovalRolledBack { .. })),
        "{removed:?}"
    );
    assert!(sink.redone, "a step was done again");
    let seen = sink.seen.expect("marked by then");
    assert!(!seen.whole && seen.why.starts_with("Install the tool: "), "{seen:?}");
    assert!(journal::rolled_back(&world.user_journals, "fixture").is_some_and(|m| m.whole));
}

#[test]
fn a_step_kubaos_did_that_owns_nothing_is_done_again_as_done() {
    let early = app(r#"
[[action]]
name = "Install the engine"
kind = "apt"
packages = ["early-engine"]

[[action]]
name = "Install the tool"
kind = "apt"
repo = "fixture"
packages = ["fixture-tool"]
"#);
    let world = World::new(&[("early", &early)]);
    // The tool is there already: the step sets its repository up and owns nothing.
    world
        .host
        .state
        .borrow_mut()
        .dpkg
        .insert("fixture-tool".to_owned(), "1.0".to_owned());
    world.install("early", true).0.unwrap();
    let state = |world: &World| world.journals("early").1.unwrap().entry(2).unwrap().state;
    assert_eq!(state(&world), crate::apps::journal::State::Done);
    world
        .host
        .state
        .borrow_mut()
        .fail
        .insert("apt-remove:early-engine".to_owned());

    let (removed, _) = world.remove("early");

    assert!(
        matches!(removed, Err(EngineError::RemovalRolledBack { .. })),
        "{removed:?}"
    );
    assert_eq!(state(&world), crate::apps::journal::State::Done);
}
