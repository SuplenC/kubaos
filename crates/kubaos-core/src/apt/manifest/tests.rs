use std::collections::BTreeSet;
use std::path::PathBuf;
use std::process::Command;

use super::Inactive;
use super::MANIFEST;
use super::Manifest;
use super::ManifestError;
use super::is_package_name;
use crate::apt::choices::Choices;
use crate::apt::choices::Switch;

const SMALL: &str = r#"
[repo.docker]
key_url = "https://download.docker.com/linux/ubuntu/gpg"
fingerprint = "9DC858229FC7DD38854AE2D88D81803C0EBFCD88"
keyring = "/etc/apt/keyrings/docker.gpg"

[group.tools]
summary = "Tools"
packages = ["curl", "git"]

# A comment between groups.
[group.docker]
summary = "Docker"
repo = "docker"
optional = "on"
when_on = ["system/40-apps.sh"]
packages = [
  "docker-ce",  # the engine
  "containerd.io",
]

[group.chrome]
summary = "Chrome"
arch = "amd64"
optional = "off"
packages = ["google-chrome-stable"]
"#;

/// The repo's checkout, from this crate.
fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn groups_keep_the_files_order_and_their_keys() {
    let manifest = Manifest::parse(SMALL).unwrap();

    let names: Vec<&str> = manifest.groups().iter().map(|g| g.name.as_str()).collect();
    assert_eq!(names, ["tools", "docker", "chrome"]);
    let docker = manifest.group("docker").unwrap();
    assert_eq!(docker.packages, ["docker-ce", "containerd.io"]);
    assert_eq!(docker.repo.as_deref(), Some("docker"));
    assert_eq!(docker.optional, Some(Switch::On));
    assert_eq!(docker.when_on, ["system/40-apps.sh"]);
    assert_eq!(manifest.repos()["docker"].keyring, "/etc/apt/keyrings/docker.gpg");
    assert!(manifest.lists("git") && !manifest.lists("vim"));
}

#[test]
fn a_group_is_inactive_on_another_architecture_or_when_turned_off() {
    let manifest = Manifest::parse(SMALL).unwrap();
    let mut choices = Choices::default();

    assert_eq!(manifest.group("tools").unwrap().inactive("arm64", &choices), None);
    assert_eq!(manifest.group("docker").unwrap().inactive("arm64", &choices), None);
    assert_eq!(
        manifest.group("chrome").unwrap().inactive("arm64", &choices),
        Some(Inactive::Arch("amd64".into()))
    );
    assert_eq!(
        manifest.group("chrome").unwrap().inactive("amd64", &choices),
        Some(Inactive::Off)
    );

    choices.set("docker", Switch::Off);
    choices.set("chrome", Switch::On);
    choices.set("tools", Switch::Off);

    assert_eq!(
        manifest.group("docker").unwrap().inactive("amd64", &choices),
        Some(Inactive::Off)
    );
    assert_eq!(manifest.group("chrome").unwrap().inactive("amd64", &choices), None);
    assert_eq!(manifest.group("tools").unwrap().inactive("amd64", &choices), None);
}

#[test]
fn a_group_naming_a_repo_the_manifest_lacks_is_refused() {
    let text = "[group.x]\nsummary = \"x\"\nrepo = \"nowhere\"\npackages = [\"curl\"]\n";

    let err = Manifest::parse(text).unwrap_err();

    assert!(matches!(err, ManifestError::UnknownRepo { ref repo, .. } if repo == "nowhere"));
}

#[test]
fn a_name_that_cant_be_a_package_is_refused() {
    let text = "[group.x]\nsummary = \"x\"\npackages = [\"curl; rm -rf /\"]\n";

    let err = Manifest::parse(text).unwrap_err();

    assert!(matches!(err, ManifestError::InvalidPackage { .. }));
}

#[test]
fn an_escape_the_scripts_cant_read_is_refused() {
    let text = "[group.x]\nsummary = \"a \\\"quoted\\\" word\"\npackages = [\"curl\"]\n";

    let err = Manifest::parse(text).unwrap_err();

    assert!(matches!(err, ManifestError::Escaped { .. }));
}

