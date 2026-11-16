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
use crate::apps::host::fake::Unit;
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
category = "system"
tags = ["fixture"]
description = "An app the tests install."
description_from = "https://example.com/fixture"

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
fn an_app_in_another_setup_loses_that_setups_own_steps_first() {
    let switching = r#"
name = "Two ways"
summary = "An app with setups"
category = "system"
tags = ["fixture"]
description = "An app the tests install."
description_from = "https://example.com/fixture"

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
         description = \"An app the tests install.\"\ndescription_from = \"https://example.com/test\"\n{actions}"
    )
}

impl World {
    /// The app `id`'s folder given `text`, as a release changes it.
    fn write_app(&self, id: &str, text: &str) {
        fs::write(self.catalog.join(id).join("app.toml"), text).unwrap();
    }

    /// A file of the app `id`'s folder.
    fn folder_file(&self, id: &str, name: &str, text: &str) {
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
fn an_undo_that_stopped_partway_is_marked_and_done_again_by_the_next_install() {
    let world = World::new(&[("grouped", GROUPED)]);
    world.install("grouped", true).0.unwrap();
    world.host.state.borrow_mut().fail.insert("groupdel:grouped".to_owned());

    let (removed, _) = world.remove("grouped");

    assert!(matches!(removed, Err(EngineError::Undo { .. })), "{removed:?}");
    let root = world.journals("grouped").1.unwrap();
    assert_eq!(root.entry(2).unwrap().state, crate::apps::journal::State::Undoing);
    assert!(!world.host.state.borrow().groups["grouped"].contains("kuba"));

    world.host.state.borrow_mut().fail.clear();
    world.install("grouped", true).0.unwrap();

    assert!(world.host.state.borrow().groups["grouped"].contains("kuba"));
    let root = world.journals("grouped").1.unwrap();
    assert_eq!(root.entry(2).unwrap().state, crate::apps::journal::State::Done);
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

    let Err(EngineError::Undo { source, .. }) = result else {
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
    let made = std::process::Command::new("tar")
        .arg("-czf")
        .arg(&archive)
        .arg("-C")
        .arg(src.path())
        .arg("pkg")
        .status()
        .unwrap();
    assert!(made.success());
    world.host.state.borrow_mut().downloads.insert(
        "https://example.com/runtime.tar.gz".to_owned(),
        ("4".repeat(64), fs::read(&archive).unwrap()),
    );
    let before = world.host.snapshot();
    // tar gives what it unpacks the archive's modes less the umask: under a umask that lets the
    // group write, as Ubuntu's 002 does, only the action itself keeps the files the owner's.
    let umask = Umask::set(0o002);

    world.install("runtime", true).0.unwrap();
    drop(umask);

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

/// The process's umask, set for a test and put back when it goes.
struct Umask(rustix::fs::Mode);

impl Umask {
    fn set(mask: u32) -> Self {
        Self(rustix::process::umask(rustix::fs::Mode::from_raw_mode(mask)))
    }
}

impl Drop for Umask {
    fn drop(&mut self) {
        rustix::process::umask(self.0);
    }
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
    assert!(
        entry.contains("Exec=google-chrome --app=https://sketch.example/ --class=kubaos-web-sketch\n"),
        "{entry}"
    );
    assert!(entry.contains("StartupWMClass=kubaos-web-sketch\n"), "{entry}");
    let icon = home.join(".local/share/icons/hicolor/scalable/apps/kubaos-web-sketch.svg");
    assert!(entry.contains(&format!("Icon={}\n", icon.display())), "{entry}");
    assert_eq!(fs::read_to_string(icon).unwrap(), "<svg/>\n");

    world.remove("sketch").0.unwrap();

    assert_eq!(world.host.snapshot(), before);
    assert!(world.host.state.borrow().nix.contains("fd"));
}
