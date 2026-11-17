//! The AI apps' vendor repositories, as the update's migration puts them in place again.

use std::fs;
use std::path::Path;

use super::fake;
use super::key_sandbox;
use super::names;
use super::put;
use super::with_app;
use super::world;

#[test]
fn the_ai_repos_migration_refreshes_only_the_repositories_kubaos_set_up_and_never_fails() {
    let w = world();
    for id in ["claude-desktop", "devin-desktop"] {
        with_app(&w, id);
    }
    let mut m = fake(&w, &[]);
    w.updater(&mut m)
        .add_apps(&names(&["claude-desktop", "devin-desktop"]), None)
        .unwrap();
    let dir = w.tmp.path().join("migration");
    let sources = dir.join("sources.list.d");
    // Cursor installed by hand: its package wrote the source, which stays Anysphere's.
    let cursor =
        "Types: deb\nURIs: https://downloads.cursor.com/aptrepo\nSigned-By: /usr/share/keyrings/anysphere.gpg\n";
    put(&sources.join("cursor.sources"), cursor);
    let sandbox = key_sandbox(&dir, "windsurf");
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let migration = fs::read_dir(root.join("migrations/system"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .find(|p| p.to_string_lossy().ends_with("-ai-repos.sh"))
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
        wrote(&at("claude-desktop.sources")) && wrote("/etc/apt/keyrings/claude-desktop.gpg"),
        "{calls:#?}"
    );
    assert!(
        !calls.iter().any(|c| c.contains("anysphere") || c.contains("cursor")),
        "Cursor was installed by hand, so its source is Anysphere's: {calls:#?}"
    );
    assert_eq!(fs::read_to_string(sources.join("cursor.sources")).unwrap(), cursor);
    assert!(!wrote(&at("windsurf.sources")), "{calls:#?}");
    assert!(stderr.contains("apt repository windsurf not refreshed"), "{stderr}");
    let updates = calls.iter().filter(|c| *c == "sudo apt-get update").count();
    assert_eq!(updates, 1, "one apt update for every repository: {calls:#?}");
}
