//! Tailscale's, Syncthing's, Nextcloud's and Dropbox's vendor repositories, as the update's
//! migration puts them in place again.

use std::fs;
use std::path::Path;
use std::path::PathBuf;

use super::World;
use super::fake;
use super::key_sandbox;
use super::names;
use super::with_app;
use super::world;

/// The system-repos migration run on `w`, its sources and keys going to a folder of the test's;
/// what it ran, and the sources folder.
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
        .find(|p| p.to_string_lossy().ends_with("-system-repos.sh"))
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

/// The release's folders of the four apps in the checkout's catalog, Dropbox's scripts writing
/// their dropbox-repo in a folder of the test's rather than the machine's.
fn with_repo_apps(w: &World) {
    with_app(w, "tailscale");
    with_app(w, "syncthing");
    with_app(w, "nextcloud");
    let dropbox = with_app(w, "dropbox");
    let file = w.tmp.path().join("etc-default/dropbox-repo");
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    let toml = dropbox.join("app.toml");
    let text = fs::read_to_string(&toml).unwrap();
    fs::write(
        &toml,
        text.replace("/etc/default/dropbox-repo", &file.display().to_string()),
    )
    .unwrap();
}

#[test]
fn the_system_repos_migration_refreshes_the_four_repositories_where_kubaos_set_them_up() {
    let w = world();
    with_repo_apps(&w);
    let mut m = fake(&w, &[]);
    w.updater(&mut m)
        .add_apps(&names(&["tailscale", "syncthing", "nextcloud", "dropbox"]), None)
        .unwrap();

    let (calls, sources) = migrate(&w);

    let wrote = |file: &str| {
        calls
            .iter()
            .any(|c| c.starts_with("sudo install -D -m 0644 ") && c.ends_with(&format!(" {file}")))
    };
    for repo in ["tailscale", "syncthing", "nextcloud", "dropbox"] {
        let at = sources.join(format!("{repo}.sources")).display().to_string();
        assert!(
            wrote(&at) && wrote(&format!("/etc/apt/keyrings/{repo}.gpg")),
            "{repo}: {calls:#?}"
        );
    }
    let updates = calls.iter().filter(|c| *c == "sudo apt-get update").count();
    assert_eq!(updates, 1, "{calls:#?}");
}

#[test]
fn the_system_repos_migration_leaves_a_machine_without_kubaoss_apps_of_them_alone() {
    let w = world();

    let (calls, _) = migrate(&w);

    assert!(!calls.iter().any(|c| c.starts_with("sudo ")), "{calls:#?}");
}
