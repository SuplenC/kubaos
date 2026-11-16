use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;

use super::Catalog;
use super::Picks;
use super::Status;
use super::action::Kind;
use super::action::Words;
use super::catalog::CATALOG;
use super::catalog::CatalogError;
use super::journal::Journal;
use super::status::AppState;
use super::status::listing;
use crate::apt::Dpkg;
use crate::apt::Manifest;
use crate::apt::manifest::MANIFEST;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

const ABOUT: &str = r#"category = "development"
tags = ["fixture"]
summary = "An app for the tests"
description = "An app the tests install."
description_from = "https://example.com/app"
"#;

fn app(name: &str, actions: &str) -> String {
    format!("name = \"{name}\"\n{ABOUT}{actions}")
}

const AGENT: &str = "[[action]]\nname = \"Install Agent\"\nkind = \"apt\"\nrepo = \"vendor\"\npackages = [\"agent\"]\n";

const ARM_ONLY: &str = "[[action]]\nname = \"Install it\"\nkind = \"apt\"\narch = \"arm64\"\npackages = [\"tool\"]\n";

fn catalog_of(apps: &[(&str, String)]) -> (tempfile::TempDir, Catalog) {
    let dir = tempfile::tempdir().unwrap();
    for (id, text) in apps {
        fs::create_dir_all(dir.path().join(id)).unwrap();
        fs::write(dir.path().join(id).join("app.toml"), text).unwrap();
    }
    let catalog = Catalog::load(dir.path()).unwrap();
    (dir, catalog)
}

#[test]
fn the_catalog_loads_every_apps_folder_and_nothing_else() {
    let (dir, _) = catalog_of(&[("agent", app("Agent", AGENT))]);
    fs::write(dir.path().join("notes.toml"), "x").unwrap();
    fs::create_dir_all(dir.path().join("empty")).unwrap();
    fs::create_dir_all(dir.path().join("Not-An-Id")).unwrap();

    let catalog = Catalog::load(dir.path()).unwrap();

    let ids: Vec<&str> = catalog.apps().iter().map(|a| a.id.as_str()).collect();
    assert_eq!(ids, ["agent"]);
    assert_eq!(catalog.packages(), ["agent".to_owned()].into());
    assert_eq!(Catalog::load(&dir.path().join("missing")).unwrap(), Catalog::default());
}

#[test]
fn an_app_that_isnt_valid_is_named() {
    let (dir, _) = catalog_of(&[]);
    fs::create_dir_all(dir.path().join("bad")).unwrap();
    fs::write(dir.path().join("bad/app.toml"), "name = \"Bad\"\n").unwrap();

    let err = Catalog::load(dir.path()).unwrap_err();

    assert!(
        matches!(err, CatalogError::App { ref id, .. } if id == "bad"),
        "{err:?}"
    );
}

#[test]
fn an_installed_app_is_one_whose_user_journal_is_complete() {
    let (_dir, catalog) = catalog_of(&[
        ("agent", app("Agent", AGENT)),
        ("tool", app("Tool", ARM_ONLY)),
        ("half", app("Half", AGENT)),
    ]);
    let journals = tempfile::tempdir().unwrap();
    let mut done = Journal::new("agent", None);
    done.complete = true;
    done.entries.push(super::journal::Entry {
        step: 1,
        state: super::journal::State::Done,
        claims: vec![super::journal::Claim::new("package:agent".to_owned(), true)],
        prior: super::journal::Prior::default(),
        action: catalog.get("agent").unwrap().actions[0].clone(),
        replaced: None,
    });
    done.save(journals.path()).unwrap();
    // A run that stopped: steps journaled, not complete.
    let mut half = Journal::new("half", None);
    half.entries.push(super::journal::Entry {
        step: 1,
        state: super::journal::State::Doing,
        claims: Vec::new(),
        prior: super::journal::Prior::default(),
        action: catalog.get("half").unwrap().actions[0].clone(),
        replaced: None,
    });
    half.save(journals.path()).unwrap();
    let mut picks = Picks::default();
    picks.add("half");
    let dpkg = Dpkg::parse("installed install agent 1.2\n");

    let status = Status::read(
        &catalog,
        &picks,
        journals.path(),
        &journals.path().join("root"),
        "amd64",
        Some(&dpkg),
    )
    .unwrap();

    let state = |id: &str| status.get(id).unwrap().state;
    assert_eq!(state("agent"), AppState::Installed);
    assert_eq!(state("half"), AppState::Missing);
    assert_eq!(state("tool"), AppState::Unavailable);
    assert_eq!(status.get("agent").unwrap().version.as_deref(), Some("1.2"));
    assert_eq!(status.get("half").unwrap().version, None);
}

