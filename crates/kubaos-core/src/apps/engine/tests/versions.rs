//! An app's downloads of one version: one unpacked into another's folder, owning only what it
//! adds, and the symlinks and unit files that name the version's folder.

use std::fs;
use std::path::Path;

use super::World;
use super::app;
use super::packed;
use crate::apps::host::fake::Unit;

/// A runtime unpacked to `~/.vendor/versions/<version>`, a GPU library unpacked into its `lib/gpu`,
/// a link to the runtime's program and a user unit running it, all of `version`.
fn runtime(version: &str) -> String {
    let (main, gpu) = shas(version);
    app(&format!(
        r#"
[[action]]
name = "Download the runtime"
kind = "download"
to = "~/.vendor/versions/@VERSION@"
version = "{version}"
unpack = "tar-zst"
amd64 = {{ url = "https://example.com/runtime-{version}.tar.zst", sha256 = "{main}" }}

[[action]]
name = "Download the GPU library into the runtime"
kind = "download"
to = "~/.vendor/versions/@VERSION@/lib/gpu"
version = "{version}"
unpack = "tar-zst"
amd64 = {{ url = "https://example.com/gpu-{version}.tar.zst", sha256 = "{gpu}" }}

[[action]]
name = "Put the program on PATH"
kind = "symlink"
to = "~/.local/bin/tool"
target = "@HOME@/.vendor/versions/@VERSION@/bin/tool"

[[action]]
name = "Put the service in place"
kind = "file"
to = "~/.config/systemd/user/tool.service"
template = true
text = "[Service]\nExecStart=@HOME@/.vendor/versions/@VERSION@/bin/tool serve\n"
"#
    ))
}

/// The sha256 pins of the runtime's and the GPU library's archives of `version`.
fn shas(version: &str) -> (String, String) {
    let pin = |digits: String| digits.repeat(64).chars().take(64).collect();
    let digits = version.replace('.', "");
    (pin(digits.clone()), pin(format!("9{digits}")))
}

/// A world whose release offers the runtime at each of `versions`, its folder `runtime`.
fn world(versions: &[&str]) -> World {
    let world = World::new(&[("runtime", &runtime(versions[0]))]);
    for version in versions {
        let (main, gpu) = shas(version);
        let mut state = world.host.state.borrow_mut();
        state.downloads.insert(
            format!("https://example.com/runtime-{version}.tar.zst"),
            (main, packed(&format!("tool {version}"), "tar-zst")),
        );
        state.downloads.insert(
            format!("https://example.com/gpu-{version}.tar.zst"),
            (gpu, packed(&format!("gpu {version}"), "tar-zst")),
        );
    }
    world
}

#[test]
fn a_download_into_another_downloads_folder_owns_only_what_it_adds_and_the_app_goes_whole() {
    let world = world(&["1.1"]);
    let before = world.host.snapshot();

    world.install("runtime", true).0.unwrap();

    let folder = world.host_home().join(".vendor/versions/1.1");
    assert_eq!(fs::read_to_string(folder.join("bin/tool")).unwrap(), "tool 1.1");
    assert_eq!(fs::read_to_string(folder.join("lib/gpu/bin/tool")).unwrap(), "gpu 1.1");
    let journal = world.journals("runtime").0.unwrap();
    let gpu = journal.entries.iter().find(|e| e.step == 2).unwrap();
    let owned: Vec<&str> = gpu
        .claims
        .iter()
        .filter(|c| c.owned)
        .map(|c| c.claim.as_str())
        .collect();
    assert_eq!(
        owned,
        [
            format!("dir:{}", folder.join("lib").display()),
            format!("path:{}", folder.join("lib/gpu").display())
        ]
    );
    world.remove("runtime").0.unwrap();
    assert_eq!(world.host.snapshot(), before);
}

#[test]
fn a_symlink_and_a_unit_file_name_the_folder_of_the_apps_download_version() {
    let world = world(&["1.1", "1.3"]);
    let home = world.host_home();

    world.install("runtime", true).0.unwrap();
    world.write_app("runtime", &runtime("1.3"));
    world.install("runtime", true).0.unwrap();

    let link = fs::read_link(home.join(".local/bin/tool")).unwrap();
    assert_eq!(link, home.join(".vendor/versions/1.3/bin/tool"));
    let unit = fs::read_to_string(home.join(".config/systemd/user/tool.service")).unwrap();
    assert!(
        unit.contains(&format!(
            "ExecStart={}/.vendor/versions/1.3/bin/tool serve",
            home.display()
        )),
        "{unit}"
    );
    assert_eq!(
        fs::read_to_string(home.join(".vendor/versions/1.3/lib/gpu/bin/tool")).unwrap(),
        "gpu 1.3"
    );
    assert!(!home.join(".vendor/versions/1.1").exists());
}

#[test]
fn a_unit_file_a_release_rewrites_restarts_its_unit_while_it_runs() {
    let world = world(&["1.1", "1.3"]);
    world.install("runtime", true).0.unwrap();
    world.host.state.borrow_mut().units.insert(
        "user:tool.service".to_owned(),
        Unit {
            exists: true,
            active: true,
            ..Unit::default()
        },
    );

    world.write_app("runtime", &runtime("1.3"));
    world.install("runtime", true).0.unwrap();

    let log = world.host.state.borrow().log.clone();
    assert!(log.iter().any(|l| l == "restart tool.service"), "{log:?}");
}

#[test]
fn a_unit_file_written_while_its_unit_is_stopped_restarts_nothing() {
    let world = world(&["1.1"]);

    world.install("runtime", true).0.unwrap();

    let log = world.host.state.borrow().log.clone();
    assert!(!log.iter().any(|l| l.starts_with("restart")), "{log:?}");
    assert!(Path::new(&world.host_home().join(".config/systemd/user/tool.service")).is_file());
}
