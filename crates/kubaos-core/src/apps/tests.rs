use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;

use super::Catalog;
use super::Picks;
use super::Platform;
use super::Status;
use super::action::Kind;
use super::action::Words;
use super::catalog::CATALOG;
use super::catalog::CatalogError;
use super::folder::FolderError;
use super::journal::Journal;
use super::status::AppState;
use super::status::listing;
use crate::apt::Dpkg;
use crate::apt::Manifest;
use crate::apt::manifest::MANIFEST;
use crate::sandbox;
use crate::unfree::Unfree;

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
fn an_app_requiring_one_the_catalog_lacks_or_one_with_setups_is_refused() {
    let lab = format!(
        "{}[setup.a]\nname = \"A\"\nsummary = \"A\"\n[setup.b]\nname = \"B\"\nsummary = \"B\"\n",
        app("Lab", AGENT)
    );
    for (requires, expected) in [
        ("missing", "requires missing, which the catalog doesn't have"),
        ("lab", "can be set up more than one way"),
    ] {
        let dir = tempfile::tempdir().unwrap();
        for (id, text) in [
            ("agent", app("Agent", &format!("requires = [\"{requires}\"]\n{AGENT}"))),
            ("lab", lab.clone()),
        ] {
            fs::create_dir_all(dir.path().join(id)).unwrap();
            fs::write(dir.path().join(id).join("app.toml"), text).unwrap();
        }

        let err = Catalog::load(dir.path()).unwrap_err();

        assert!(err.to_string().contains(expected), "{err}");
    }
}

