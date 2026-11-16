use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::path::PathBuf;
use std::process::Command;

use super::Added;
use super::App;
use super::AppState;
use super::Catalog;
use super::CatalogError;
use super::ManifestError;
use super::Picks;
use super::Source;
use super::Status;
use super::catalog::CATALOG;
use super::plan::Facts;
use super::plan::installs;
use super::plan::removals;
use crate::apt::Dpkg;
use crate::apt::Manifest;
use crate::apt::manifest::MANIFEST;

const SHA: &str = "418c7825f73107b7b5ba3babae5e3b485801dc22d9dc0d518af632f7906cbdd1";

const AGENT: &str = r#"
# An app from a vendor repository.
name = "Agent"
summary = "A coding agent"

[apt]
repo = "vendor"
packages = ["agent", "agent-data"]
"#;

const EDITOR: &str = r#"
name = "Editor"
summary = "An editor"

[nix]
packages = ["editor", "python3Packages.editor-lsp"]
"#;

fn desk(self_updates: bool) -> String {
    format!(
        r#"
name = "Desk"
summary = "A desktop app"

[deb]
package = "desk"
version = "1.2.0"
self_updates = {self_updates}

[deb.amd64]
url = "https://example.com/desk-1.2.0-amd64.deb"
sha256 = "{SHA}"
"#
    )
}

fn catalog(self_updates: bool) -> Catalog {
    Catalog::new(vec![
        App::parse("agent", AGENT).unwrap(),
        App::parse("desk", &desk(self_updates)).unwrap(),
        App::parse("editor", EDITOR).unwrap(),
    ])
}

fn picks(ids: &[&str]) -> Picks {
    let mut picks = Picks::default();
    for id in ids {
        picks.add(id);
    }
    picks
}

/// dpkg with `installed`, each `name` or `name=version`.
fn dpkg(installed: &[&str]) -> Dpkg {
    let mut lines = String::new();
    for p in installed {
        let (name, version) = p.split_once('=').unwrap_or((p, "1.0"));
        let _ = writeln!(lines, "installed install {name} {version}");
    }
    Dpkg::parse(&lines)
}

fn nix(names: &[&str]) -> BTreeSet<String> {
    names.iter().map(|n| (*n).to_owned()).collect()
}

