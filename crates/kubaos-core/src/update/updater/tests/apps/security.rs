//! The repositories of the security and VPN apps, as the update's migration puts them in place
//! again.

use std::fs;
use std::path::Path;
use std::path::PathBuf;

use super::World;
use super::fake;
use super::host;
use super::key_sandbox;
use super::names;
use super::put;
use super::with_app;
use super::world;

/// Each app of a vendor's repository, with the repository and the keyring its source names.
const REPOS: [(&str, &str, &str); 6] = [
    (
        "1password",
        "1password",
        "/usr/share/keyrings/1password-archive-keyring.gpg",
    ),
    ("keepassxc", "keepassxc", "/etc/apt/keyrings/keepassxc.gpg"),
    ("cryptomator", "cryptomator", "/etc/apt/keyrings/cryptomator.gpg"),
    ("mullvad-vpn", "mullvad", "/etc/apt/keyrings/mullvad.gpg"),
    (
        "proton-vpn",
        "protonvpn-stable",
        "/usr/share/keyrings/protonvpn-stable-archive-keyring.gpg",
    ),
    ("nordvpn", "nordvpn", "/etc/apt/keyrings/nordvpn.gpg"),
];

/// The security-repos migration run on `w`, its sources and keys going to a folder of the test's,
/// with `trusted` as `/etc/apt/trusted.gpg.d`; what it ran, and the sources folder.
fn migrate(w: &World, trusted: &Path) -> (Vec<String>, PathBuf) {
    let dir = w.tmp.path().join("migration");
    let sources = dir.join("sources.list.d");
    fs::create_dir_all(&sources).unwrap();
    // Every key server answers.
    let sandbox = key_sandbox(&dir, "no-such-url");
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let migration = fs::read_dir(root.join("migrations/system"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .find(|p| p.to_string_lossy().ends_with("-security-repos.sh"))
        .unwrap();

    let out = sandbox
        .command("bash")
        .args(["-euo", "pipefail"])
        .arg(&migration)
        .env("KUBAOS_DIR", &root)
        .env("APT_SOURCES_D", &sources)
        .env("APT_TRUSTED_D", trusted)
        .env("KUBAOS_APP_JOURNALS", w.sys.app_journals())
        .output()
        .unwrap();

    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    (sandbox.calls(), sources)
}

#[test]
fn the_security_repos_migration_refreshes_each_vpn_and_password_managers_repository_where_kubaos_set_it_up() {
    let w = world();
    let ids: Vec<&str> = REPOS.iter().map(|(id, ..)| *id).collect();
    for id in &ids {
        with_app(&w, id);
    }
    let mut m = fake(&w, &[]);
    // No group of 1Password's helpers yet, whatever the machine running the test has.
    host(&m).sandbox().stub("getent", "exit 2");
    w.updater(&mut m).add_apps(&names(&ids), None).unwrap();
    let trusted = w.tmp.path().join("trusted.gpg.d");
    let manifest =
        crate::apt::Manifest::read(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../system/packages.toml"))
            .unwrap()
            .unwrap();
    // The copies of NordVPN's key its own setups left, which the key sandbox reads by their URL.
    let nordvpn = &manifest.repos()["nordvpn"];
    let copies: Vec<PathBuf> = nordvpn.global_keyrings.iter().map(|k| trusted.join(k)).collect();
    for copy in &copies {
        put(copy, &nordvpn.key_url);
    }

    let (calls, sources) = migrate(&w, &trusted);

    let wrote = |file: &str| {
        calls
            .iter()
            .any(|c| c.starts_with("sudo install -D -m 0644 ") && c.ends_with(&format!(" {file}")))
    };
    for (_, repo, key) in REPOS {
        let at = sources.join(format!("{repo}.sources")).display().to_string();
        assert!(wrote(&at) && wrote(key), "{repo}: {calls:#?}");
    }
    for copy in &copies {
        let removed = format!("sudo rm -f {}", copy.display());
        assert!(calls.contains(&removed), "{calls:#?}");
    }
    let updates = calls.iter().filter(|c| *c == "sudo apt-get update").count();
    assert_eq!(updates, 1, "{calls:#?}");
}

#[test]
fn the_security_repos_migration_leaves_a_machine_without_kubaoss_apps_alone() {
    let w = world();
    let trusted = w.tmp.path().join("trusted.gpg.d");

    let (calls, _) = migrate(&w, &trusted);

    assert!(!calls.iter().any(|c| c.starts_with("sudo ")), "{calls:#?}");
}
