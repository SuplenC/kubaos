//! Spotify's repository and the OBS Project's PPA, as the update's migration puts them in place
//! again.

use std::fs;
use std::path::Path;
use std::path::PathBuf;

use super::World;
use super::fake;
use super::key_sandbox;
use super::names;
use super::put;
use super::with_app;
use super::world;

/// The media-repos migration run on `w`, its sources and keys going to a folder of the test's,
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
        .find(|p| p.to_string_lossy().ends_with("-media-repos.sh"))
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

/// The release's folders of Spotify and OBS Studio in the checkout's catalog, Spotify's scripts
/// writing their spotify.list in a folder of the test's rather than the machine's.
fn with_media_apps(w: &World) {
    let spotify = with_app(w, "spotify");
    with_app(w, "obs-studio");
    let list = w.tmp.path().join("apt-sources/spotify.list");
    fs::create_dir_all(list.parent().unwrap()).unwrap();
    let toml = spotify.join("app.toml");
    let text = fs::read_to_string(&toml).unwrap();
    fs::write(
        &toml,
        text.replace("/etc/apt/sources.list.d/spotify.list", &list.display().to_string()),
    )
    .unwrap();
}

#[test]
fn the_media_repos_migration_refreshes_spotifys_and_the_ppas_repositories_where_kubaos_set_them_up() {
    let w = world();
    with_media_apps(&w);
    let mut m = fake(&w, &[]);
    w.updater(&mut m)
        .add_apps(&names(&["spotify", "obs-studio"]), None)
        .unwrap();
    let trusted = w.tmp.path().join("trusted.gpg.d");
    let manifest =
        crate::apt::Manifest::read(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../system/packages.toml"))
            .unwrap()
            .unwrap();
    // The copy of its key a Spotify installed by hand left, which the key sandbox reads by its URL.
    let spotify = &manifest.repos()["spotify"];
    let copy = trusted.join(&spotify.global_keyrings[0]);
    put(&copy, &spotify.key_url);

    let (calls, sources) = migrate(&w, &trusted);

    let wrote = |file: &str| {
        calls
            .iter()
            .any(|c| c.starts_with("sudo install -D -m 0644 ") && c.ends_with(&format!(" {file}")))
    };
    for (source, key) in [
        ("spotify.sources", "/etc/apt/keyrings/spotify.gpg"),
        ("obs-studio.sources", "/etc/apt/keyrings/obs-studio.gpg"),
    ] {
        let at = sources.join(source).display().to_string();
        assert!(wrote(&at) && wrote(key), "{source}: {calls:#?}");
    }
    let removed = format!("sudo rm -f {}", copy.display());
    assert!(calls.contains(&removed), "{calls:#?}");
    let updates = calls.iter().filter(|c| *c == "sudo apt-get update").count();
    assert_eq!(updates, 1, "{calls:#?}");
}

#[test]
fn the_media_repos_migration_leaves_a_machine_without_kubaoss_spotify_or_obs_alone() {
    let w = world();
    let trusted = w.tmp.path().join("trusted.gpg.d");

    let (calls, _) = migrate(&w, &trusted);

    assert!(!calls.iter().any(|c| c.starts_with("sudo ")), "{calls:#?}");
}
