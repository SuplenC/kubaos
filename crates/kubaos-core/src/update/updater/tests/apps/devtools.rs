//! The developer tools' vendor repositories, as the update's migration puts them in place again.

use std::fs;
use std::path::Path;

use super::fake;
use super::key_sandbox;
use super::names;
use super::with_app;
use super::world;

#[test]
fn the_devtools_repos_migration_refreshes_the_repositories_kubaos_set_up_and_never_fails() {
    let w = world();
    let apps = ["kubectl", "wine", "bruno"];
    for id in apps {
        with_app(&w, id);
    }
    let mut m = fake(&w, &[]);
    w.updater(&mut m).add_apps(&names(&apps), None).unwrap();
    let dir = w.tmp.path().join("migration");
    let sources = dir.join("sources.list.d");
    fs::create_dir_all(&sources).unwrap();
    // The key server Bruno's key comes from doesn't answer.
    let sandbox = key_sandbox(&dir, "E598552E09C999055FEFA7159FA6017ECABE0266");
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let migration = fs::read_dir(root.join("migrations/system"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .find(|p| p.to_string_lossy().ends_with("-devtools-repos.sh"))
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
    for (source, key) in [
        ("kubernetes.sources", "/etc/apt/keyrings/kubernetes.gpg"),
        ("winehq.sources", "/etc/apt/keyrings/winehq.gpg"),
    ] {
        assert!(wrote(&at(source)) && wrote(key), "{source}: {calls:#?}");
    }
    assert!(!wrote(&at("bruno.sources")), "{calls:#?}");
    assert!(stderr.contains("apt repository bruno not refreshed"), "{stderr}");
    for unset in ["dbeaver.sources", "lens.sources"] {
        assert!(!wrote(&at(unset)), "no app set {unset} up: {calls:#?}");
    }
    let updates = calls.iter().filter(|c| *c == "sudo apt-get update").count();
    assert_eq!(updates, 1, "one apt update for every repository: {calls:#?}");
}
