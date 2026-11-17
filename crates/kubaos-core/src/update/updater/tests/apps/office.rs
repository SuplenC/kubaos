//! ONLYOFFICE's vendor repository, as the update's migration puts it in place again.

use std::fs;
use std::path::Path;
use std::path::PathBuf;

use super::World;
use super::fake;
use super::key_sandbox;
use super::names;
use super::with_app;
use super::world;

/// The office-repos migration run on `w`, its sources and keys going to a folder of the test's;
/// what it ran, and what it wrote in that folder.
fn migrate(w: &World) -> (Vec<String>, PathBuf) {
    let dir = w.tmp.path().join("migration");
    let sources = dir.join("sources.list.d");
    fs::create_dir_all(&sources).unwrap();
    // Every key server answers.
    let sandbox = key_sandbox(&dir, "no-such-url");
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let migration = fs::read_dir(root.join("migrations/system"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .find(|p| p.to_string_lossy().ends_with("-office-repos.sh"))
        .unwrap();

    let out = sandbox
        .command("bash")
        .args(["-euo", "pipefail"])
        .arg(&migration)
        .env("KUBAOS_DIR", &root)
        .env("APT_SOURCES_D", &sources)
        .env("KUBAOS_APP_JOURNALS", w.sys.app_journals())
        .output()
        .unwrap();

    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    (sandbox.calls(), sources)
}

#[test]
fn the_office_repos_migration_refreshes_onlyoffices_repository_where_kubaos_set_it_up() {
    let w = world();
    with_app(&w, "onlyoffice");
    let mut m = fake(&w, &[]);
    w.updater(&mut m).add_apps(&names(&["onlyoffice"]), None).unwrap();

    let (calls, sources) = migrate(&w);

    let wrote = |file: &str| {
        calls
            .iter()
            .any(|c| c.starts_with("sudo install -D -m 0644 ") && c.ends_with(&format!(" {file}")))
    };
    let at = sources.join("onlyoffice.sources").display().to_string();
    assert!(wrote(&at) && wrote("/etc/apt/keyrings/onlyoffice.gpg"), "{calls:#?}");
    let updates = calls.iter().filter(|c| *c == "sudo apt-get update").count();
    assert_eq!(updates, 1, "{calls:#?}");
}

#[test]
fn the_office_repos_migration_leaves_a_machine_without_kubaoss_onlyoffice_alone() {
    let w = world();

    let (calls, _) = migrate(&w);

    assert!(!calls.iter().any(|c| c.starts_with("sudo ")), "{calls:#?}");
}