#[test]
fn an_app_kubaos_owns_nothing_of_is_found_and_one_with_no_journal_is_read_from_dpkg() {
    let (_dir, catalog) = catalog_of(&[
        ("agent", app("Agent", AGENT)),
        ("other", app("Other", &AGENT.replace("agent", "other"))),
        ("third", app("Third", &AGENT.replace("agent", "third"))),
        ("gone", app("Gone", &AGENT.replace("agent", "gone"))),
    ]);
    let journals = tempfile::tempdir().unwrap();
    let mut found = Journal::new("agent", None);
    found.complete = true;
    found.root_adoption = Some(super::journal::Adoption::Found);
    found.save(journals.path()).unwrap();
    let mut picks = Picks::default();
    picks.add("third");
    let dpkg = Dpkg::parse("installed install agent 1.2\ninstalled install other 2.0\ninstalled install third 3.0\n");

    let status = Status::read(
        &catalog,
        &picks,
        journals.path(),
        &journals.path().join("root"),
        "amd64",
        Some(&dpkg),
    )
    .unwrap();

    let state = |id: &str| status.get(id).unwrap().state;
    assert_eq!(state("agent"), AppState::Found);
    assert_eq!(state("other"), AppState::Found, "installed by hand, not picked");
    assert_eq!(
        state("third"),
        AppState::Installed,
        "picked, so taking it in makes it KubaOS's"
    );
    assert_eq!(state("gone"), AppState::Available);
    assert_eq!(status.get("other").unwrap().version.as_deref(), Some("2.0"));
}

#[test]
fn the_list_and_the_status_say_what_each_app_is_in_json() {
    let (_dir, catalog) = catalog_of(&[("agent", app("Agent", AGENT))]);
    let picks = Picks::default();

    let list = serde_json::to_value(listing(&catalog, &picks)).unwrap();
    let journals = tempfile::tempdir().unwrap();
    let status = serde_json::to_value(
        Status::read(
            &catalog,
            &picks,
            journals.path(),
            &journals.path().join("root"),
            "amd64",
            None,
        )
        .unwrap(),
    )
    .unwrap();

    assert_eq!(list[0]["category"], "development");
    assert_eq!(list[0]["tags"][0], "fixture");
    assert_eq!(status["apps"][0]["summary"], "An app for the tests");
    assert_eq!(status["apps"][0]["state"], "available");
}

#[test]
fn the_picks_keep_the_setup_of_a_picked_app_only() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("apps.toml");
    let mut picks = Picks::default();
    picks.add("agent");
    picks.add_setup("studio", "service");

    picks.write(&path).unwrap();
    let text = fs::read_to_string(&path).unwrap();
    let back = Picks::read(&path).unwrap();

    assert!(text.starts_with("# The apps of KubaOS's catalog"), "{text}");
    assert!(
        text.contains("apps = [\"agent\", \"studio\"]\n\n[setups]\nstudio = \"service\"\n"),
        "{text}"
    );
    assert_eq!(back, picks);
    fs::write(&path, "apps = [\"agent\"]\n[setups]\nstudio = \"service\"\n").unwrap();
    assert_eq!(Picks::read(&path).unwrap().setup("studio"), None);
    fs::write(&path, "apps = [\"editor\"]\n\n[nix_added]\neditor = [\"editor\"]\n").unwrap();
    assert!(Picks::read(&path).is_err());
}

