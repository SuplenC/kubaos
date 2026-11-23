//! The cloud CLIs, container and CI tools of the shipped catalog, installed and removed on the fake
//! machine.

use super::ai::run;
use super::ai::shipped;
use super::ai::world;
use crate::apps::Catalog;
use crate::apps::Platform;
use crate::apps::action::Kind;
use crate::apps::folder::Requirement;
use crate::apps::picks::Picks;
use crate::unfree::Unfree;

/// Each its packages, from a vendor's repository or nixpkgs, and nothing an app of its own needs.
const ALONE: [&str; 8] = [
    "aws-cli",
    "azure-cli",
    "dive",
    "doctl",
    "flyctl",
    "glab",
    "google-cloud-cli",
    "hcloud",
];

/// Every one of them: those, and act, which runs its jobs in Docker.
const ALL: [&str; 9] = [
    "act",
    "aws-cli",
    "azure-cli",
    "dive",
    "doctl",
    "flyctl",
    "glab",
    "google-cloud-cli",
    "hcloud",
];

#[test]
fn every_cloud_tool_installs_on_the_fake_machine_and_its_removal_leaves_it_as_before() {
    for id in ALONE {
        let world = world(id);
        let before = world.host.snapshot();
        let mut picks = Picks::default();
        picks.add(id);

        let ran = run(&world, |e| e.add(id, None, &picks)).unwrap_or_else(|e| panic!("{id}: {e}"));

        let catalog = Catalog::load(&world.catalog).unwrap();
        let app = catalog.get(id).unwrap();
        assert_eq!(ran.len(), 1, "{id}");
        {
            let state = world.host.state.borrow();
            for action in &app.actions {
                match &action.kind {
                    Kind::Apt { packages, repo, .. } => {
                        assert!(packages.iter().all(|p| state.dpkg.contains_key(p)), "{id}");
                        let repo = repo.as_ref().unwrap_or_else(|| panic!("{id}: Ubuntu's is too old"));
                        assert!(state.repos.contains(repo), "{id}: {repo} wasn't set up");
                    }
                    Kind::Nix { packages, .. } => assert!(packages.iter().all(|p| state.nix.contains(p)), "{id}"),
                    // The icon of the app's folder, in the theme.
                    Kind::File { to, .. } if to.starts_with("~/.local/share/icons/") => {
                        let icon = world.host_home().join(to.trim_start_matches("~/"));
                        assert!(icon.is_file(), "{id}: {}", icon.display());
                    }
                    other => panic!("{id}: a cloud tool has no {other:?}"),
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
fn act_requires_docker_and_every_cloud_tool_comes_on_both_architectures_free() {
    let catalog = Catalog::load(&shipped("act").join("..")).unwrap();

    let act = catalog.get("act").unwrap();
    let required: Vec<&str> = act.requires.iter().map(Requirement::id).collect();
    assert_eq!(required, ["docker"]);
    for id in ALONE {
        assert!(catalog.get(id).unwrap().requires.is_empty(), "{id}");
    }

    // Google's and Microsoft's repositories and nixpkgs build each one for amd64 and arm64, and
    // none is unfree, so a machine that refuses unfree packages has them all.
    for arch in ["amd64", "arm64"] {
        let on: Vec<&str> = ALL
            .into_iter()
            .filter(|id| catalog.available(catalog.get(id).unwrap(), None, Platform::new(arch, Unfree::Refused)))
            .collect();
        assert_eq!(on, ALL, "{arch}");
    }
}
