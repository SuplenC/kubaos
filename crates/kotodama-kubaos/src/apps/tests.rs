use std::collections::VecDeque;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;
use std::sync::Mutex;
use std::time::Duration;

use kotodama_protocol::JobState;
use kubaos_core::apps::Catalog;
use kubaos_core::apps::Picks;
use kubaos_core::apps::Platform;
use kubaos_core::apps::Status;
use kubaos_core::apps::journal::Adoption;
use kubaos_core::apps::journal::Journal;
use kubaos_core::apt::Dpkg;
use kubaos_core::apt::dpkg;
use kubaos_core::state::Paths;
use kubaos_core::unfree::Unfree;

use super::*;
use crate::fake::FakeRunner;
use crate::run::OnEnd;
use crate::run::OnLine;

/// What every fixture app says it is, beside its name, summary and actions.
const ABOUT: &str = r#"category = "development"
description = "An app the tests install."
description_from = "https://example.com/app"
"#;

const CHROME: &str = r#"
name = "Google Chrome"
summary = "Google's web browser"
tags = ["browser", "web"]
icon = "google-chrome"

[[action]]
name = "Install Google Chrome"
kind = "apt"
repo = "google-chrome"
arch = "amd64"
packages = ["google-chrome-stable"]
"#;

const CLAUDE: &str = r#"
name = "Claude Code"
summary = "Anthropic's coding agent, in the terminal"
tags = ["coding agent"]

[[action]]
name = "Install Claude Code"
kind = "apt"
repo = "claude-code"
packages = ["claude-code"]
"#;

/// A `.deb` app with a download for an architecture no machine has, so it is never offered.
const ODD: &str = r#"
name = "Odd"
summary = "Only for another machine"
tags = ["odd"]

[[action]]
name = "Install Odd"
kind = "deb"
package = "odd"
version = "1.0"
arm64 = { url = "https://example.com/odd-1.0-arm64.deb", sha256 = "418c7825f73107b7b5ba3babae5e3b485801dc22d9dc0d518af632f7906cbdd1" }
"#;

/// The catalog of the tests: Chrome, Claude Code and Odd.
const FOLDERS: [(&str, &str); 3] = [("google-chrome", CHROME), ("claude-code", CLAUDE), ("odd", ODD)];

/// dpkg with `installed`, each `name=version`.
fn dpkg_with(installed: &[&str]) -> String {
    let mut lines = String::new();
    for p in installed {
        let (name, version) = p.split_once('=').unwrap();
        let _ = writeln!(lines, "installed install {name} {version}");
    }
    lines
}

/// Writes `folders`, each `(id, app.toml)`, as the catalog `dir`.
fn write_catalog(dir: &Path, folders: &[(&str, &str)]) {
    for (id, text) in folders {
        fs::create_dir_all(dir.join(id)).unwrap();
        fs::write(dir.join(id).join("app.toml"), format!("{ABOUT}{text}")).unwrap();
    }
}

/// A complete user journal of the app `id`, in `setup`, in `journals`.
fn installed_journal(journals: &Path, id: &str, setup: Option<&str>) {
    let mut journal = Journal::new(id, setup);
    journal.complete = true;
    journal.root_adoption = Some(Adoption::Owned);
    journal.save(journals).unwrap();
}

/// The apps of [`FOLDERS`] on an amd64 machine with these dpkg packages.
fn apps(installed: &[&str]) -> Apps {
    apps_on("amd64", installed)
}

/// The apps of [`FOLDERS`] on a machine of `arch` with these dpkg packages.
fn apps_on(arch: &str, installed: &[&str]) -> Apps {
    apps_of(&FOLDERS, arch, installed, None)
}

