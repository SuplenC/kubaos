//! The graphics apps' vendor repositories, as the update's migration puts them in place again.

use std::fs;
use std::path::Path;

use super::fake;
use super::key_sandbox;
use super::names;
use super::with_app;
use super::world;

#[test]
fn the_graphics_repos_migration_refreshes_only_the_repositories_kubaos_set_up() {
    let w = world();
    with_app(&w, "inkscape");
    let mut m = fake(&w, &[]);
    w.updater(&mut m).add_apps(&names(&["inkscape"]), None).unwrap();
    let dir = w.tmp.path().join("migration");
    let sources = dir.join("sources.list.d");
    fs::create_dir_all(&sources).unwrap();
    // Every key server answers.
    let sandbox = key_sandbox(&dir, "no-such-key-server");
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let migration = fs::read_dir(root.join("migrations/system"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .find(|p| p.to_string_lossy().ends_with("-graphics-repos.sh"))
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

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "{stderr}");
    let calls = sandbox.calls();
    let wrote = |file: &str| {
        calls
            .iter()
            .any(|c| c.starts_with("sudo install -D -m 0644 ") && c.ends_with(&format!(" {file}")))
    };
    let at = |name: &str| sources.join(name).display().to_string();
    assert!(
        wrote(&at("inkscape.sources")) && wrote("/etc/apt/keyrings/inkscape.gpg"),
        "{calls:#?}"
    );
    assert!(
        !wrote(&at("darktable.sources")),
        "no app set darktable's up: {calls:#?}"
    );
    let updates = calls.iter().filter(|c| *c == "sudo apt-get update").count();
    assert_eq!(updates, 1, "{calls:#?}");
}
