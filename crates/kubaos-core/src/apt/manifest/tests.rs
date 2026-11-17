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
when_on = ["system/docker-on.sh"]
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
    assert_eq!(docker.when_on, ["system/docker-on.sh"]);
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
fn a_repos_next_key_is_all_three_of_its_keys_or_none() {
    let repo = "[repo.v]\nkey_url = \"https://v/k\"\nfingerprint = \"AB\"\nkeyring = \"/etc/apt/keyrings/v.gpg\"\n";
    let whole = format!(
        "{repo}next_key_url = \"https://v/n\"\nnext_fingerprint = \"CD\"\nnext_keyring = \"/etc/apt/keyrings/v-next.gpg\"\n"
    );
    let half = format!("{repo}next_fingerprint = \"CD\"\n");

    let manifest = Manifest::parse(&whole).unwrap();
    let err = Manifest::parse(&half).unwrap_err();

    let keyrings: Vec<&str> = manifest.repos()["v"].keyrings().collect();
    assert_eq!(keyrings, ["/etc/apt/keyrings/v.gpg", "/etc/apt/keyrings/v-next.gpg"]);
    assert!(matches!(err, ManifestError::HalfNextKey { ref repo } if repo == "v"));
}

#[test]
fn a_repos_global_keyring_is_a_file_name_of_trusted_gpg_d() {
    let repo = "[repo.v]\nkey_url = \"https://v/k\"\nfingerprint = \"AB\"\nkeyring = \"/etc/apt/keyrings/v.gpg\"\n";
    let good = format!("{repo}global_keyrings = [\"v.gpg\", \"v-old.asc\"]\nglobal_fingerprints = [\"CD\"]\n");

    let manifest = Manifest::parse(&good).unwrap();

    assert_eq!(manifest.repos()["v"].global_keyrings, ["v.gpg", "v-old.asc"]);
    for bad in [
        "../sources.list",
        "/etc/apt/trusted.gpg",
        "v",
        ".gpg",
        "sub/v.gpg",
        ".v.gpg",
        "v w.gpg",
        "*.gpg",
        "v$x.gpg",
        "v\tw.gpg",
    ] {
        let err = Manifest::parse(&format!("{repo}global_keyrings = [\"{bad}\"]\n")).unwrap_err();
        assert!(
            matches!(err, ManifestError::InvalidGlobalKeyring { ref name, .. } if name == bad),
            "{bad}: {err}"
        );
    }
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
        let next = [
            ("next_key_url", &repo.next_key_url),
            ("next_fingerprint", &repo.next_fingerprint),
            ("next_keyring", &repo.next_keyring),
        ]
        .into_iter()
        .filter_map(|(key, value)| Some((key, value.as_ref()?)));
        for (key, value) in [
            ("key_url", &repo.key_url),
            ("fingerprint", &repo.fingerprint),
            ("keyring", &repo.keyring),
        ]
        .into_iter()
        .chain(next)
        {
            lines.insert(format!("repo.{name}\t{key}\t{value}"));
        }
        for (key, values) in [
            ("global_keyrings", &repo.global_keyrings),
            ("global_fingerprints", &repo.global_fingerprints),
        ] {
            for value in values {
                lines.insert(format!("repo.{name}\t{key}\t{value}"));
            }
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

#[test]
fn a_repo_is_in_place_once_its_own_sources_file_is_signed_by_its_primary_keyring_and_no_other_and_they_are_there() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let text = "[repo.v]\nkey_url = \"https://v/k\"\nfingerprint = \"AB\"\nkeyring = \"/etc/apt/keyrings/v.gpg\"\n";
    let manifest = Manifest::parse(text).unwrap();
    let repo = &manifest.repos()["v"];
    let sources = root.join("etc/apt/sources.list.d");
    std::fs::create_dir_all(&sources).unwrap();
    std::fs::create_dir_all(root.join("etc/apt/keyrings")).unwrap();
    let ours = "Types: deb\nURIs: https://v/apt\nSigned-By: /etc/apt/keyrings/v.gpg\n";

    assert!(!repo.in_place("v", root), "nothing there");
    std::fs::write(sources.join("v.list"), "deb https://v/apt stable main\n").unwrap();
    assert!(!repo.in_place("v", root), "the vendor's own list");
    std::fs::write(
        sources.join("v.sources"),
        "Types: deb\nURIs: https://v/apt\nSigned-By: /usr/share/keyrings/v.asc\n",
    )
    .unwrap();
    std::fs::write(root.join("etc/apt/keyrings/v.gpg"), "key").unwrap();
    assert!(
        !repo.in_place("v", root),
        "the vendor's own sources file, its keyring another"
    );
    std::fs::write(sources.join("v.sources"), ours).unwrap();
    assert!(repo.in_place("v", root));
    std::fs::remove_file(root.join("etc/apt/keyrings/v.gpg")).unwrap();
    assert!(!repo.in_place("v", root), "its keyring gone");

    let next = format!(
        "{text}next_key_url = \"https://v/k2\"\nnext_fingerprint = \"CD\"\nnext_keyring = \"/etc/apt/keyrings/v-next.gpg\"\n"
    );
    let manifest = Manifest::parse(&next).unwrap();
    let repo = &manifest.repos()["v"];
    std::fs::write(root.join("etc/apt/keyrings/v.gpg"), "key").unwrap();
    std::fs::write(root.join("etc/apt/keyrings/v-next.gpg"), "key").unwrap();
    assert!(
        repo.in_place("v", root),
        "signed by its key alone, as a vendor writes the file again"
    );
    let both = "Types: deb\nURIs: https://v/apt\nSigned-By: /etc/apt/keyrings/v.gpg /etc/apt/keyrings/v-next.gpg\n";
    std::fs::write(sources.join("v.sources"), both).unwrap();
    assert!(repo.in_place("v", root));
    std::fs::write(
        sources.join("v.sources"),
        "Types: deb\nURIs: https://v/apt\nSigned-By: /etc/apt/keyrings/v-next.gpg\n",
    )
    .unwrap();
    assert!(!repo.in_place("v", root), "not signed by its primary key");
    std::fs::write(root.join("etc/apt/keyrings/stray.gpg"), "key").unwrap();
    std::fs::write(
        sources.join("v.sources"),
        "Types: deb\nURIs: https://v/apt\nSigned-By: /etc/apt/keyrings/v.gpg /etc/apt/keyrings/stray.gpg\n",
    )
    .unwrap();
    assert!(!repo.in_place("v", root), "a vendor's stray key");
    std::fs::write(sources.join("v.sources"), format!("{both}Enabled: no\n")).unwrap();
    assert!(!repo.in_place("v", root), "turned off");
    std::fs::write(sources.join("v.sources"), both).unwrap();
    std::fs::remove_file(root.join("etc/apt/keyrings/v-next.gpg")).unwrap();
    assert!(!repo.in_place("v", root), "a keyring it names gone");
}

#[test]
fn every_shipped_repo_is_in_place_once_repo_setup_wrote_its_sources_file_and_keys() {
    let manifest = Manifest::read(&repo_root().join(MANIFEST)).unwrap().unwrap();
    for (name, repo) in manifest.repos() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let sources = root.join("etc/apt/sources.list.d");
        std::fs::create_dir_all(&sources).unwrap();
        let shipped = repo_root().join(format!("system/files/apt/sources.list.d/{name}.sources"));
        std::fs::copy(&shipped, sources.join(format!("{name}.sources"))).unwrap();
        for keyring in repo.keyrings() {
            let at = root.join(keyring.trim_start_matches('/'));
            std::fs::create_dir_all(at.parent().unwrap()).unwrap();
            std::fs::write(at, "key").unwrap();
        }

        assert!(repo.in_place(name, root), "{name}");
    }
}
