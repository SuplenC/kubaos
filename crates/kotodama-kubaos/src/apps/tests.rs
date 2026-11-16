use std::collections::VecDeque;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;
use std::sync::Mutex;
use std::time::Duration;

use kotodama_protocol::JobState;
use kubaos_core::apps::App;
use kubaos_core::apps::Catalog;
use kubaos_core::apps::Picks;
use kubaos_core::apps::Status;
use kubaos_core::apps::plan::Facts;
use kubaos_core::apt::Dpkg;
use kubaos_core::apt::dpkg;
use kubaos_core::state::Paths;

use super::*;
use crate::fake::FakeRunner;
use crate::run::OnEnd;
use crate::run::OnLine;

const CHROME: &str = r#"
name = "Google Chrome"
summary = "Google's web browser"
icon = "google-chrome"

[apt]
repo = "google-chrome"
arch = "amd64"
packages = ["google-chrome-stable"]
"#;

const CLAUDE: &str = r#"
name = "Claude Code"
summary = "Anthropic's coding agent, in the terminal"

[apt]
repo = "claude-code"
packages = ["claude-code"]
"#;

/// A `.deb` app with a download for an architecture no machine has, so it is never offered.
const ODD: &str = r#"
name = "Odd"
summary = "Only for another machine"

[deb]
package = "odd"
version = "1.0"

[deb.arm64]
url = "https://example.com/odd-1.0-arm64.deb"
sha256 = "418c7825f73107b7b5ba3babae5e3b485801dc22d9dc0d518af632f7906cbdd1"
"#;

fn catalog() -> Catalog {
    Catalog::new(vec![
        App::parse("google-chrome", CHROME).unwrap(),
        App::parse("claude-code", CLAUDE).unwrap(),
        App::parse("odd", ODD).unwrap(),
    ])
}

/// dpkg with `installed`, each `name=version`.
fn dpkg_with(installed: &[&str]) -> String {
    let mut lines = String::new();
    for p in installed {
        let (name, version) = p.split_once('=').unwrap();
        let _ = writeln!(lines, "installed install {name} {version}");
    }
    lines
}

/// The apps of [`catalog`] on an amd64 machine with these dpkg packages.
fn apps(installed: &[&str]) -> Apps {
    apps_on("amd64", installed)
}

/// The apps of [`catalog`] on a machine of `arch` with these dpkg packages.
fn apps_on(arch: &str, installed: &[&str]) -> Apps {
    apps_of(&catalog(), arch, installed, &[])
}

/// The apps of `catalog` on a machine of `arch` with these dpkg packages and setups' markers.
fn apps_of(catalog: &Catalog, arch: &str, installed: &[&str], markers: &[&str]) -> Apps {
    let dpkg = Dpkg::parse(&dpkg_with(installed));
    let facts = Facts {
        catalog,
        picks: &Picks::default(),
        arch,
        dpkg: &dpkg,
        nix: &BTreeSet::new(),
        markers: &markers.iter().map(|m| (*m).to_owned()).collect(),
    };
    Apps::new(catalog, &Status::new(&facts))
}

fn ids(apps: &Apps) -> Vec<String> {
    apps.items().into_iter().map(|item| item.id).collect()
}

fn item(apps: &Apps, id: &str) -> Item {
    apps.items().into_iter().find(|item| item.id == id).expect("the Item")
}

#[test]
fn each_app_offers_only_the_half_that_applies() {
    let none = apps(&[]);
    let chrome = apps(&["google-chrome-stable=131.0"]);

    assert_eq!(ids(&none), ["app-install:claude-code", "app-install:google-chrome"]);
    assert_eq!(ids(&chrome), ["app-install:claude-code", "app-remove:google-chrome"]);
}

#[test]
fn an_apt_app_built_for_one_architecture_has_no_item_on_another() {
    let arm = apps_on("arm64", &[]);

    assert_eq!(ids(&arm), ["app-install:claude-code", "app-install:odd"]);
}