fn ids(apps: &[&App]) -> Vec<String> {
    apps.iter().map(|a| a.id.clone()).collect()
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn a_manifest_names_its_app_and_its_one_source() {
    let app = App::parse("desk", &desk(true)).unwrap();

    assert_eq!((app.name.as_str(), app.source.kind()), ("Desk", "deb"));
    let Source::Deb(deb) = &app.source else {
        panic!("not a deb: {:?}", app.source);
    };
    assert_eq!((deb.package.as_str(), deb.version.as_str()), ("desk", "1.2.0"));
    assert!(deb.self_updates);
    assert_eq!(deb.download("amd64").unwrap().sha256, SHA);
    assert!(deb.download("arm64").is_none());
    assert_eq!(app.icon, None);
}

#[test]
fn a_manifest_may_name_its_icon_from_the_icon_theme() {
    let text = AGENT.replace(
        "summary = \"A coding agent\"",
        "summary = \"A coding agent\"\nicon = \"agent-cli\"",
    );

    let app = App::parse("agent", &text).unwrap();

    assert_eq!(app.icon.as_deref(), Some("agent-cli"));
}

#[test]
fn every_picked_app_the_machine_lacks_is_installed_and_nothing_else() {
    let catalog = catalog(false);
    let picks = picks(&["agent", "desk", "editor"]);
    let (dpkg, nix) = (dpkg(&["agent", "desk=1.2.0"]), nix(&[]));
    let facts = Facts {
        catalog: &catalog,
        picks: &picks,
        arch: "amd64",
        dpkg: &dpkg,
        nix: &nix,
    };

    assert_eq!(ids(&installs(&facts)), ["agent", "editor"]);
    assert_eq!(facts.state(catalog.get("agent").unwrap()), AppState::Missing);
}

#[test]
fn once_everything_picked_is_there_a_second_run_installs_nothing() {
    let catalog = catalog(false);
    let picks = picks(&["agent", "desk", "editor"]);
    let dpkg = dpkg(&["agent", "agent-data", "desk=1.2.0"]);
    let nix = nix(&["editor", "python3Packages.editor-lsp", "ripgrep"]);
    let facts = Facts {
        catalog: &catalog,
        picks: &picks,
        arch: "amd64",
        dpkg: &dpkg,
        nix: &nix,
    };

    assert!(installs(&facts).is_empty());
}

#[test]
fn an_app_on_the_machine_but_not_picked_is_left_alone() {
    let catalog = catalog(false);
    let picks = picks(&[]);
    let (dpkg, nix) = (dpkg(&["agent", "agent-data"]), nix(&[]));
    let facts = Facts {
        catalog: &catalog,
        picks: &picks,
        arch: "amd64",
        dpkg: &dpkg,
        nix: &nix,
    };

    assert!(installs(&facts).is_empty());
    let status = Status::new(&facts);
    let agent = &status.apps[0];
    assert_eq!(
        (agent.id.as_str(), agent.picked, agent.state),
        ("agent", false, AppState::Installed)
    );
}

#[test]
fn a_pinned_deb_at_another_version_is_put_back_to_the_pin_unless_it_updates_itself() {
    let picks = picks(&["desk"]);
    let (dpkg, nix) = (dpkg(&["desk=1.1.0"]), nix(&[]));
    let pinned = catalog(false);
    let updating = catalog(true);
    let facts = |catalog| Facts {
        catalog,
        picks: &picks,
        arch: "amd64",
        dpkg: &dpkg,
        nix: &nix,
    };

    assert_eq!(ids(&installs(&facts(&pinned))), ["desk"]);
    assert_eq!(facts(&pinned).state(pinned.get("desk").unwrap()), AppState::Outdated);
    assert!(installs(&facts(&updating)).is_empty());
}

#[test]
fn a_deb_with_no_download_for_the_architecture_is_unavailable_and_never_installed() {
    let catalog = catalog(false);
    let picks = picks(&["desk"]);
    let (dpkg, nix) = (dpkg(&[]), nix(&[]));
    let facts = Facts {
        catalog: &catalog,
        picks: &picks,
        arch: "arm64",
        dpkg: &dpkg,
        nix: &nix,
    };

    assert_eq!(facts.state(catalog.get("desk").unwrap()), AppState::Unavailable);
    assert!(installs(&facts).is_empty());
}

#[test]
fn an_apt_app_for_another_architecture_is_unavailable_unless_it_is_there() {
    let browser = App::parse("browser", &format!("{AGENT}arch = \"amd64\"\n")).unwrap();
    let catalog = Catalog::new(vec![browser]);
    let picks = picks(&["browser"]);
    let nix = nix(&[]);
    let facts = |arch, dpkg| Facts {
        catalog: &catalog,
        picks: &picks,
        arch,
        dpkg,
        nix: &nix,
    };
    let (none, both) = (dpkg(&[]), dpkg(&["agent", "agent-data"]));
    let browser = catalog.get("browser").unwrap();

    assert_eq!(facts("arm64", &none).state(browser), AppState::Unavailable);
    assert!(installs(&facts("arm64", &none)).is_empty());
    assert_eq!(facts("amd64", &none).state(browser), AppState::Missing);
    assert_eq!(facts("arm64", &both).state(browser), AppState::Installed);
    assert!(browser.source.available("amd64") && !browser.source.available("arm64"));
}

#[test]
fn an_app_names_the_system_hooks_it_needs_beyond_its_packages() {
    let app = App::parse("agent", &format!("system_hooks = \"agent_socket\"\n{AGENT}")).unwrap();

    assert_eq!(app.system_hooks.as_deref(), Some("agent_socket"));
    assert_eq!(App::parse("agent", AGENT).unwrap().system_hooks, None);
}

#[test]
fn a_nixpkgs_app_has_no_system_hooks_as_nothing_would_run_them() {
    let error = App::parse("editor", &format!("system_hooks = \"editor_socket\"\n{EDITOR}")).unwrap_err();

    assert!(matches!(error, ManifestError::SystemHooksOnNix), "{error}");
}

#[test]
fn a_removal_takes_only_the_named_apps_that_are_there() {
    let catalog = catalog(false);
    let picks = picks(&["agent", "desk"]);
    let (dpkg, nix) = (dpkg(&["agent", "agent-data", "desk=0.9"]), nix(&[]));
    let facts = Facts {
        catalog: &catalog,
        picks: &picks,
        arch: "amd64",
        dpkg: &dpkg,
        nix: &nix,
    };

    let named = ["desk".to_owned(), "editor".to_owned(), "nope".to_owned()];

    assert_eq!(ids(&removals(&facts, &named)), ["desk"]);
}

#[test]
fn the_status_lists_every_app_and_the_picks_the_catalog_lacks() {
    let catalog = catalog(false);
    let picks = picks(&["agent", "gone"]);
    let (dpkg, nix) = (dpkg(&["agent=2.1", "agent-data"]), nix(&[]));
    let facts = Facts {
        catalog: &catalog,
        picks: &picks,
        arch: "amd64",
        dpkg: &dpkg,
        nix: &nix,
    };

    let status = Status::new(&facts);

    let rows: Vec<(&str, &str, bool, Option<&str>)> = status
        .apps
        .iter()
        .map(|a| (a.id.as_str(), a.state.word(), a.picked, a.version.as_deref()))
        .collect();
    assert_eq!(
        rows,
        [
            ("agent", "installed", true, Some("2.1")),
            ("desk", "available", false, None),
            ("editor", "available", false, None),
        ]
    );
    assert_eq!(status.unknown, ["gone"]);
    let json = serde_json::to_value(&status).unwrap();
    assert_eq!(json["apps"][0]["source"], "apt");
    assert_eq!(json["apps"][0]["state"], "installed");
}

#[test]
fn a_manifest_that_cant_be_installed_safely_is_refused() {
    let deb = desk(false);
    let cases: Vec<(&str, String)> = vec![
        ("no source", "name = \"X\"\nsummary = \"Y\"\n".to_owned()),
        ("two sources", format!("{AGENT}\n[nix]\npackages = [\"x\"]\n")),
        ("unknown key", format!("homepage = \"x\"\n{AGENT}")),
        ("unknown key of a source", format!("{AGENT}\nicon = \"x\"\n")),
        (
            "an icon that is a path",
            format!("icon = \"/usr/share/icons/x.png\"\n{AGENT}"),
        ),
        ("an empty icon", format!("icon = \"\"\n{AGENT}")),
        ("empty name", AGENT.replace("\"Agent\"", "\"\"")),
        ("no packages", AGENT.replace("[\"agent\", \"agent-data\"]", "[]")),
        ("bad apt name", AGENT.replace("\"agent-data\"", "\"Agent Data\"")),
        (
            "an option as a package",
            AGENT.replace("\"agent-data\"", "\"--force-yes\""),
        ),
        ("bad nix name", EDITOR.replace("\"editor\"", "\"../editor\"")),
        ("http", deb.replace("https://", "http://")),
        ("url with a space", deb.replace("desk-1.2.0", "desk 1.2.0")),
        ("short sha", deb.replace(SHA, "abc")),
        ("uppercase sha", deb.replace(SHA, &SHA.to_uppercase())),
        ("bad version", deb.replace("\"1.2.0\"", "\"latest version\"")),
        ("no download", deb.split("[deb.amd64]").next().unwrap().to_owned()),
        ("an escape", AGENT.replace("A coding agent", "A \\u0063oding agent")),
        (
            "an architecture dpkg doesn't name",
            format!("{AGENT}arch = \"x86_64\"\n"),
        ),
        (
            "system hooks that aren't a function name",
            format!("system_hooks = \"x; reboot\"\n{AGENT}"),
        ),
        (
            "system hooks that are a path",
            format!("system_hooks = \"../lib\"\n{AGENT}"),
        ),
        ("uppercase system hooks", format!("system_hooks = \"Agent\"\n{AGENT}")),
        ("the old setup key", format!("setup = \"agent_socket\"\n{AGENT}")),
        (
            "a multi-line string",
            AGENT.replace("\"A coding agent\"", "\"\"\"A coding agent\"\"\""),
        ),
    ];
    for (what, text) in cases {
        assert!(App::parse("x", &text).is_err(), "{what} was accepted:\n{text}");
    }
}

#[test]
fn an_id_is_never_a_path_or_an_option() {
    for bad in [
        "",
        "../etc",
        "a/b",
        "-x",
        "Claude",
        "a b",
        "a.toml",
        "a_b",
        &"a".repeat(65),
    ] {
        assert!(
            matches!(App::parse(bad, AGENT), Err(ManifestError::InvalidId(_))),
            "{bad:?}"
        );
    }
    for good in ["claude-code", "t3-code", "7zip"] {
        assert!(App::parse(good, AGENT).is_ok(), "{good}");
    }
}

#[test]
fn a_repo_is_a_name_never_an_option_or_a_path() {
    for bad in ["--dry-run", "-n", "../vendor", "a b", "Vendor", "", &"a".repeat(65)] {
        let text = AGENT.replace("repo = \"vendor\"", &format!("repo = \"{bad}\""));
        assert!(
            matches!(App::parse("agent", &text), Err(ManifestError::InvalidRepo { .. })),
            "{bad:?}"
        );
    }
    for good in ["vendor", "google-chrome", "cppiber-hyprland"] {
        let text = AGENT.replace("repo = \"vendor\"", &format!("repo = \"{good}\""));
        assert!(App::parse("agent", &text).is_ok(), "{good}");
    }
}

#[test]
fn an_apt_app_from_a_repo_the_package_manifest_lacks_is_refused() {
    let manifest =
        Manifest::parse("[repo.other]\nkey_url = \"https://x\"\nfingerprint = \"AB\"\nkeyring = \"/k\"\n").unwrap();

    let checked = catalog(false).check_repos(&manifest);

    assert!(matches!(checked, Err(CatalogError::UnknownRepo { id, repo }) if id == "agent" && repo == "vendor"));
}

#[test]
fn the_catalog_loads_every_manifest_in_its_folder_and_nothing_else() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("agent.toml"), AGENT).unwrap();
    std::fs::write(tmp.path().join("editor.toml"), EDITOR).unwrap();
    std::fs::write(tmp.path().join("README.md"), "not an app").unwrap();

    let catalog = Catalog::load(tmp.path()).unwrap();

    let ids: Vec<&str> = catalog.apps().iter().map(|a| a.id.as_str()).collect();
    assert_eq!(ids, ["agent", "editor"]);
    assert_eq!(Catalog::load(&tmp.path().join("missing")).unwrap(), Catalog::default());
}

