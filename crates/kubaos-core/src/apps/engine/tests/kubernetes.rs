//! The Kubernetes tools of the shipped catalog, installed and removed on the fake machine.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use super::World;
use super::ai::run;
use super::ai::shipped;
use crate::apps::Catalog;
use crate::apps::Platform;
use crate::apps::action::Kind;
use crate::apps::action::Unpack;
use crate::apps::host::fake::Unit;
use crate::apps::picks::Picks;
use crate::unfree::Unfree;

/// The Kubernetes tools of the DevOps category but kubectl, k9s and Lens, which the developer
/// tools' tests cover.
const APPS: [&str; 12] = [
    "argocd",
    "flux",
    "headlamp",
    "helm",
    "k3d",
    "kind",
    "kubectx",
    "kustomize",
    "minikube",
    "skaffold",
    "stern",
    "tilt",
];

fn picks(id: &str) -> Picks {
    let mut picks = Picks::default();
    picks.add(id);
    picks
}

/// A machine whose catalog is the shipped app `id` and the apps it requires, every file of their
/// folders copied with its mode as git keeps it.
fn world(id: &str) -> World {
    let catalog = Catalog::load(&shipped(id).join("..")).unwrap();
    let mut ids = vec![id.to_owned()];
    ids.extend(catalog.get(id).unwrap().requires(None).map(|r| r.id().to_owned()));
    let texts: Vec<(String, String)> = ids
        .iter()
        .map(|id| (id.clone(), fs::read_to_string(shipped(id).join("app.toml")).unwrap()))
        .collect();
    let apps: Vec<(&str, &str)> = texts.iter().map(|(id, text)| (id.as_str(), text.as_str())).collect();
    let world = World::new(&apps);
    for id in &ids {
        for entry in fs::read_dir(shipped(id)).unwrap() {
            let path = entry.unwrap().path();
            let name = path.file_name().unwrap();
            if name == "app.toml" {
                continue;
            }
            let to = world.catalog.join(id).join(name);
            fs::copy(&path, &to).unwrap();
            let executable = fs::metadata(&path).unwrap().permissions().mode() & 0o100 != 0;
            let mode = if executable { 0o755 } else { 0o644 };
            fs::set_permissions(&to, fs::Permissions::from_mode(mode)).unwrap();
        }
    }
    world
}

/// Docker's units, as its packages bring them, so removing the packages takes them: Docker's
/// install leaves `docker.service` and `containerd.service` off and starts `docker.socket`.
fn docker_as_packaged(world: &World) {
    let unit = || Unit {
        exists: true,
        ..Unit::default()
    };
    let mut state = world.host.state.borrow_mut();
    for (package, units) in [
        ("docker-ce", vec!["system:docker.service", "system:docker.socket"]),
        ("containerd.io", vec!["system:containerd.service"]),
    ] {
        state.package_units.insert(
            package.to_owned(),
            units.into_iter().map(|name| (name.to_owned(), unit())).collect(),
        );
    }
}

/// Each download and `.deb` of the app served at its pin, as the fake machine downloads it: a
/// tarball as its release lays it out, the program in a folder of the platform's name.
fn serve(world: &World, id: &str) {
    let catalog = Catalog::load(&world.catalog).unwrap();
    for action in &catalog.get(id).unwrap().actions {
        let Some(pin) = action.download("amd64") else {
            continue;
        };
        let bytes = match &action.kind {
            Kind::Deb { package, version, .. } => format!("deb {package} {version}").into_bytes(),
            Kind::Download {
                unpack: Unpack::TarGz, ..
            } => tarball(world, &format!("linux-amd64/{id}")),
            _ => format!("{id} release").into_bytes(),
        };
        world
            .host
            .state
            .borrow_mut()
            .downloads
            .insert(pin.url.clone(), (pin.sha256.clone(), bytes));
    }
}