#[test]
fn a_script_on_a_group_that_cant_be_turned_off_is_refused() {
    let text = "[group.x]\nsummary = \"x\"\nwhen_on = [\"system/x.sh\"]\npackages = [\"curl\"]\n";

    let err = Manifest::parse(text).unwrap_err();

    assert!(matches!(err, ManifestError::ScriptNotOptional { .. }));
}

#[test]
fn an_unknown_key_is_refused() {
    let text = "[group.x]\nsummary = \"x\"\npakages = [\"curl\"]\npackages = []\n";

    assert!(matches!(Manifest::parse(text), Err(ManifestError::Parse(_))));
}

#[test]
fn package_names_follow_debian_policy() {
    for good in [
        "7zip",
        "libspa-0.2-bluetooth",
        "gstreamer1.0-libav",
        "g++",
        "containerd.io",
    ] {
        assert!(is_package_name(good), "{good}");
    }
    for bad in ["", "a", "-x", "Curl", "a b", "foo_bar", "x;y"] {
        assert!(!is_package_name(bad), "{bad}");
    }
}

#[test]
fn the_release_manifest_parses_and_each_repo_has_its_sources_file() {
    let root = repo_root();

    let manifest = Manifest::read(&root.join(MANIFEST)).unwrap().unwrap();

    assert!(
        manifest
            .group("files")
            .unwrap()
            .packages
            .contains(&"nautilus".to_owned())
    );
    for name in manifest.repos().keys() {
        let sources = root.join(format!("system/files/apt/sources.list.d/{name}.sources"));
        assert!(sources.is_file(), "no {}", sources.display());
    }
    for group in manifest.groups() {
        for script in group.when_on.first().into_iter().chain(group.when_off.first()) {
            assert!(root.join(script).is_file(), "group {}: no {script}", group.name);
        }
    }
}

/// The manifest as `manifest_dump` in `system/manifest.sh` prints it: one
/// `<table><TAB><key><TAB><value>` line per value, an array's items one line each.
fn rust_dump(manifest: &Manifest) -> BTreeSet<String> {
    let mut lines = BTreeSet::new();
    for (name, repo) in manifest.repos() {
        for (key, value) in [
            ("key_url", &repo.key_url),
            ("fingerprint", &repo.fingerprint),
            ("keyring", &repo.keyring),
        ] {
            lines.insert(format!("repo.{name}\t{key}\t{value}"));
        }
    }
    for group in manifest.groups() {
        let table = format!("group.{}", group.name);
        let mut add = |key: &str, value: &str| lines.insert(format!("{table}\t{key}\t{value}"));
        add("summary", &group.summary);
        if let Some(arch) = &group.arch {
            add("arch", arch);
        }
        if let Some(repo) = &group.repo {
            add("repo", repo);
        }
        if group.hold {
            add("hold", "true");
        }
        if group.if_available {
            add("if_available", "true");
        }
        if let Some(optional) = group.optional {
            add("optional", &optional.to_string());
        }
        for (key, values) in [
            ("packages", &group.packages),
            ("when_on", &group.when_on),
            ("when_off", &group.when_off),
        ] {
            for value in values {
                add(key, value);
            }
        }
    }
    lines
}

/// The scripts' awk reader's dump of `path`.
fn shell_dump(path: &std::path::Path) -> BTreeSet<String> {
    let lib = repo_root().join("system/manifest.sh");
    let out = Command::new("bash")
        .args(["-c", "source \"$1\"; manifest_dump \"$2\"", "bash"])
        .arg(&lib)
        .arg(path)
        .output()
        .expect("run bash");
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    String::from_utf8(out.stdout)
        .unwrap()
        .lines()
        .map(str::to_owned)
        .collect()
}

#[test]
fn the_scripts_read_the_release_manifest_as_the_updater_does() {
    let path = repo_root().join(MANIFEST);
    let manifest = Manifest::read(&path).unwrap().unwrap();

    assert_eq!(shell_dump(&path), rust_dump(&manifest));
}

#[test]
fn the_scripts_read_comments_inline_arrays_and_split_arrays_as_the_updater_does() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("packages.toml");
    std::fs::write(&path, SMALL).unwrap();

    assert_eq!(shell_dump(&path), rust_dump(&Manifest::parse(SMALL).unwrap()));
}