#[test]
fn the_picks_round_trip_sorted_with_their_header() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("kubaos/apps.toml");
    let mut picks = Picks::read(&path).unwrap();
    picks.add("t3-code");
    picks.add("claude-code");

    picks.write(&path).unwrap();

    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.starts_with("# The apps of KubaOS's catalog"));
    assert!(text.ends_with("apps = [\"claude-code\", \"t3-code\"]\n"), "{text}");
    assert_eq!(Picks::read(&path).unwrap(), picks);
}

#[test]
fn the_picks_are_only_a_list_of_apps() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("apps.toml");
    std::fs::write(&path, "apps = [\"editor\"]\n\n[nix_added]\neditor = [\"editor\"]\n").unwrap();

    assert!(Picks::read(&path).is_err());
}

#[test]
fn the_record_of_added_packages_round_trips_with_its_header() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("kubaos/apps-added.toml");
    let mut added = Added::read(&path).unwrap();
    added.record("studio", ["studio", "editor"]);
    added.record("empty", Vec::<String>::new());

    added.write(&path).unwrap();

    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.starts_with("# The nixpkgs packages each app"), "{text}");
    assert!(!text.contains("empty"), "{text}");
    let back = Added::read(&path).unwrap();
    assert_eq!(back, added);
    assert_eq!(back.of("studio"), ["editor", "studio"]);
}