#[test]
fn apps_requiring_each_other_in_a_circle_are_refused_naming_it() {
    let dir = tempfile::tempdir().unwrap();
    for (id, requires) in [("agent", "desk"), ("desk", "editor"), ("editor", "agent")] {
        fs::create_dir_all(dir.path().join(id)).unwrap();
        let text = app(id, &format!("requires = [\"{requires}\"]\n{AGENT}"));
        fs::write(dir.path().join(id).join("app.toml"), text).unwrap();
    }

    let err = Catalog::load(dir.path()).unwrap_err();

    assert_eq!(
        err.to_string(),
        "apps require each other in a circle: agent → desk → editor → agent"
    );
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
fn the_status_shows_an_svg_icon_by_its_path_in_the_catalog_whether_or_not_the_app_is_there() {
    let (_dir, catalog) = catalog_of(&[(
        "agent",
        app("Agent", AGENT).replace("name = \"Agent\"", "name = \"Agent\"\nicon = \"agent.svg\""),
    )]);
    let journals = tempfile::tempdir().unwrap();

    let status = Status::read(
        &catalog,
        &Picks::default(),
        journals.path(),
        &journals.path().join("root"),
        Platform::new("amd64", Unfree::Allowed),
        None,
    )
    .unwrap();

    let dir = &catalog.get("agent").unwrap().dir;
    assert_eq!(
        status.get("agent").unwrap().icon.as_deref(),
        Some(dir.join("agent.svg").to_str().unwrap())
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
        Platform::new("amd64", Unfree::Allowed),
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
fn an_unfree_nix_app_is_unavailable_while_the_users_nixpkgs_refuses_unfree_and_so_is_one_requiring_it() {
    let chat = "[[action]]\nname = \"Install Chat\"\nkind = \"nix\"\npackages = [\"discord\"]\nunfree = true\n";
    let free = "[[action]]\nname = \"Install Free\"\nkind = \"nix\"\npackages = [\"element-desktop\"]\n";
    let (_dir, catalog) = catalog_of(&[
        ("chat", app("Chat", chat)),
        ("free", app("Free", free)),
        ("bot", app("Bot", &format!("requires = [\"chat\"]\n{AGENT}"))),
    ]);
    let journals = tempfile::tempdir().unwrap();
    let read = |unfree| {
        Status::read(
            &catalog,
            &Picks::default(),
            journals.path(),
            &journals.path().join("root"),
            Platform::new("amd64", unfree),
            None,
        )
        .unwrap()
    };

    let allowed = read(Unfree::Allowed);
    let refused = read(Unfree::Refused);

    for id in ["chat", "free", "bot"] {
        assert_eq!(allowed.get(id).unwrap().state, AppState::Available, "{id}");
    }
    assert_eq!(refused.get("chat").unwrap().state, AppState::Unavailable);
    assert_eq!(refused.get("bot").unwrap().state, AppState::Unavailable);
    assert_eq!(refused.get("free").unwrap().state, AppState::Available);
}

#[test]
fn a_nix_step_for_one_architecture_has_nothing_for_another() {
    let zoom = "[[action]]\nname = \"Install Zoom\"\nkind = \"nix\"\narch = \"amd64\"\npackages = [\"zoom-us\"]\n";
    let (_dir, catalog) = catalog_of(&[("zoom", app("Zoom", zoom))]);
    let zoom = catalog.get("zoom").unwrap();

    assert!(catalog.available(zoom, None, Platform::new("amd64", Unfree::Allowed)));
    assert!(!catalog.available(zoom, None, Platform::new("arm64", Unfree::Allowed)));
    let wrong = app("Zoom", &zoom_with_arch("riscv64"));
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("zoom")).unwrap();
    fs::write(dir.path().join("zoom/app.toml"), wrong).unwrap();
    assert!(Catalog::load(dir.path()).is_err());
}

#[test]
fn an_unfree_nix_app_lists_the_architectures_it_installs_on_whatever_the_user_lets_in() {
    let zoom = "[[action]]\nname = \"Install Zoom\"\nkind = \"nix\"\narch = \"amd64\"\npackages = [\"zoom-us\"]\nunfree = true\n";
    let (_dir, catalog) = catalog_of(&[("zoom", app("Zoom", zoom))]);

    let list = serde_json::to_value(listing(&catalog, &Picks::default())).unwrap();

    assert_eq!(list[0]["arches"], serde_json::json!(["amd64"]));
}

fn zoom_with_arch(arch: &str) -> String {
    format!("[[action]]\nname = \"Install Zoom\"\nkind = \"nix\"\narch = \"{arch}\"\npackages = [\"zoom-us\"]\n")
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
        Platform::new("amd64", Unfree::Allowed),
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
fn ubuntus_snap_firefox_is_not_firefox_and_mozillas_is() {
    let catalog = shipped();
    let journals = tempfile::tempdir().unwrap();
    let mut picks = Picks::default();
    picks.add("firefox");
    let state = |dpkg: &str, picks: &Picks| {
        let dpkg = Dpkg::parse(dpkg);
        let status = Status::read(
            &catalog,
            picks,
            journals.path(),
            &journals.path().join("root"),
            Platform::new("amd64", Unfree::Allowed),
            Some(&dpkg),
        )
        .unwrap();
        status.get("firefox").unwrap().state
    };
    let snap = "installed install firefox 1:1snap1-0ubuntu9.1\n";
    let mozillas = "installed install firefox 160.0~build1\n";

    assert_eq!(state(snap, &Picks::default()), AppState::Available);
    assert_eq!(state(snap, &picks), AppState::Missing);
    assert_eq!(state(mozillas, &Picks::default()), AppState::Found);
    assert_eq!(state(mozillas, &picks), AppState::Installed);
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
            Platform::new("amd64", Unfree::Allowed),
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

/// An app of `category` whose `about` follows its name.
fn app_in(name: &str, category: &str, rest: &str) -> String {
    format!(
        "name = \"{name}\"\ncategory = \"{category}\"\ntags = [\"fixture\"]\nsummary = \"{name}, for the tests\"\n\
         description = \"An app the tests list.\"\ndescription_from = \"kubaos\"\n{rest}"
    )
}

#[test]
fn the_list_groups_the_apps_by_category_and_says_where_each_comes_from_and_runs() {
    let (_dir, catalog) = catalog_of(&[
        (
            "chat",
            app_in(
                "Chat",
                "communication",
                "requires = [\"browser\"]\n[[action]]\nname = \"Window\"\nkind = \"web_app\"\nid = \"chat\"\n\
                 title = \"Chat\"\nurl = \"https://chat.example/\"\nicon = \"chat.svg\"\n",
            ),
        ),
        (
            "browser",
            app_in(
                "Browser",
                "browsers",
                "[[action]]\nname = \"Install it\"\nkind = \"apt\"\nrepo = \"vendor\"\narch = \"amd64\"\npackages = [\"browser\"]\n",
            ),
        ),
        (
            "editor",
            app_in(
                "Editor",
                "development",
                "[[action]]\nname = \"Install it\"\nkind = \"apt\"\npackages = [\"editor\"]\n\
                 [[action]]\nname = \"Its plugins\"\nkind = \"nix\"\npackages = [\"editor-plugins\"]\n\
                 [[action]]\nname = \"Again\"\nkind = \"apt\"\npackages = [\"editor-extra\"]\n",
            ),
        ),
        (
            "atlas",
            app_in(
                "Atlas",
                "development",
                "[setup.web]\nname = \"Atlas\"\nsummary = \"in a window\"\nrequires = [\"browser\"]\n\
                 [[setup.web.action]]\nname = \"Window\"\nkind = \"web_app\"\nid = \"atlas\"\ntitle = \"Atlas\"\n\
                 url = \"https://atlas.example/\"\nicon = \"atlas.svg\"\n\
                 [setup.local]\nname = \"Atlas here\"\nsummary = \"on this machine\"\n\
                 [[setup.local.action]]\nname = \"Install it\"\nkind = \"deb\"\npackage = \"atlas\"\nversion = \"1.0\"\n\
                 arm64 = { url = \"https://atlas.example/atlas-arm64.deb\", sha256 = \"{}\" }\n",
            )
            .replace("{}", &"0".repeat(64)),
        ),
    ]);

    let picks = Picks::default();
    let listed = listing(&catalog, &picks);
    let list = serde_json::to_value(&listed).unwrap();

    let ids: Vec<&str> = listed.iter().map(|a| a.id).collect();
    assert_eq!(ids, ["browser", "atlas", "editor", "chat"]);
    assert_eq!(
        list[0]["sources"],
        serde_json::json!([{ "kind": "apt_repo", "repo": "vendor" }])
    );
    assert_eq!(list[0]["arches"], serde_json::json!(["amd64"]));
    assert!(list[0].get("requires").is_none(), "{}", list[0]);
    assert!(list[1].get("sources").is_none(), "{}", list[1]);
    assert_eq!(list[1]["setups"][0]["sources"], serde_json::json!([{ "kind": "deb" }]));
    assert_eq!(
        list[1]["setups"][1]["sources"],
        serde_json::json!([{ "kind": "web_app" }])
    );
    assert_eq!(list[1]["arches"], serde_json::json!(["amd64", "arm64"]));
    assert_eq!(list[1]["setups"][0]["id"], "local");
    assert!(list[1]["setups"][0].get("requires").is_none(), "{}", list[1]);
    assert_eq!(list[1]["setups"][1]["requires"], serde_json::json!(["browser"]));
    assert_eq!(
        list[2]["sources"],
        serde_json::json!([{ "kind": "archive" }, { "kind": "nixpkgs" }])
    );
    assert_eq!(list[2]["arches"], serde_json::json!(["amd64", "arm64"]));
    assert_eq!(list[3]["requires"], serde_json::json!(["browser"]));
    assert_eq!(list[3]["arches"], serde_json::json!(["amd64"]));
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
    assert_eq!(
        ids,
        [
            "aider",
            "alacritty",
            "bitwarden",
            "brave",
            "chatgpt",
            "chromium",
            "claude-code",
            "claude-desktop",
            "codex",
            "cursor",
            "devin-desktop",
            "discord",
            "docker",
            "element",
            "emacs",
            "firefox",
            "gemini-cli",
            "github-copilot-cli",
            "google-chrome",
            "helix",
            "jetbrains-toolbox",
            "kitty",
            "librewolf",
            "lm-studio",
            "mattermost",
            "microsoft-edge",
            "microsoft-teams",
            "neovide",
            "opencode",
            "penpot",
            "proton-mail-bridge",
            "signal",
            "slack",
            "sublime-text",
            "t3-code",
            "telegram",
            "thunderbird",
            "tor-browser",
            "vivaldi",
            "vscode",
            "wezterm",
            "whatsapp",
            "zed",
            "zen-browser",
            "zoom",
            "zulip",
        ]
    );
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
fn every_repositorys_sources_file_is_signed_by_the_key_the_manifest_pins() {
    let root = repo_root();
    let manifest = Manifest::read(&root.join(MANIFEST)).unwrap().unwrap();

    for (name, repo) in manifest.repos() {
        let sources = fs::read_to_string(root.join(format!("system/files/apt/sources.list.d/{name}.sources"))).unwrap();
        let signed_by: Vec<&str> = sources
            .lines()
            .filter_map(|l| l.strip_prefix("Signed-By:"))
            .map(str::trim)
            .collect();
        let keyrings: Vec<&str> = repo.keyrings().collect();
        assert_eq!(signed_by, [keyrings.join(" ")], "{name}");
    }
}

#[test]
fn firefoxs_pin_is_there_before_apt_picks_a_firefox() {
    // Ubuntu's firefox, which installs the snap, has an epoch, so without the pin apt takes it.
    let catalog = shipped();
    let firefox = catalog.get("firefox").unwrap();

    let kinds: Vec<&Kind> = firefox.all_actions().map(|a| &a.kind).collect();

    assert!(
        matches!(kinds[..], [Kind::File { to, .. }, Kind::Apt { repo: Some(repo), .. }]
            if to == "/etc/apt/preferences.d/kubaos-firefox" && repo == "mozilla"),
        "{kinds:?}"
    );
    let pin = fs::read_to_string(firefox.dir.join("kubaos-firefox.pref")).unwrap();
    assert!(
        pin.contains("Package: firefox ") && pin.contains("Pin: origin packages.mozilla.org\nPin-Priority: 1000\n")
    );
}

#[test]
fn every_shipped_step_resolves_on_both_architectures_with_its_files_there() {
    for app in shipped().apps() {
        for arch in ["amd64", "arm64"] {
            let words = Words {
                user: "kuba".to_owned(),
                home: PathBuf::from("/home/kuba"),
                arch: arch.to_owned(),
                version: app.version().map(str::to_owned),
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

#[test]
fn penpot_opens_in_a_window_and_requires_chrome_and_docker_for_its_local_stack() {
    let catalog = shipped();
    let penpot = catalog.get("penpot").unwrap();

    let required = |setup: &str| -> Vec<&str> {
        penpot
            .requires(penpot.setup(setup))
            .map(super::folder::Requirement::id)
            .collect()
    };
    assert_eq!(required("web"), ["google-chrome"]);
    assert_eq!(required("local"), ["docker", "google-chrome"]);
    let local = penpot.setup("local").unwrap();
    assert!(
        local
            .keeps
            .as_deref()
            .is_some_and(|k| k.contains("penpot_penpot_postgres_v15"))
    );
    let compose_run = local
        .actions
        .iter()
        .find_map(|a| match &a.kind {
            Kind::Run { run, root, undo, .. } => Some((run.clone(), *root, undo.clone())),
            _ => None,
        })
        .unwrap();
    assert_eq!(compose_run, (vec!["./up".to_owned()], true, vec!["./down".to_owned()]));
    for arch in ["amd64", "arm64"] {
        assert_eq!(
            catalog.available(penpot, penpot.setup("web"), Platform::new(arch, Unfree::Allowed)),
            arch == "amd64",
            "{arch}: Chrome is built for amd64 alone"
        );
    }
}

#[test]
fn penpots_stack_is_pinned_by_digest_bound_to_this_machine_and_keeps_its_volumes() {
    let folder = repo_root().join("apps/penpot");
    let compose = fs::read_to_string(folder.join("docker-compose.yaml")).unwrap();
    let down = fs::read_to_string(folder.join("down")).unwrap();

    let images: Vec<&str> = compose
        .lines()
        .filter_map(|line| line.trim().strip_prefix("image: "))
        .collect();
    assert_eq!(images.len(), 8, "{images:?}");
    for image in images {
        let digest = image.trim_matches('"').split_once("@sha256:").map(|(_, d)| d);
        assert!(
            digest.is_some_and(|d| d.len() == 64 && d.chars().all(|c| c.is_ascii_hexdigit())),
            "{image} isn't pinned by its digest"
        );
    }
    let mut in_ports = false;
    for line in compose.lines() {
        let item = line.trim();
        if item.ends_with(':') {
            in_ports = item == "ports:";
        } else if in_ports && let Some(port) = item.strip_prefix("- ") {
            assert!(port.starts_with("127.0.0.1:"), "{port} reaches past this machine");
        }
    }
    assert!(
        compose.contains("- 127.0.0.1:9002:8080"),
        "the socket's service passes 9001 on to 9002"
    );
    assert!(compose.contains("PENPOT_PUBLIC_URI: http://localhost:9001"));
    assert!(compose.contains("PENPOT_TELEMETRY_ENABLED: \"false\""));
    assert!(
        !compose.contains("8025"),
        "the mail catcher's page shows password resets to anyone on the machine"
    );
    assert!(down.contains("docker-compose.yaml\" down --remove-orphans\n"), "{down}");
    assert!(!down.contains("--volumes") && !down.contains(" -v"), "{down}");
}

#[test]
fn penpots_proxy_runs_walled_off_as_a_user_of_its_own_and_its_stack_starts_through_up_as_root() {
    let service = fs::read_to_string(repo_root().join("apps/penpot/kubaos-penpot.service")).unwrap();
    let lines: Vec<&str> = service.lines().collect();

    for line in [
        "DynamicUser=yes",
        "ProtectSystem=strict",
        "ProtectHome=yes",
        "NoNewPrivileges=yes",
        "PrivateDevices=yes",
        "PrivateTmp=yes",
        "RestrictAddressFamilies=AF_INET AF_INET6 AF_UNIX",
        "CapabilityBoundingSet=",
    ] {
        assert!(lines.contains(&line), "{line} is missing");
    }
    assert!(
        lines.contains(&"ExecStartPre=+/usr/share/kubaos/apps/penpot/up"),
        "the stack starts as root through `up`, which records the images it pulls"
    );
    assert!(repo_root().join("apps/penpot/up").is_file());
    assert!(lines.contains(&"ExecStart=/usr/lib/systemd/systemd-socket-proxyd 127.0.0.1:9002"));
}

/// Penpot's script `name`, with the stack's folder moved to `dir`.
fn penpot_script(name: &str, into: &Path, dir: &Path) -> PathBuf {
    let text = fs::read_to_string(repo_root().join("apps/penpot").join(name)).unwrap();
    let from = "dir=/etc/kubaos/penpot\n";
    assert!(text.contains(from), "{name} keeps the stack in /etc/kubaos/penpot");
    let path = into.join(name);
    sandbox::program(&path, &text.replace(from, &format!("dir={}\n", dir.display())));
    path
}

#[test]
fn penpots_removal_takes_the_images_its_install_pulled_and_keeps_the_ones_the_machine_had() {
    let tmp = tempfile::tempdir().unwrap();
    let (home, dir, images) = (
        tmp.path().join("home"),
        tmp.path().join("penpot"),
        tmp.path().join("images"),
    );
    for folder in [&home, &dir, &images] {
        fs::create_dir_all(folder).unwrap();
    }
    let sandbox = crate::sandbox::Sandbox::new(&tmp.path().join("sandbox"), &home, "tester");
    let (theirs, ours) = (
        "postgres:15@sha256:2663a9f0fde52de94f5dad3603a1942359f49472f1017c1a6d64f66bfedacb40",
        "penpotapp/frontend:2.19.2@sha256:27af83e128712c6e621afcbc4dbae5a607204f7acc89a9ae1245b5601b4a66a8",
    );
    // A daemon whose images are files named for their references.
    sandbox.stub(
        "docker",
        &format!(
            r#"images='{images}'
file() {{ printf '%s/%s' "$images" "$(printf %s "$1" | tr '/:@' '___')"; }}
case "$1 $2" in
"image inspect") [ -e "$(file "$3")" ]; exit ;;
"image rm") rm "$(file "$3")"; exit ;;
esac
for arg; do
	case $arg in
	config) printf '%s\n' '{theirs}' '{ours}'; exit 0 ;;
	up) touch "$(file '{theirs}')" "$(file '{ours}')"; exit 0 ;;
	down) exit 0 ;;
	esac
done
exit 1"#,
            images = images.display()
        ),
    );
    let had = images.join(theirs.replace(['/', ':', '@'], "_"));
    fs::write(&had, "").unwrap();
    let pulled = images.join(ours.replace(['/', ':', '@'], "_"));
    // The secret an earlier run made, so this run makes only the list of pulled images.
    fs::write(dir.join("secret.env"), "PENPOT_SECRET_KEY=earlier\n").unwrap();
    fs::set_permissions(dir.join("secret.env"), fs::Permissions::from_mode(0o600)).unwrap();
    let (up, down) = (
        penpot_script("up", tmp.path(), &dir),
        penpot_script("down", tmp.path(), &dir),
    );

    let ran_up = sandbox.command(&up).output().unwrap();
    assert!(ran_up.status.success(), "{ran_up:?}");
    assert!(pulled.exists() && had.exists());
    let mode = fs::metadata(dir.join("pulled-images")).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o600, "pulled-images is readable by more than root");
    let ran_down = sandbox.command(&down).output().unwrap();

    assert!(ran_down.status.success(), "{ran_down:?}");
    assert!(!pulled.exists(), "the image the install pulled stayed");
    assert!(had.exists(), "the image the machine had went");
    assert!(!dir.join("pulled-images").exists() && !dir.join("secret.env").exists());
}

#[test]
fn a_catalog_named_by_hand_must_be_there() {
    let dir = tempfile::tempdir().unwrap();

    let err = Catalog::load_existing(&dir.path().join("typo")).unwrap_err();

    assert!(matches!(err, CatalogError::Read { .. }), "{err:?}");
    assert_eq!(Catalog::load_existing(dir.path()).unwrap(), Catalog::default());
}

/// A setup `local` whose install runs `./up`, with `rest` after `kind = "run"`, beside a web setup.
fn stack_app(rest: &str) -> String {
    app_in(
        "Stack",
        "graphics",
        &format!(
            "[setup.web]\nname = \"Stack\"\nsummary = \"in a window\"\n\
             [[setup.web.action]]\nname = \"Window\"\nkind = \"web_app\"\nid = \"stack\"\ntitle = \"Stack\"\n\
             url = \"https://stack.example/\"\nicon = \"stack.svg\"\n\
             [setup.local]\nname = \"Stack here\"\nsummary = \"on this machine\"\n\
             [[setup.local.action]]\nname = \"Start it\"\nkind = \"run\"\nroot = true\ndo = [\"./up\"]\n\
             check = [\"./up-to-date\"]\nundo = [\"./down\"]\n{rest}\
             [[setup.local.action]]\nname = \"Window\"\nkind = \"web_app\"\nid = \"stack-local\"\n\
             title = \"Stack\"\nurl = \"http://localhost:9001/\"\nicon = \"stack.svg\"\n"
        ),
    )
}

#[test]
fn a_run_that_installs_the_app_is_its_setups_source() {
    let (_dir, catalog) = catalog_of(&[("stack", stack_app("source = \"container_images\"\n"))]);

    let list = serde_json::to_value(listing(&catalog, &Picks::default())).unwrap();

    assert!(list[0].get("sources").is_none(), "{}", list[0]);
    assert_eq!(list[0]["setups"][0]["id"], "local");
    assert_eq!(
        list[0]["setups"][0]["sources"],
        serde_json::json!([{ "kind": "container_images" }])
    );
    assert_eq!(
        list[0]["setups"][1]["sources"],
        serde_json::json!([{ "kind": "web_app" }])
    );
}

#[test]
fn an_install_whose_source_nothing_names_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("stack")).unwrap();
    fs::write(dir.path().join("stack/app.toml"), stack_app("")).unwrap();

    let err = Catalog::load(dir.path()).unwrap_err();

    let CatalogError::App { source, .. } = &err else {
        panic!("{err:?}");
    };
    assert!(matches!(**source, FolderError::NoSource(_)), "{source:?}");
}

#[test]
fn a_run_names_its_source_only_as_the_installs_only_source() {
    let configure = "source = \"container_images\"\nstage = \"configure\"\n";
    let beside = "source = \"container_images\"\n\
                  [[setup.local.action]]\nname = \"Engine\"\nkind = \"apt\"\npackages = [\"engine\"]\n";
    for rest in [configure, beside] {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("stack")).unwrap();
        fs::write(dir.path().join("stack/app.toml"), stack_app(rest)).unwrap();

        let err = Catalog::load(dir.path()).unwrap_err();

        let CatalogError::App { source, .. } = &err else {
            panic!("{err:?}");
        };
        assert!(matches!(**source, FolderError::StraySource(_)), "{rest}: {source:?}");
    }
}