#[test]
fn the_items_are_named_by_the_app_and_found_by_its_name_and_id() {
    let apps = apps(&["claude-code=2.1"]);

    let install = item(&apps, "app-install:google-chrome");
    let remove = item(&apps, "app-remove:claude-code");

    assert_eq!(install.title, "Install Google Chrome");
    assert_eq!(install.subtitle.as_deref(), Some("Google's web browser"));
    assert_eq!(remove.title, "Remove Claude Code");
    assert_eq!(remove.subtitle.as_deref(), Some("Installed, version 2.1"));
    for word in ["chrome", "google", "google-chrome", "install"] {
        assert!(
            install.keywords.iter().any(|k| k == word),
            "{word} in {:?}",
            install.keywords
        );
    }
    for word in ["claude", "claude-code", "remove", "uninstall"] {
        assert!(
            remove.keywords.iter().any(|k| k == word),
            "{word} in {:?}",
            remove.keywords
        );
    }
}

#[test]
fn no_word_saying_what_an_app_is_finds_its_install_or_remove_item() {
    let none = apps(&[]);
    let chrome = apps(&["google-chrome-stable=131.0"]);

    for item in [
        item(&none, "app-install:google-chrome"),
        item(&chrome, "app-remove:google-chrome"),
    ] {
        for word in ["web", "browser"] {
            assert!(
                !item.keywords.iter().any(|k| k == word),
                "`{}` has `{word}` as a keyword, which belongs to opening the app",
                item.id
            );
        }
    }
}

#[test]
fn the_icon_is_the_manifests_or_a_generic_one() {
    let apps = apps(&[]);

    assert_eq!(
        item(&apps, "app-install:google-chrome").icon.as_deref(),
        Some("google-chrome")
    );
    assert_eq!(item(&apps, "app-install:claude-code").icon.as_deref(), Some(ICON));
}

#[test]
fn removing_asks_first_and_installing_asks_only_for_root() {
    let apps = apps(&["claude-code=2.1"]);

    let install = item(&apps, "app-install:google-chrome").actions;
    let remove = item(&apps, "app-remove:claude-code").actions;

    assert_eq!((install[0].id.as_str(), install[0].confirm), (INSTALL, false));
    assert_eq!((remove[0].id.as_str(), remove[0].confirm), (REMOVE, true));
}

#[test]
fn hide_may_name_either_half_of_every_app() {
    assert_eq!(
        apps(&[]).ids(),
        [
            "app-install:claude-code",
            "app-remove:claude-code",
            "app-install:google-chrome",
            "app-remove:google-chrome",
            "app-install:odd",
            "app-remove:odd",
        ]
    );
}

fn start(apps: &Apps, id: &str, action: &str) -> (AppJobs, FakeRunner, Result<Job, String>) {
    let mut jobs = AppJobs::default();
    let runner = FakeRunner::default();
    let started = jobs.start(apps, id, action, &runner);
    (jobs, runner, started)
}

#[test]
fn installing_runs_kubaos_app_add_in_the_background_as_a_job() {
    let (_, runner, started) = start(&apps(&[]), "app-install:google-chrome", INSTALL);

    let job = started.unwrap();
    assert_eq!(runner.backgrounded(), ["kubaos app add google-chrome --json"]);
    assert_eq!(
        (job.title.as_str(), job.state),
        ("Installing Google Chrome", JobState::Running)
    );
}

#[test]
fn removing_runs_kubaos_app_remove() {
    let (_, runner, started) = start(&apps(&["claude-code=2.1"]), "app-remove:claude-code", REMOVE);

    assert_eq!(started.unwrap().title, "Removing Claude Code");
    assert_eq!(runner.backgrounded(), ["kubaos app remove claude-code --json"]);
}

#[test]
fn a_half_that_doesnt_apply_or_an_app_not_offered_runs_nothing() {
    let apps = apps(&["claude-code=2.1"]);
    for (id, action) in [
        ("app-install:claude-code", INSTALL),
        ("app-remove:google-chrome", REMOVE),
        ("app-install:google-chrome", REMOVE),
        ("app-install:odd", INSTALL),
        ("app-install:gone", INSTALL),
    ] {
        let (_, runner, started) = start(&apps, id, action);

        assert!(started.is_err(), "{id} {action} started");
        assert!(runner.backgrounded().is_empty(), "{id} {action} ran");
    }
}

