//! The infrastructure-as-code, secrets and security tools of the shipped catalog, installed and
//! removed on the fake machine.

use std::ffi::OsStr;
use std::fs;
use std::path::Path;

use super::World;
use super::ai::run;
use super::ai::shipped;
use super::ai::world;
use crate::apps::Catalog;
use crate::apps::Platform;
use crate::apps::action::Kind;
use crate::apps::picks::Picks;
use crate::unfree::Unfree;

/// Each its packages, from a vendor's repository, Ubuntu's archive or nixpkgs.
const PACKAGED: [&str; 8] = [
    "age",
    "ansible",
    "opentofu",
    "packer",
    "sops",
    "terraform",
    "trivy",
    "vault",
];

/// Every one of them: those and Pulumi's download.
const ALL: [&str; 9] = [
    "age",
    "ansible",
    "opentofu",
    "packer",
    "pulumi",
    "sops",
    "terraform",
    "trivy",
    "vault",
];

fn picks(id: &str) -> Picks {
    let mut picks = Picks::default();
    picks.add(id);
    picks
}

#[test]
fn every_packaged_iac_tool_installs_on_the_fake_machine_and_its_removal_leaves_it_as_before() {
    for id in PACKAGED {
        let world = world(id);
        let before = world.host.snapshot();

        let ran = run(&world, |e| e.add(id, None, &picks(id))).unwrap_or_else(|e| panic!("{id}: {e}"));

        let catalog = Catalog::load(&world.catalog).unwrap();
        let app = catalog.get(id).unwrap();
        assert_eq!(ran.len(), 1, "{id}");
        {
            let state = world.host.state.borrow();
            for action in &app.actions {
                match &action.kind {
                    Kind::Apt { packages, repo, .. } => {
                        assert!(packages.iter().all(|p| state.dpkg.contains_key(p)), "{id}");
                        if let Some(repo) = repo {
                            assert!(state.repos.contains(repo), "{id}: {repo} wasn't set up");
                        }
                    }
                    Kind::Nix { packages, .. } => assert!(packages.iter().all(|p| state.nix.contains(p)), "{id}"),
                    // The icon of the app's folder, in the theme.
                    Kind::File { to, .. } if to.starts_with("~/.local/share/icons/") => {
                        let icon = world.host_home().join(to.trim_start_matches("~/"));
                        assert!(icon.is_file(), "{id}: {}", icon.display());
                    }
                    other => panic!("{id}: a packaged tool has no {other:?}"),
                }
            }
        }

        let gone = run(&world, |e| e.remove_with(id, &Picks::default())).unwrap_or_else(|e| panic!("{id}: {e}"));

        assert_eq!(gone.len(), 1, "{id}");
        assert_eq!(world.host.snapshot(), before, "{id}");
        assert_eq!(world.journals(id), (None, None), "{id}");
    }
}

#[test]
fn terraform_packer_and_vault_share_hashicorps_repository() {
    let catalog = Catalog::load(&shipped("terraform").join("..")).unwrap();
    for id in ["terraform", "packer", "vault"] {
        let repos: Vec<&str> = catalog
            .get(id)
            .unwrap()
            .actions
            .iter()
            .filter_map(|a| match &a.kind {
                Kind::Apt { repo, .. } => repo.as_deref(),
                _ => None,
            })
            .collect();
        assert_eq!(repos, ["hashicorp"], "{id}");
    }
}

#[test]
fn every_iac_tool_is_on_both_architectures_and_none_is_unfree() {
    let catalog = Catalog::load(&shipped("terraform").join("..")).unwrap();
    for arch in ["amd64", "arm64"] {
        let on: Vec<&str> = ALL
            .into_iter()
            .filter(|id| catalog.available(catalog.get(id).unwrap(), None, Platform::new(arch, Unfree::Refused)))
            .collect();
        assert_eq!(on, ALL, "{arch}");
    }
}

const PULUMI: &str = "pulumi";

/// A Pulumi release, packed as Pulumi packs it: the CLI and its language hosts in `pulumi/`.
fn pulumi_tarball() -> Vec<u8> {
    let src = tempfile::tempdir().unwrap();
    let dir = src.path().join("pulumi");
    fs::create_dir_all(&dir).unwrap();
    for program in ["pulumi", "pulumi-language-nodejs", "pulumi-language-python"] {
        fs::write(dir.join(program), program).unwrap();
    }
    let archive = src.path().join("pulumi.tar.gz");
    let made = crate::sandbox::output(
        "tar",
        [
            OsStr::new("-czf"),
            archive.as_os_str(),
            OsStr::new("-C"),
            src.path().as_os_str(),
            OsStr::new("pulumi"),
        ],
    );
    assert!(made.status.success());
    fs::read(archive).unwrap()
}

/// The shipped Pulumi's world, its release served at the URL and sha256 its folder pins, and the
/// version that pin is.
fn pulumi_world() -> (World, String) {
    let world = world(PULUMI);
    let catalog = Catalog::load(&world.catalog).unwrap();
    let Kind::Download { amd64, version, .. } = &catalog.get(PULUMI).unwrap().actions[0].kind else {
        panic!("Pulumi's first step is its download");
    };
    let pin = amd64.as_ref().unwrap();
    world
        .host
        .state
        .borrow_mut()
        .downloads
        .insert(pin.url.clone(), (pin.sha256.clone(), pulumi_tarball()));
    (world, version.clone())
}

#[test]
fn pulumi_unpacks_its_cli_beside_its_language_hosts_and_links_the_cli_alone_onto_path() {
    let (world, version) = pulumi_world();
    let before = world.host.snapshot();

    run(&world, |e| e.add(PULUMI, None, &picks(PULUMI))).unwrap();

    let folder = format!("/opt/pulumi/{version}");
    for program in ["pulumi", "pulumi-language-nodejs", "pulumi-language-python"] {
        let path = world.system_path(&format!("{folder}/{program}"));
        assert_eq!(fs::read_to_string(&path).unwrap(), program, "{}", path.display());
    }
    // Pulumi finds its language hosts beside the program the link resolves to.
    assert_eq!(
        fs::read_link(world.system_path("/usr/local/bin/pulumi")).unwrap(),
        Path::new(&format!("{folder}/pulumi"))
    );
    assert!(!world.system_path("/usr/local/bin/pulumi-language-nodejs").exists());

    run(&world, |e| e.remove_with(PULUMI, &Picks::default())).unwrap();

    assert_eq!(world.journals(PULUMI), (None, None));
    assert_eq!(world.host.snapshot(), before);
}