#[test]
fn a_run_names_only_a_source_the_catalog_knows() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("stack")).unwrap();
    fs::write(dir.path().join("stack/app.toml"), stack_app("source = \"floppies\"\n")).unwrap();

    assert!(Catalog::load(dir.path()).is_err());
}

#[test]
fn a_configure_step_is_no_source() {
    let (_dir, catalog) = catalog_of(&[(
        "notes",
        app_in(
            "Notes",
            "office",
            &format!(
                "[[action]]\nname = \"Install it\"\nkind = \"apt\"\npackages = [\"notes\"]\n\
                 [[action]]\nname = \"Its theme\"\nstage = \"configure\"\nkind = \"download\"\n\
                 to = \"~/.config/kubaos/notes/theme\"\nversion = \"1\"\nunpack = \"file\"\n\
                 amd64 = {{ url = \"https://notes.example/theme\", sha256 = \"{}\" }}\n",
                "0".repeat(64)
            ),
        ),
    )]);

    let list = serde_json::to_value(listing(&catalog, &Picks::default())).unwrap();

    assert_eq!(list[0]["sources"], serde_json::json!([{ "kind": "archive" }]));
}

#[test]
fn penpots_local_stack_comes_from_pinned_images_and_its_web_setup_from_the_website() {
    let catalog = shipped();

    let list = serde_json::to_value(listing(&catalog, &Picks::default())).unwrap();

    let penpot = list.as_array().unwrap().iter().find(|a| a["id"] == "penpot").unwrap();
    assert_eq!(penpot["setups"][0]["id"], "local");
    assert_eq!(
        penpot["setups"][0]["sources"],
        serde_json::json!([{ "kind": "container_images" }])
    );
    assert_eq!(
        penpot["setups"][1]["sources"],
        serde_json::json!([{ "kind": "web_app" }])
    );
}