#[test]
fn one_app_changes_at_a_time() {
    let apps = apps(&[]);
    let (mut jobs, runner, started) = start(&apps, "app-install:google-chrome", INSTALL);
    started.unwrap();

    let second = jobs.start(&apps, "app-install:claude-code", INSTALL, &runner);

    assert_eq!(
        second.unwrap_err(),
        "Installing Google Chrome is still going: one app at a time"
    );
    assert_eq!(runner.backgrounded().len(), 1);
}

#[test]
fn a_program_that_cant_start_is_no_job() {
    let apps = apps(&[]);
    let mut jobs = AppJobs::default();
    let runner = FakeRunner {
        failing: Some("no kubaos".to_owned()),
        ..FakeRunner::default()
    };

    assert!(
        jobs.start(&apps, "app-install:google-chrome", INSTALL, &runner)
            .is_err()
    );
    assert!(
        jobs.start(&apps, "app-install:google-chrome", INSTALL, &FakeRunner::default())
            .is_ok()
    );
}

/// A job started for Google Chrome, and what it reports for each of `lines`, then its end.
fn heard(lines: &[&str], succeeded: bool) -> (Vec<Job>, Job) {
    let (mut jobs, _, started) = start(&apps(&[]), "app-install:google-chrome", INSTALL);
    let id = started.unwrap().id;
    let reported = lines
        .iter()
        .filter_map(|line| jobs.heard(Heard::line(&id, *line)))
        .collect();
    let ended = jobs.heard(Heard::ended(&id, succeeded)).expect("the end is reported");
    (reported, ended)
}

const LOG: &str = "/home/u/.local/state/kubaos/update/2026-11-16-app.log";

/// The summary `kubaos app add` ends with, having installed `installed`.
fn summary(installed: &[&str]) -> String {
    let installed: Vec<String> = installed.iter().map(|name| format!("\"{name}\"")).collect();
    format!(
        r#"{{"type":"summary","command":"app","from":{{"version":"0.1.0","commit":"a"}},"to":{{"version":"0.1.0","commit":"a"}},"channel":"stable","changes":[],"packages":0,"apps":{{"installed":[{}]}},"inputs":[],"migrations":[],"reloaded":[],"restart":"none","log":"{LOG}"}}"#,
        installed.join(",")
    )
}

const STARTED: &str =
    r#"{"type":"started","command":"app","log":"/home/u/.local/state/kubaos/update/2026-11-16-app.log"}"#;
const STEP: &str = r#"{"type":"step","text":"Installing Google Chrome"}"#;

#[test]
fn the_job_shows_each_step_and_how_far_it_got() {
    let (reported, _) = heard(
        &[
            STARTED,
            r#"{"type":"phase_started","phase":"apps"}"#,
            STEP,
            r#"{"type":"output","line":"Unpacking google-chrome-stable"}"#,
            r#"{"type":"progress","done":1,"total":4}"#,
        ],
        true,
    );

    let shown: Vec<(Option<&str>, Option<f32>)> = reported
        .iter()
        .map(|job| (job.message.as_deref(), job.progress.map(Fraction::get)))
        .collect();
    assert_eq!(
        shown,
        [
            (Some("Installing Google Chrome"), None),
            (Some("Installing Google Chrome"), Some(0.25)),
        ]
    );
}

#[test]
fn a_run_that_went_through_ends_named_for_the_app_with_its_log() {
    let summary = summary(&["Google Chrome"]);

    let (_, ended) = heard(&[STARTED, STEP, &summary], true);

    assert_eq!(
        (ended.title.as_str(), ended.state),
        ("Google Chrome installed", JobState::Succeeded)
    );
    assert_eq!(
        ended.message.as_deref(),
        Some("The log: /home/u/.local/state/kubaos/update/2026-11-16-app.log")
    );
}

#[test]
fn a_failed_run_ends_with_why_and_its_log() {
    let failed = r#"{"type":"failed","phase":"apps","error":"installing the apps: pkexec exited with 126","log":"/home/u/.local/state/kubaos/update/2026-11-16-app.log"}"#;

    let (_, ended) = heard(&[STARTED, STEP, failed], false);

    assert_eq!(
        (ended.title.as_str(), ended.state),
        ("Google Chrome wasn't installed", JobState::Failed)
    );
    assert_eq!(
        ended.message.as_deref(),
        Some(
            "installing the apps: pkexec exited with 126\nThe log: \
             /home/u/.local/state/kubaos/update/2026-11-16-app.log"
        )
    );
}