#[test]
fn every_shipped_manifest_validates_and_its_repo_has_a_sources_file() {
    let root = repo_root();
    let manifest = Manifest::read(&root.join(MANIFEST)).unwrap().unwrap();

    let catalog = Catalog::load(&root.join(CATALOG)).unwrap();

    catalog.check_repos(&manifest).unwrap();
    for id in ["bitwarden", "claude-code", "docker", "google-chrome", "t3-code"] {
        assert!(catalog.get(id).is_some(), "the catalog has no {id}");
    }
    let lib = std::fs::read_to_string(root.join("system/lib.sh")).unwrap();
    for app in catalog.apps() {
        if let Source::Apt { repo: Some(repo), .. } = &app.source {
            let sources = root.join(format!("system/files/apt/sources.list.d/{repo}.sources"));
            assert!(sources.is_file(), "{}: no {}", app.id, sources.display());
        }
        if let Some(hooks) = &app.system_hooks {
            for verb in ["install", "remove"] {
                let function = format!("\n{verb}_{hooks}() {{\n");
                assert!(
                    lib.contains(&function),
                    "{}: system/lib.sh has no {verb}_{hooks}",
                    app.id
                );
            }
        }
    }
}

/// An app as `manifest_dump` in `system/manifest.sh` prints it: one `<table><TAB><key><TAB>
/// <value>` line per value, an array's items one line each.
fn rust_dump(app: &App) -> BTreeSet<String> {
    let mut lines = BTreeSet::new();
    let mut add = |table: &str, key: &str, value: &str| lines.insert(format!("{table}\t{key}\t{value}"));
    add("", "name", &app.name);
    add("", "summary", &app.summary);
    if let Some(icon) = &app.icon {
        add("", "icon", icon);
    }
    if let Some(hooks) = &app.system_hooks {
        add("", "system_hooks", hooks);
    }
    match &app.source {
        Source::Apt { repo, arch, packages } => {
            if let Some(repo) = repo {
                add("apt", "repo", repo);
            }
            if let Some(arch) = arch {
                add("apt", "arch", arch);
            }
            for p in packages {
                add("apt", "packages", p);
            }
        }
        Source::Nix { packages } => {
            for p in packages {
                add("nix", "packages", p);
            }
        }
        Source::Deb(deb) => {
            add("deb", "package", &deb.package);
            add("deb", "version", &deb.version);
            if deb.self_updates {
                add("deb", "self_updates", "true");
            }
            for (arch, download) in &deb.downloads {
                add(&format!("deb.{arch}"), "url", &download.url);
                add(&format!("deb.{arch}"), "sha256", &download.sha256);
            }
        }
    }
    lines
}

fn shell_dump(path: &std::path::Path) -> BTreeSet<String> {
    let out = Command::new("bash")
        .args(["-c", "source \"$1\"; manifest_dump \"$2\"", "bash"])
        .arg(repo_root().join("system/manifest.sh"))
        .arg(path)
        .output()
        .expect("run bash");
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    String::from_utf8(out.stdout)
        .unwrap()
        .lines()
        .filter(|line| !line.starts_with("deb\tself_updates\tfalse"))
        .map(str::to_owned)
        .collect()
}

#[test]
fn the_root_helper_reads_every_shipped_manifest_as_the_engine_does() {
    let dir = repo_root().join(CATALOG);
    let catalog = Catalog::load(&dir).unwrap();

    for app in catalog.apps() {
        let path = dir.join(format!("{}.toml", app.id));
        assert_eq!(shell_dump(&path), rust_dump(app), "{}", app.id);
    }
}