/// The licences of the repository itself, whose texts are `LICENSE-MIT` and `LICENSE-APACHE` at
/// its root: a file under one of them needs nothing in `LICENSES/`.
const OURS: [&str; 2] = ["MIT", "Apache-2.0"];

/// The beginnings of the SPDX identifiers a header comment names a licence by.
const SPDX_PREFIXES: [&str; 13] = [
    "MPL-",
    "GPL-",
    "LGPL-",
    "AGPL-",
    "CC-BY",
    "CC0-",
    "OFL-",
    "BSD-",
    "EPL-",
    "Apache-",
    "MIT",
    "ISC",
    "Unlicense",
];

/// The comment a file starts with: an SVG's or an XML file's first `<!-- … -->`, else its first
/// run of `#` or `//` lines, a shebang skipped.
fn header_comment(text: &str) -> String {
    let rest = text.trim_start();
    let rest = match rest.strip_prefix("<?xml") {
        Some(after) => after.split_once("?>").map_or("", |(_, r)| r).trim_start(),
        None => rest,
    };
    if let Some(after) = rest.strip_prefix("<!--") {
        return after.split_once("-->").map_or("", |(c, _)| c).to_owned();
    }
    rest.lines()
        .skip_while(|line| line.starts_with("#!"))
        .map_while(|line| {
            let line = line.trim_start();
            line.strip_prefix('#').or_else(|| line.strip_prefix("//"))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// The SPDX licence identifiers `comment` names.
fn licences_named(comment: &str) -> Vec<String> {
    let mut found: Vec<String> = comment
        .split(|c: char| !(c.is_ascii_alphanumeric() || matches!(c, '-' | '.' | '+')))
        .map(|word| word.trim_end_matches('.'))
        .filter(|word| SPDX_PREFIXES.iter().any(|prefix| word.starts_with(prefix)))
        .map(str::to_owned)
        .collect();
    found.sort();
    found.dedup();
    found
}

/// What is wrong with the licence notices of the apps' files under `root`: a picture that names
/// no licence in its header comment, or a licence a header names whose text isn't in
/// `LICENSES/`.
fn licence_problems(root: &Path) -> Vec<String> {
    let mut problems = Vec::new();
    let mut files: Vec<PathBuf> = Vec::new();
    for app in fs::read_dir(root.join("apps")).unwrap() {
        let app = app.unwrap().path();
        if app.is_dir() {
            files.extend(walk(&app));
        }
    }
    files.sort();
    for file in files {
        if file.file_name().is_some_and(|n| n == "app.toml") {
            continue;
        }
        let shown = file.strip_prefix(root).unwrap_or(&file).display().to_string();
        let picture = file
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| matches!(e, "svg" | "png" | "jpg" | "jpeg" | "webp" | "ico"));
        let Ok(text) = fs::read_to_string(&file) else {
            problems.push(format!(
                "{shown}: not text, so it can't name its licence; an icon is an .svg"
            ));
            continue;
        };
        let licences = licences_named(&header_comment(&text));
        if picture && licences.is_empty() {
            problems.push(format!("{shown}: its header comment names no licence"));
        }
        for licence in licences.iter().filter(|l| !OURS.contains(&l.as_str())) {
            if !root.join("LICENSES").join(format!("{licence}.txt")).is_file() {
                problems.push(format!(
                    "{shown}: {licence}, whose text isn't in LICENSES/{licence}.txt"
                ));
            }
        }
    }
    problems
}

fn walk(dir: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            found.extend(walk(&path));
        } else {
            found.push(path);
        }
    }
    found
}