#[test]
fn a_warning_beside_the_app_installed_is_said_but_fails_nothing() {
    let warning = r#"{"type":"warning","text":"no log of this run: disk full"}"#;
    let summary = summary(&["Google Chrome"]);

    let (_, ended) = heard(&[warning, &summary], true);

    assert_eq!(
        (ended.title.as_str(), ended.state),
        ("Google Chrome installed", JobState::Succeeded)
    );
    assert_eq!(
        ended.message.as_deref(),
        Some(
            "no log of this run: disk full\nThe log: \
             /home/u/.local/state/kubaos/update/2026-11-16-app.log"
        )
    );
}

#[test]
fn a_summary_without_the_app_is_a_failure() {
    let summary = summary(&["Claude Code"]);

    let (_, ended) = heard(&[STARTED, &summary], true);

    assert_eq!(
        (ended.title.as_str(), ended.state),
        ("Google Chrome wasn't installed", JobState::Failed)
    );
}

#[test]
fn an_app_missing_after_its_install_is_a_failure() {
    let warning = r#"{"type":"warning","text":"Google Chrome isn't on the machine after its install"}"#;
    let summary = summary(&[]);

    let (_, ended) = heard(&[STARTED, warning, &summary], true);

    assert_eq!(ended.state, JobState::Failed);
    assert_eq!(
        ended.message.as_deref(),
        Some(
            "Google Chrome isn't on the machine after its install\nThe log: \
             /home/u/.local/state/kubaos/update/2026-11-16-app.log"
        )
    );
}

#[test]
fn a_run_that_stopped_before_its_events_ends_with_what_it_said() {
    let (_, ended) = heard(&["error: kubaos.d is locked by another kubaos"], false);

    assert_eq!(ended.state, JobState::Failed);
    assert_eq!(
        ended.message.as_deref(),
        Some("error: kubaos.d is locked by another kubaos")
    );
}

#[test]
fn a_run_that_says_nothing_ends_naming_the_command() {
    let (_, ended) = heard(&[], false);

    assert_eq!(ended.message.as_deref(), Some("kubaos app add google-chrome stopped"));
}

#[test]
fn another_jobs_lines_change_nothing_and_once_ended_the_next_app_may_start() {
    let apps = apps(&[]);
    let (mut jobs, runner, started) = start(&apps, "app-install:google-chrome", INSTALL);
    let id = started.unwrap().id;

    assert_eq!(jobs.heard(Heard::line("app-99", STEP)), None);
    assert_eq!(jobs.heard(Heard::ended("app-99", true)), None);
    jobs.heard(Heard::ended(&id, true)).unwrap();
    let next = jobs.start(&apps, "app-install:claude-code", INSTALL, &runner).unwrap();

    assert_ne!(next.id, id);
}

/// The files of a machine under `dir` whose catalog has `manifests`, each `(id, text)`.
fn files(dir: &Path, manifests: &[(&str, &str)]) -> AppFiles {
    let catalog = dir.join("apps");
    fs::create_dir(&catalog).unwrap();
    for (id, text) in manifests {
        fs::write(catalog.join(format!("{id}.toml")), text).unwrap();
    }
    let paths = Paths::from_vars(Some(dir.join("home").into()), None, None).unwrap();
    AppFiles::new(catalog, &paths)
}

fn query() -> String {
    std::iter::once("dpkg-query")
        .chain(dpkg::QUERY)
        .collect::<Vec<_>>()
        .join(" ")
}

#[test]
fn the_apps_are_read_with_one_dpkg_query() {
    let dir = tempfile::tempdir().unwrap();
    let files = files(dir.path(), &[("google-chrome", CHROME), ("claude-code", CLAUDE)]);
    let runner = FakeRunner {
        outputs: [(query(), dpkg_with(&["claude-code=2.1"]))].into(),
        ..FakeRunner::default()
    };

    let apps = Apps::read(&files, &runner).unwrap();

    assert_eq!(ids(&apps), ["app-remove:claude-code", "app-install:google-chrome"]);
    assert_eq!(runner.ran(), [query()]);
}