/// The apps of `folders` on a machine of `arch` with these dpkg packages, each app whose
/// packages all are installed, an app with setups in `setup` (else its desktop one).
fn apps_of(folders: &[(&str, &str)], arch: &str, installed: &[&str], setup: Option<&str>) -> Apps {
    let dir = tempfile::tempdir().unwrap();
    write_catalog(&dir.path().join("apps"), folders);
    let catalog = Catalog::load(&dir.path().join("apps")).unwrap();
    let journals = dir.path().join("journals");
    let names: Vec<&str> = installed.iter().map(|p| p.split_once('=').unwrap().0).collect();
    for app in catalog.apps() {
        let packages = app.packages();
        if !packages.is_empty() && packages.iter().all(|p| names.contains(p)) {
            let setup = app.has_setups().then(|| setup.unwrap_or("desktop"));
            installed_journal(&journals, &app.id, setup);
        }
    }
    let dpkg = Dpkg::parse(&dpkg_with(installed));
    Apps::new(
        &Status::read(
            &catalog,
            &Picks::default(),
            &journals,
            &journals.join("root"),
            Platform::new(arch, Unfree::Allowed),
            Some(&dpkg),
        )
        .unwrap(),
    )
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
fn an_apps_tags_find_its_install_item_and_never_its_removal() {
    let none = apps(&[]);
    let chrome = apps(&["google-chrome-stable=131.0"]);

    let install = item(&none, "app-install:google-chrome");
    let remove = item(&chrome, "app-remove:google-chrome");

    for word in ["web", "browser"] {
        assert!(
            install.keywords.iter().any(|k| k == word),
            "{word} in {:?}",
            install.keywords
        );
        assert!(
            !remove.keywords.iter().any(|k| k == word),
            "`{}` has `{word}` as a keyword, which belongs to opening the app",
            remove.id
        );
    }
}

#[test]
fn the_icon_is_the_folders_or_a_generic_one() {
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
fn hide_may_name_every_item_of_every_app() {
    assert_eq!(
        apps(&[]).ids(),
        [
            "app-install:claude-code",
            "app-remove:claude-code",
            "app-configure:claude-code",
            "app-install:google-chrome",
            "app-remove:google-chrome",
            "app-configure:google-chrome",
            "app-install:odd",
            "app-remove:odd",
            "app-configure:odd",
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
            (Some("Installing Google Chrome, 2 of 4"), Some(0.25)),
        ]
    );
}

/// What a running Job shows after each of `lines` that changed it: its message and progress.
fn shown(lines: &[&str]) -> Vec<(Option<String>, Option<f32>)> {
    let (reported, _) = heard(lines, true);
    reported
        .iter()
        .map(|job| (job.message.clone(), job.progress.map(Fraction::get)))
        .collect()
}

#[test]
fn the_job_names_the_step_it_is_on_and_its_place_among_the_apps_steps() {
    let shown = shown(&[
        STARTED,
        r#"{"type":"app","app":"t3-code","name":"T3 Code","n":1,"of":1}"#,
        r#"{"type":"step","text":"Install T3 Code"}"#,
        r#"{"type":"progress","done":0,"total":4}"#,
        r#"{"type":"step","text":"Start the service"}"#,
        r#"{"type":"progress","done":1,"total":4}"#,
        r#"{"type":"progress","done":4,"total":4}"#,
    ]);

    assert_eq!(
        shown,
        [
            (Some("Install T3 Code".to_owned()), None),
            (Some("Install T3 Code, 1 of 4".to_owned()), Some(0.0)),
            (Some("Start the service".to_owned()), Some(0.0)),
            (Some("Start the service, 2 of 4".to_owned()), Some(0.25)),
            (Some("Start the service, 4 of 4".to_owned()), Some(1.0)),
        ]
    );
}

#[test]
fn a_run_of_several_apps_names_the_app_it_is_on_and_its_place_in_the_run() {
    let shown = shown(&[
        STARTED,
        r#"{"type":"app","app":"neovim","name":"Neovim","n":1,"of":2}"#,
        r#"{"type":"step","text":"Install Neovim"}"#,
        r#"{"type":"progress","done":0,"total":1}"#,
        r#"{"type":"action","app":"neovim","step":1,"of":1,"name":"Install Neovim","state":"done"}"#,
        r#"{"type":"progress","done":1,"total":1}"#,
        r#"{"type":"app","app":"lazyvim","name":"LazyVim","n":2,"of":2}"#,
        r#"{"type":"step","text":"Write its config"}"#,
        r#"{"type":"progress","done":0,"total":2}"#,
    ]);

    assert_eq!(
        shown,
        [
            (Some("Neovim (1 of 2)".to_owned()), None),
            (Some("Neovim (1 of 2): Install Neovim".to_owned()), None),
            (Some("Neovim (1 of 2): Install Neovim, 1 of 1".to_owned()), Some(0.0)),
            (Some("Neovim (1 of 2): Install Neovim, 1 of 1".to_owned()), Some(0.5)),
            (Some("LazyVim (2 of 2)".to_owned()), Some(0.5)),
            (Some("LazyVim (2 of 2): Write its config".to_owned()), Some(0.5)),
            (Some("LazyVim (2 of 2): Write its config, 1 of 2".to_owned()), Some(0.5)),
        ]
    );
}

/// An app on Neovim with a configure stage.
const LAZYVIM: &str = r#"
name = "LazyVim"
summary = "Neovim, set up"
tags = ["editor"]

[[action]]
name = "Install LazyVim"
kind = "apt"
packages = ["lazyvim"]

[[action]]
name = "Write KubaOS's defaults"
kind = "file"
stage = "configure"
to = "~/.config/kubaos/lazyvim/defaults.lua"
text = "vim.o.number = true\n"
"#;

#[test]
fn an_installed_app_with_a_configure_stage_has_a_configure_item_and_others_none() {
    let folders = [("lazyvim", LAZYVIM), ("claude-code", CLAUDE)];

    let installed = apps_of(&folders, "amd64", &["lazyvim=1.0", "claude-code=2.1"], None);
    let missing = apps_of(&folders, "amd64", &[], None);

    let configure = item(&installed, "app-configure:lazyvim");
    assert_eq!(configure.title, "Configure LazyVim");
    assert_eq!(
        configure.subtitle.as_deref(),
        Some("Put KubaOS's defaults for LazyVim back")
    );
    assert!(!ids(&installed).contains(&"app-configure:claude-code".to_owned()));
    assert!(!ids(&missing).iter().any(|id| id.starts_with("app-configure:")));
}

#[test]
fn configuring_runs_kubaos_app_configure_and_ends_when_the_summary_names_the_app() {
    let apps = apps_of(&[("lazyvim", LAZYVIM)], "amd64", &["lazyvim=1.0"], None);
    let (mut jobs, runner, started) = start(&apps, "app-configure:lazyvim", CONFIGURE);
    let job = started.unwrap();
    let summary = summary(&[]).replace(r#""installed":[]"#, r#""configured":["LazyVim"]"#);

    jobs.heard(Heard::line(&job.id, summary));
    let ended = jobs.heard(Heard::ended(&job.id, true)).unwrap();

    assert_eq!(runner.backgrounded(), ["kubaos app configure lazyvim --json"]);
    assert_eq!(job.title, "Configuring LazyVim");
    assert_eq!(
        (ended.title.as_str(), ended.state),
        ("LazyVim configured", JobState::Succeeded)
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

/// The files of a machine under `dir` whose catalog has `folders`, each `(id, app.toml)`.
fn files(dir: &Path, folders: &[(&str, &str)]) -> AppFiles {
    let system = SystemPaths::under(dir);
    let catalog = system.catalog();
    fs::create_dir_all(&catalog).unwrap();
    write_catalog(&catalog, folders);
    AppFiles::new(&system, &paths(dir))
}

/// The paths of the user whose home is `<dir>/home`.
fn paths(dir: &Path) -> Paths {
    Paths::from_vars(Some(dir.join("home").into()), None, None).unwrap()
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
    installed_journal(&paths(dir.path()).app_journals(), "claude-code", None);
    let runner = FakeRunner {
        outputs: [(query(), dpkg_with(&["claude-code=2.1"]))].into(),
        ..FakeRunner::default()
    };

    let apps = Apps::read(&files, &runner).unwrap();

    assert_eq!(ids(&apps), ["app-remove:claude-code", "app-install:google-chrome"]);
    assert_eq!(
        item(&apps, "app-remove:claude-code").subtitle.as_deref(),
        Some("Installed, version 2.1")
    );
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
    let journals = paths(dir.path()).app_journals();
    installed_journal(&journals, "claude-code", None);
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
    Journal::forget(&journals, "claude-code").unwrap();
    reader.read();
    dpkg_asked.recv_timeout(wait).unwrap();
    dpkg_asked.recv_timeout(wait).unwrap();
    reader.read();
    let next = readings.recv_timeout(wait).unwrap();

    assert_eq!(ids(&first), ["app-remove:claude-code", "app-install:google-chrome"]);
    assert_eq!(ids(&next), ["app-install:claude-code", "app-install:google-chrome"]);
}

#[test]
fn the_watch_covers_dpkg_the_picks_and_each_apps_journal() {
    let dir = tempfile::tempdir().unwrap();
    let files = files(dir.path(), &[("google-chrome", CHROME)]);
    let paths = paths(dir.path());
    std::fs::create_dir_all(paths.app_journals().join("google-chrome")).unwrap();

    let watched = files.watched_dirs();

    assert!(watched.contains(&PathBuf::from("/var/lib/dpkg")), "{watched:?}");
    assert!(
        watched.contains(&paths.app_picks().parent().unwrap().to_path_buf()),
        "{watched:?}"
    );
    assert!(
        watched.contains(&paths.app_journals().join("google-chrome")),
        "{watched:?}"
    );
    assert!(files.changes(Path::new(DPKG_STATUS)));
    assert!(files.changes(&paths.app_picks()));
    assert!(files.changes(&Journal::file(&paths.app_journals(), "google-chrome")));
    assert!(!files.changes(&paths.app_journals().join(".lock")));
}

#[test]
fn turning_unfree_packages_on_or_off_is_watched_as_it_changes_which_apps_can_be_installed() {
    let dir = tempfile::tempdir().unwrap();
    let files = files(dir.path(), &[("google-chrome", CHROME)]);
    let state_dir = paths(dir.path()).state_dir().root().to_path_buf();

    assert!(files.watched_dirs().contains(&state_dir));
    assert!(files.changes(&state_dir.join("nixpkgs.toml")));
    assert!(!files.changes(&state_dir.join("packages.toml")));
}

#[test]
fn the_journals_folder_is_watched_before_any_app_has_one() {
    // Regression: right after the update that brought journals, their folder wasn't there, and
    // the menu never heard of the first one.
    let dir = tempfile::tempdir().unwrap();
    let files = files(dir.path(), &[]);
    let paths = paths(dir.path());
    std::fs::create_dir_all(paths.app_journals()).unwrap();

    let watched = files.watched_dirs();

    assert!(watched.contains(&paths.app_journals()), "{watched:?}");
    assert!(files.changes(&paths.app_journals().join("t3-code")));
}

/// An app with two setups: its desktop app alone, or a service beside it.
const LAB: &str = r#"
name = "Lab"
summary = "An app for agents"
tags = ["agents"]

[[action]]
name = "Install Lab"
kind = "deb"
package = "lab"
version = "2.0"
self_updates = true
amd64 = { url = "https://example.com/lab-amd64.deb", sha256 = "418c7825f73107b7b5ba3babae5e3b485801dc22d9dc0d518af632f7906cbdd1" }

[setup.desktop]
name = "Lab"
summary = "The desktop app"

[setup.service]
name = "Lab as a service"
summary = "Its agents in a service"

[[setup.service.action]]
name = "Keep it running"
kind = "linger"
"#;

/// Lab alone, on an amd64 machine with these dpkg packages, installed in `setup` when lab is.
fn lab(installed: &[&str], setup: Option<&str>) -> Apps {
    apps_of(&[("lab", LAB)], "amd64", installed, setup)
}

#[test]
fn an_app_installed_by_other_means_has_no_remove_item() {
    let dir = tempfile::tempdir().unwrap();
    write_catalog(&dir.path().join("apps"), &FOLDERS);
    let catalog = Catalog::load(&dir.path().join("apps")).unwrap();
    let journals = dir.path().join("journals");
    let mut found = Journal::new("claude-code", None);
    found.complete = true;
    found.root_adoption = Some(Adoption::Found);
    found.save(&journals).unwrap();
    let dpkg = Dpkg::parse(&dpkg_with(&["claude-code=2.1", "google-chrome-stable=140"]));

    let apps = Apps::new(
        &Status::read(
            &catalog,
            &Picks::default(),
            &journals,
            &journals.join("root"),
            Platform::new("amd64", Unfree::Allowed),
            Some(&dpkg),
        )
        .unwrap(),
    );

    assert!(ids(&apps).is_empty(), "{:?}", ids(&apps));
}

#[test]
fn removing_an_app_kubaos_didnt_install_ends_saying_it_left_it() {
    let (mut jobs, _, started) = start(&apps(&["claude-code=2.1"]), "app-remove:claude-code", REMOVE);
    let id = started.unwrap().id;
    let summary = summary(&[]).replace(r#""apps":{"installed":[]}"#, r#""apps":{"left":["Claude Code"]}"#);
    jobs.heard(Heard::line(&id, &summary));

    let ended = jobs.heard(Heard::ended(&id, true)).unwrap();

    assert_eq!(
        (ended.title.as_str(), ended.state),
        ("Claude Code left as it is", JobState::Succeeded)
    );
    assert!(
        ended
            .message
            .as_deref()
            .is_some_and(|m| m.starts_with("I didn't install Claude Code, so it's not mine to bin")),
        "{ended:?}"
    );
}

#[test]
fn an_app_with_setups_offers_an_install_for_each_and_none_for_the_app_alone() {
    let apps = lab(&[], None);

    assert_eq!(ids(&apps), ["app-install:lab:desktop", "app-install:lab:service"]);
    let desktop = item(&apps, "app-install:lab:desktop");
    let service = item(&apps, "app-install:lab:service");
    assert_eq!(
        (desktop.title.as_str(), desktop.subtitle.as_deref()),
        ("Install Lab", Some("An app for agents"))
    );
    assert_eq!(
        (service.title.as_str(), service.subtitle.as_deref()),
        ("Install Lab as a service", Some("An app for agents"))
    );
    assert_eq!(service.actions[0].title, "Install Lab as a service");
}

#[test]
fn an_app_in_one_setup_offers_its_removal_and_the_switch_to_each_other() {
    let desktop = lab(&["lab=2.0"], None);
    let service = lab(&["lab=2.0"], Some("service"));

    assert_eq!(ids(&desktop), ["app-remove:lab", "app-install:lab:service"]);
    assert_eq!(
        item(&desktop, "app-remove:lab").subtitle.as_deref(),
        Some("Installed: Lab, version 2.0")
    );
    assert_eq!(
        item(&desktop, "app-install:lab:service").subtitle.as_deref(),
        Some("Replaces Lab: An app for agents")
    );
    assert_eq!(ids(&service), ["app-remove:lab", "app-install:lab:desktop"]);
    assert_eq!(
        item(&service, "app-remove:lab").subtitle.as_deref(),
        Some("Installed: Lab as a service, version 2.0")
    );
}

#[test]
fn installing_a_setup_runs_kubaos_app_add_with_it_named_for_the_setup() {
    let (_, runner, started) = start(&lab(&[], None), "app-install:lab:service", INSTALL);

    assert_eq!(started.unwrap().title, "Installing Lab as a service");
    assert_eq!(runner.backgrounded(), ["kubaos app add lab --setup service --json"]);
}

#[test]
fn a_setup_install_ends_installed_when_the_summary_names_the_setup() {
    let (mut jobs, _, started) = start(&lab(&[], None), "app-install:lab:service", INSTALL);
    let id = started.unwrap().id;
    jobs.heard(Heard::line(&id, summary(&["Lab as a service"])));

    let ended = jobs.heard(Heard::ended(&id, true)).unwrap();

    assert_eq!(
        (ended.title.as_str(), ended.state),
        ("Lab as a service installed", JobState::Succeeded)
    );
}

#[test]
fn a_setup_item_that_doesnt_apply_runs_nothing() {
    let service = lab(&["lab=2.0"], Some("service"));
    for (apps, id) in [
        (&service, "app-install:lab:service"),
        (&service, "app-install:lab"),
        (&service, "app-install:lab:server"),
        (&service, "app-remove:lab:service"),
        (&lab(&[], None), "app-install:lab"),
    ] {
        let action = if id.starts_with("app-remove") { REMOVE } else { INSTALL };
        let (_, runner, started) = start(apps, id, action);

        assert!(started.is_err(), "{id} started");
        assert!(runner.backgrounded().is_empty(), "{id} ran");
    }
}

#[test]
fn hide_may_name_each_setups_install_the_removal_and_the_configure() {
    assert_eq!(
        lab(&[], None).ids(),
        [
            "app-install:lab:desktop",
            "app-install:lab:service",
            "app-remove:lab",
            "app-configure:lab",
        ]
    );
}

#[test]
fn the_setup_an_app_is_in_is_its_journals() {
    let dir = tempfile::tempdir().unwrap();
    let files = files(dir.path(), &[("lab", LAB)]);
    installed_journal(&paths(dir.path()).app_journals(), "lab", Some("service"));
    let runner = FakeRunner {
        outputs: [(query(), dpkg_with(&["lab=2.0"]))].into(),
        ..FakeRunner::default()
    };

    let apps = Apps::read(&files, &runner).unwrap();

    assert_eq!(ids(&apps), ["app-remove:lab", "app-install:lab:desktop"]);
}

/// The Items of the shipped catalog on an amd64 machine: every app's Install Items with none
/// installed, then every app's Remove Item with all of them installed by KubaOS.
fn shipped_items() -> Vec<Item> {
    let dir = tempfile::tempdir().unwrap();
    let catalog = Catalog::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../apps")).unwrap();
    let journals = dir.path().join("journals");
    let read = |journals: &Path| {
        let status = Status::read(
            &catalog,
            &Picks::default(),
            journals,
            &journals.join("root"),
            "amd64",
            Some(&Dpkg::default()),
        )
        .unwrap();
        Apps::new(&status).items()
    };
    let mut items = read(&journals);
    for app in catalog.apps() {
        let setup = app.setups.first().map(|s| s.id.as_str());
        installed_journal(&journals, &app.id, setup);
    }
    items.extend(read(&journals));
    items
}

/// Google Chrome's own Item, as Kotodama's apps Extension makes it from Chrome's desktop entry:
/// its name the title, its generic name a keyword and the subtitle, its desktop id a weak keyword.
fn open_google_chrome() -> Item {
    Item {
        id: "google-chrome".to_owned(),
        kind: ItemKind::App {
            desktop_id: "google-chrome.desktop".to_owned(),
        },
        identity: Some("app:google-chrome.desktop".to_owned()),
        title: "Google Chrome".to_owned(),
        subtitle: Some("Web Browser".to_owned()),
        icon: Some("google-chrome".to_owned()),
        names: Vec::new(),
        keywords: vec!["Web Browser".to_owned()],
        hidden: false,
        app_only: false,
        weak_keywords: vec!["google-chrome".to_owned()],
        yields: false,
        actions: vec![Action::new("open", "Open")],
    }
}

/// How well `item` matches `query` in the Daemon's ranking: `None` when it doesn't.
fn tier(item: &Item, query: &str) -> Option<kotodamad::rank::Tier> {
    let haystack = kotodamad::rank::Haystack::new(item);
    kotodamad::rank::Tiers::default().texts(
        haystack.texts(),
        &kotodamad::rank::Query::new(query),
        kotodamad::rank::Matched::ALL,
    )
}

#[test]
fn no_install_or_remove_item_of_the_catalog_ranks_above_opening_google_chrome() {
    let items = shipped_items();
    let chrome = open_google_chrome();

    for query in ["browser", "chrome", "google", "web"] {
        let open = tier(&chrome, query);
        assert!(open.is_some(), "{query} finds Google Chrome");
        for item in &items {
            // On an equal match an app ranks before an action, so a tie leaves Chrome first.
            assert!(
                tier(item, query) <= open,
                "{query}: {} ({:?}) above Google Chrome ({open:?})",
                item.title,
                tier(item, query)
            );
        }
    }
}

#[test]
fn typing_chrome_or_google_ranks_google_chromes_items_above_chromiums() {
    let items = shipped_items();
    let of = |id: &str| -> Vec<&Item> { items.iter().filter(|i| i.id.rsplit(':').next() == Some(id)).collect() };
    let (chrome, chromium) = (of("google-chrome"), of("chromium"));
    assert_eq!((chrome.len(), chromium.len()), (2, 2), "an Install and a Remove each");

    for query in ["chrome", "google"] {
        for theirs in &chromium {
            for ours in &chrome {
                assert!(
                    tier(ours, query) > tier(theirs, query),
                    "{query}: {} ({:?}) not above {} ({:?})",
                    ours.title,
                    tier(ours, query),
                    theirs.title,
                    tier(theirs, query)
                );
            }
        }
    }
}