/// A gzipped tarball holding `file`, which holds its own path, made by the tar of `world`'s
/// sandbox.
fn tarball(world: &World, file: &str) -> Vec<u8> {
    let src = tempfile::tempdir().unwrap();
    let path = src.path().join(file);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, file).unwrap();
    let archive = src.path().join("archive.tar.gz");
    let top = file.split('/').next().unwrap();
    let made = world
        .host
        .sandbox()
        .command("tar")
        .args(["-c", "--gzip", "-f"])
        .arg(&archive)
        .arg("-C")
        .arg(src.path())
        .arg(top)
        .status()
        .unwrap();
    assert!(made.success());
    fs::read(archive).unwrap()
}

#[test]
fn every_kubernetes_tool_installs_on_the_fake_machine_and_its_removal_leaves_it_as_before() {
    for id in APPS {
        let world = world(id);
        serve(&world, id);
        docker_as_packaged(&world);
        let before = world.host.snapshot();

        let ran = run(&world, |e| e.add(id, None, &picks(id))).unwrap_or_else(|e| panic!("{id}: {e}"));

        let catalog = Catalog::load(&world.catalog).unwrap();
        let app = catalog.get(id).unwrap();
        let names: Vec<&str> = ran.iter().map(|r| r.name.as_str()).collect();
        let mut expected: Vec<&str> = app
            .requires(None)
            .map(|required| catalog.get(required.id()).unwrap().name.as_str())
            .collect();
        expected.push(app.name.as_str());
        assert_eq!(names, expected, "{id}");
        // The version a download's folder and the link to it are filled in with.
        let pinned = app
            .actions
            .iter()
            .find_map(|action| match &action.kind {
                Kind::Download { version, .. } => Some(version.as_str()),
                _ => None,
            })
            .unwrap_or_default();
        let filled = |path: &str| path.replace("@VERSION@", pinned);
        {
            let state = world.host.state.borrow();
            for action in &app.actions {
                match &action.kind {
                    Kind::Apt { packages, repo, .. } => {
                        assert!(packages.iter().all(|p| state.dpkg.contains_key(p)), "{id}");
                        assert_eq!(repo, &None, "{id}: from Ubuntu's archive");
                    }
                    Kind::Deb { package, version, .. } => assert_eq!(&state.dpkg[package], version, "{id}"),
                    Kind::Nix { packages, .. } => assert!(packages.iter().all(|p| state.nix.contains(p)), "{id}"),
                    Kind::Download { to, .. } => assert!(world.system_path(&filled(to)).exists(), "{id}: {to}"),
                    Kind::Symlink { to, target } => {
                        let target = filled(target);
                        let program = world.system_path(&target);
                        assert_eq!(
                            fs::read_link(world.system_path(to)).unwrap(),
                            Path::new(&target),
                            "{id}"
                        );
                        assert_eq!(
                            fs::read_to_string(program).unwrap(),
                            format!("linux-amd64/{id}"),
                            "{id}"
                        );
                    }
                    // The icon of the app's folder, in the theme.
                    Kind::File { to, .. } if to.starts_with("~/.local/share/icons/") => {
                        let icon = world.host_home().join(to.trim_start_matches("~/"));
                        assert!(icon.is_file(), "{id}: {}", icon.display());
                    }
                    other => panic!("{id}: a Kubernetes tool has no {other:?}"),
                }
            }
        }

        let gone = run(&world, |e| e.remove_with(id, &Picks::default())).unwrap_or_else(|e| panic!("{id}: {e}"));

        assert_eq!(gone.len(), ran.len(), "{id}: what it brought goes with it");
        assert_eq!(world.host.snapshot(), before, "{id}");
        assert_eq!(world.journals(id), (None, None), "{id}");
    }
}

#[test]
fn every_kubernetes_tool_installs_on_both_architectures_and_none_is_unfree() {
    let catalog = Catalog::load(&shipped("helm").join("..")).unwrap();
    for arch in ["amd64", "arm64"] {
        for unfree in [Unfree::Allowed, Unfree::Refused] {
            let on: Vec<&str> = APPS
                .into_iter()
                .filter(|id| catalog.available(catalog.get(id).unwrap(), None, Platform::new(arch, unfree)))
                .collect();
            assert_eq!(on, APPS, "{arch}, unfree {unfree:?}");
        }
    }
}