#[test]
fn an_empty_catalog_asks_dpkg_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let files = files(dir.path(), &[]);
    let runner = FakeRunner::default();

    let apps = Apps::read(&files, &runner).unwrap();

    assert_eq!(apps, Apps::default());
    assert!(runner.ran().is_empty(), "{:?}", runner.ran());
}

/// Answers each `dpkg-query` with the next of `answers`, a failure saying its `Err`, and tells
/// `asked` once it has.
struct Answers {
    answers: Mutex<VecDeque<Result<String, String>>>,
    asked: mpsc::Sender<()>,
}

impl Runner for Answers {
    fn output(&self, argv: &[String]) -> Result<String, RunError> {
        let answer = self.answers.lock().unwrap().pop_front().expect("an answer left");
        self.asked.send(()).unwrap();
        answer.map_err(|stderr| RunError::Failed {
            program: argv[0].clone(),
            stderr,
        })
    }

    fn launch(&self, _argv: &[String], _delay: Duration) -> Result<(), RunError> {
        Ok(())
    }

    fn background(&self, _argv: &[String], _line: OnLine, _ended: OnEnd) -> Result<(), RunError> {
        Ok(())
    }
}

#[test]
fn a_reading_that_fails_is_given_to_no_one_so_the_last_one_stands() {
    let dir = tempfile::tempdir().unwrap();
    let files = files(dir.path(), &[("google-chrome", CHROME), ("claude-code", CLAUDE)]);
    let (asked, dpkg_asked) = mpsc::channel();
    let runner = Answers {
        answers: Mutex::new(VecDeque::from([
            Ok(dpkg_with(&["claude-code=2.1"])),
            Err("dpkg-query: database locked".to_owned()),
            Ok(dpkg_with(&[])),
        ])),
        asked,
    };
    let (sender, readings) = mpsc::channel();
    let reader = AppsReader::start(files, runner, move |apps| sender.send(apps).unwrap()).unwrap();
    let wait = Duration::from_secs(5);

    reader.read();
    let first = readings.recv_timeout(wait).unwrap();
    reader.read();
    dpkg_asked.recv_timeout(wait).unwrap();
    dpkg_asked.recv_timeout(wait).unwrap();
    reader.read();
    let next = readings.recv_timeout(wait).unwrap();

    assert_eq!(ids(&first), ["app-remove:claude-code", "app-install:google-chrome"]);
    assert_eq!(ids(&next), ["app-install:claude-code", "app-install:google-chrome"]);
}

#[test]
fn the_watch_covers_dpkg_the_picks_and_the_users_nix_packages() {
    let paths = Paths::from_vars(Some("/home/u".into()), None, None).unwrap();
    let files = AppFiles::new("/usr/share/kubaos/apps".into(), &paths);

    let watched = files.watched();

    assert!(watched.contains(&PathBuf::from(DPKG_STATUS)), "{watched:?}");
    assert!(watched.contains(&paths.app_picks()), "{watched:?}");
    assert!(watched.contains(&paths.state_dir().packages_file()), "{watched:?}");
}

/// An app with two setups: its desktop app alone, or a service beside it.
const LAB: &str = r#"
name = "Lab"
summary = "An app for agents"

[deb]
package = "lab"
version = "2.0"
self_updates = true

[deb.amd64]
url = "https://example.com/lab-amd64.deb"
sha256 = "418c7825f73107b7b5ba3babae5e3b485801dc22d9dc0d518af632f7906cbdd1"

[setup.desktop]
name = "Lab"
summary = "The desktop app"

[setup.service]
name = "Lab as a service"
summary = "Its agents in a service"
user_hooks = "lab_service"
marker = ".config/systemd/user/lab.service"
"#;

const LAB_MARKER: &str = ".config/systemd/user/lab.service";

/// Lab alone, on an amd64 machine with these dpkg packages and setups' markers.
fn lab(installed: &[&str], markers: &[&str]) -> Apps {
    let catalog = Catalog::new(vec![App::parse("lab", LAB).unwrap()]);
    apps_of(&catalog, "amd64", installed, markers)
}