/// The shipped catalog, read from the repository.
fn shipped() -> Catalog {
    Catalog::load(&repo_root().join(CATALOG)).unwrap()
}

#[test]
fn every_shipped_app_validates_and_its_repos_are_the_manifests() {
    let root = repo_root();
    let manifest = Manifest::read(&root.join(MANIFEST)).unwrap().unwrap();

    let catalog = shipped();

    catalog.check_repos(&manifest).unwrap();
    let ids: Vec<&str> = catalog.apps().iter().map(|a| a.id.as_str()).collect();
    assert_eq!(ids, ["bitwarden", "claude-code", "docker", "google-chrome", "t3-code"]);
    for app in catalog.apps() {
        for action in app.all_actions() {
            if let Kind::Apt { repo: Some(repo), .. } = &action.kind {
                let sources = root.join(format!("system/files/apt/sources.list.d/{repo}.sources"));
                assert!(sources.is_file(), "{}: no {}", app.id, sources.display());
            }
        }
    }
}

#[test]
fn every_shipped_step_resolves_on_both_architectures_with_its_files_there() {
    for app in shipped().apps() {
        for arch in ["amd64", "arm64"] {
            let words = Words {
                user: "kuba".to_owned(),
                home: PathBuf::from("/home/kuba"),
                arch: arch.to_owned(),
            };
            for action in app.all_actions() {
                action
                    .resolve(&words, &app.dir)
                    .unwrap_or_else(|e| panic!("{}: {}: {e}", app.id, action.name));
            }
        }
    }
}

#[test]
fn every_script_a_shipped_app_runs_can_run() {
    for app in shipped().apps() {
        for action in app.all_actions() {
            let Kind::Run { run, check, undo, .. } = &action.kind else {
                continue;
            };
            let readies = app.ready.iter().chain(app.setups.iter().flat_map(|s| &s.ready));
            let programs = [run, check, undo]
                .into_iter()
                .chain(readies.map(|r| &r.check))
                .filter_map(|argv| argv.first()?.strip_prefix("./"));
            for script in programs {
                let mode = fs::metadata(app.dir.join(script)).unwrap().permissions().mode();
                assert_eq!(mode & 0o111, 0o111, "{}: {script} isn't executable", app.id);
            }
        }
    }
}

#[test]
fn dockers_drop_in_is_the_one_the_base_installs_for_any_docker() {
    let root = repo_root();

    assert_eq!(
        fs::read_to_string(root.join("apps/docker/kubaos-stop.conf")).unwrap(),
        fs::read_to_string(root.join("system/files/systemd/docker.service.d/kubaos-stop.conf")).unwrap()
    );
}

#[test]
fn t3s_runtime_is_marked_whole_at_the_version_it_downloads() {
    let catalog = shipped();
    let t3 = catalog.get("t3-code").unwrap();
    let service = t3.setup("service").unwrap();

    let version = service
        .actions
        .iter()
        .find_map(|a| match &a.kind {
            Kind::Download { version, .. } => Some(version.clone()),
            _ => None,
        })
        .unwrap();
    let marked = service
        .actions
        .iter()
        .find_map(|a| match &a.kind {
            Kind::Run { run, .. } if run.first().is_some_and(|p| p == "./install-t3") => run.get(1).cloned(),
            _ => None,
        })
        .unwrap();

    assert_eq!(marked, format!("@HOME@/.t3/runtime/versions/{version}"));
}

#[test]
fn every_shipped_app_says_what_it_is_in_kubaos_voice_and_upstreams() {
    for app in shipped().apps() {
        assert!(!app.tags.is_empty(), "{}", app.id);
        assert!(
            !app.description_copied() || app.description_from.starts_with("https://"),
            "{}",
            app.id
        );
        assert!(
            app.summary.len() <= 100,
            "{}: a summary is one line of a menu: {}",
            app.id,
            app.summary
        );
    }
}