#[test]
fn every_third_party_file_of_an_app_names_its_licence_and_the_repo_ships_its_text() {
    let problems = licence_problems(&repo_root());

    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

#[test]
fn the_licence_check_finds_a_picture_naming_no_licence_and_a_licence_without_its_text() {
    let root = tempfile::tempdir().unwrap();
    let app = root.path().join("apps/chat");
    fs::create_dir_all(&app).unwrap();
    fs::create_dir_all(root.path().join("LICENSES")).unwrap();
    fs::write(root.path().join("LICENSES/MPL-2.0.txt"), "").unwrap();
    fs::write(app.join("app.toml"), "name = \"Chat\"\n").unwrap();
    fs::write(app.join("bare.svg"), "<svg/>\n").unwrap();
    fs::write(app.join("ours.svg"), "<!-- KubaOS's, MIT OR Apache-2.0. -->\n<svg/>\n").unwrap();
    fs::write(
        app.join("theirs.svg"),
        "<!-- Chat's logo, MPL-2.0, unmodified. -->\n<svg/>\n",
    )
    .unwrap();
    fs::write(
        app.join("gpl.svg"),
        "<!-- From a theme, GPL-3.0-or-later. -->\n<svg/>\n",
    )
    .unwrap();
    fs::write(
        app.join("stack.yaml"),
        "# Chat's stack, CC-BY-4.0, as KubaOS runs it.\nservices: {}\n",
    )
    .unwrap();
    fs::write(app.join("up"), "#!/bin/sh\n# up: start it.\nexit 0\n").unwrap();

    let problems = licence_problems(root.path());

    assert_eq!(
        problems,
        [
            "apps/chat/bare.svg: its header comment names no licence",
            "apps/chat/gpl.svg: GPL-3.0-or-later, whose text isn't in LICENSES/GPL-3.0-or-later.txt",
            "apps/chat/stack.yaml: CC-BY-4.0, whose text isn't in LICENSES/CC-BY-4.0.txt",
        ]
    );
}