#[test]
fn an_app_with_setups_offers_an_install_for_each_and_none_for_the_app_alone() {
    let apps = lab(&[], &[]);

    assert_eq!(ids(&apps), ["app-install:lab:desktop", "app-install:lab:service"]);
    let desktop = item(&apps, "app-install:lab:desktop");
    let service = item(&apps, "app-install:lab:service");
    assert_eq!(
        (desktop.title.as_str(), desktop.subtitle.as_deref()),
        ("Install Lab", Some("The desktop app"))
    );
    assert_eq!(
        (service.title.as_str(), service.subtitle.as_deref()),
        ("Install Lab as a service", Some("Its agents in a service"))
    );
    assert_eq!(service.actions[0].title, "Install Lab as a service");
}

#[test]
fn an_app_in_one_setup_offers_its_removal_and_the_switch_to_each_other() {
    let desktop = lab(&["lab=2.0"], &[]);
    let service = lab(&["lab=2.0"], &[LAB_MARKER]);

    assert_eq!(ids(&desktop), ["app-remove:lab", "app-install:lab:service"]);
    assert_eq!(
        item(&desktop, "app-remove:lab").subtitle.as_deref(),
        Some("Installed: Lab, version 2.0")
    );
    assert_eq!(
        item(&desktop, "app-install:lab:service").subtitle.as_deref(),
        Some("Replaces Lab: Its agents in a service")
    );
    assert_eq!(ids(&service), ["app-remove:lab", "app-install:lab:desktop"]);
    assert_eq!(
        item(&service, "app-remove:lab").subtitle.as_deref(),
        Some("Installed: Lab as a service, version 2.0")
    );
}

#[test]
fn installing_a_setup_runs_kubaos_app_add_with_it_named_for_the_setup() {
    let (_, runner, started) = start(&lab(&[], &[]), "app-install:lab:service", INSTALL);

    assert_eq!(started.unwrap().title, "Installing Lab as a service");
    assert_eq!(runner.backgrounded(), ["kubaos app add lab --setup service --json"]);
}

#[test]
fn a_setup_install_ends_installed_when_the_summary_names_the_setup() {
    let (mut jobs, _, started) = start(&lab(&[], &[]), "app-install:lab:service", INSTALL);
    let id = started.unwrap().id;
    jobs.heard(Heard::line(&id, summary(&["Lab as a service"])));

    let ended = jobs.heard(Heard::ended(&id, true)).unwrap();

    assert_eq!((ended.title.as_str(), ended.state), ("Lab as a service installed", JobState::Succeeded));
}

#[test]
fn a_setup_item_that_doesnt_apply_runs_nothing() {
    let service = lab(&["lab=2.0"], &[LAB_MARKER]);
    for (apps, id) in [
        (&service, "app-install:lab:service"),
        (&service, "app-install:lab"),
        (&service, "app-install:lab:server"),
        (&service, "app-remove:lab:service"),
        (&lab(&[], &[]), "app-install:lab"),
    ] {
        let action = if id.starts_with("app-remove") { REMOVE } else { INSTALL };
        let (_, runner, started) = start(apps, id, action);

        assert!(started.is_err(), "{id} started");
        assert!(runner.backgrounded().is_empty(), "{id} ran");
    }
}

#[test]
fn hide_may_name_each_setups_install_and_the_removal() {
    assert_eq!(
        lab(&[], &[]).ids(),
        ["app-install:lab:desktop", "app-install:lab:service", "app-remove:lab"]
    );
}

#[test]
fn the_markers_are_read_from_the_home_and_watched() {
    let dir = tempfile::tempdir().unwrap();
    let files = files(dir.path(), &[("lab", LAB)]);
    let marker = dir.path().join("home").join(LAB_MARKER);
    fs::create_dir_all(marker.parent().unwrap()).unwrap();
    fs::write(&marker, "").unwrap();
    let runner = FakeRunner {
        outputs: [(query(), dpkg_with(&["lab=2.0"]))].into(),
        ..FakeRunner::default()
    };

    let apps = Apps::read(&files, &runner).unwrap();

    assert_eq!(ids(&apps), ["app-remove:lab", "app-install:lab:desktop"]);
    assert!(files.watched().contains(&marker), "{:?}", files.watched());
}
