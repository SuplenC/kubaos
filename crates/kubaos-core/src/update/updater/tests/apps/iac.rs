//! The repositories of the infrastructure-as-code tools, as the update's migration puts them in
//! place again.

use std::fs;
use std::path::Path;

use super::fake;
use super::key_sandbox;
use super::names;
use super::with_app;
use super::world;

#[test]
fn the_iac_repos_migration_refreshes_each_repository_kubaos_set_up_once_and_never_fails() {
    let w = world();
    // Terraform and Vault both set HashiCorp's repository up.
    let apps = ["terraform", "vault", "trivy"];
    for id in apps {
        with_app(&w, id);
    }
    let mut m = fake(&w, &[]);
    w.updater(&mut m).add_apps(&names(&apps), None).unwrap();
    let dir = w.tmp.path().join("migration");
    let sources = dir.join("sources.list.d");
    fs::create_dir_all(&sources).unwrap();
    // The server Trivy's key comes from doesn't answer.
    let sandbox = key_sandbox(&dir, "aquasecurity.github.io");
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let migration = fs::read_dir(root.join("migrations/system"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .find(|p| p.to_string_lossy().ends_with("-iac-repos.sh"))
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
            .filter(|c| c.starts_with("sudo install -D -m 0644 ") && c.ends_with(&format!(" {file}")))
            .count()
    };
    let at = |name: &str| sources.join(name).display().to_string();
    assert_eq!(wrote(&at("hashicorp.sources")), 1, "once for both its apps: {calls:#?}");
    assert_eq!(wrote("/etc/apt/keyrings/hashicorp.gpg"), 1, "{calls:#?}");
    assert_eq!(wrote(&at("trivy.sources")), 0, "{calls:#?}");
    assert!(stderr.contains("apt repository trivy not refreshed"), "{stderr}");
    assert_eq!(
        wrote(&at("opentofu.sources")),
        0,
        "no app set OpenTofu's up: {calls:#?}"
    );
    let updates = calls.iter().filter(|c| *c == "sudo apt-get update").count();
    assert_eq!(updates, 1, "one apt update for every repository: {calls:#?}");
}
