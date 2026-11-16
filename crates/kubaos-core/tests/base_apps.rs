//! Google Chrome, Docker and Bitwarden desktop leaving the base for the Apps catalog, on a machine
//! that had them: the release's migrations `apps-catalog-handover` (system) and
//! `base-apps-picked` (user), run by bash against the release's own manifest and catalog, with
//! sudo, dpkg and dpkg-query stand-ins over a list of installed packages. Then the plans of the
//! next update, the package manifest's and the catalog's, read the machine they left.

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::fs;
use std::os::unix::fs::PermissionsExt as _;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use kubaos_core::apps;
use kubaos_core::apps::Catalog;
use kubaos_core::apps::Picks;
use kubaos_core::apt::Choices;
use kubaos_core::apt::Dpkg;
use kubaos_core::apt::Manifest;
use kubaos_core::apt::Record;
use kubaos_core::apt::plan;
use tempfile::TempDir;

/// Docker's packages, as the release's docker group and app list them.
const DOCKER: [&str; 5] = [
    "docker-ce",
    "docker-ce-cli",
    "containerd.io",
    "docker-buildx-plugin",
    "docker-compose-plugin",
];

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn executable(path: &Path, body: &str) {
    fs::write(path, format!("#!/usr/bin/env bash\n{body}\n")).expect("write a stand-in");
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("make it run");
}

/// The migration of `kind` whose name ends in `-<slug>.sh`.
fn migration(kind: &str, slug: &str) -> PathBuf {
    fs::read_dir(repo_root().join("migrations").join(kind))
        .expect("read the migrations")
        .map(|e| e.expect("an entry").path())
        .find(|p| p.to_string_lossy().ends_with(&format!("-{slug}.sh")))
        .unwrap_or_else(|| panic!("no {kind} migration {slug}"))
}

/// A machine: its installed packages, the package manifest's record and choices, and a home.
struct Machine {
    dir: TempDir,
    installed: Vec<String>,
}

impl Machine {
    /// A machine with `installed`, the record listing `recorded` under `group`, and `choices`.
    fn new(installed: &[&str], recorded: &[(&str, &str)], choices: &str) -> Self {
        let m = Self {
            dir: tempfile::tempdir().expect("a temp dir"),
            installed: installed.iter().map(|p| (*p).to_owned()).collect(),
        };
        let bin = m.path("bin");
        fs::create_dir_all(&bin).expect("make bin");
        fs::create_dir_all(m.path("home")).expect("make the home");
        let mut packages = String::new();
        for p in installed {
            let _ = writeln!(packages, "{p}");
        }
        fs::write(m.path("installed"), packages).expect("write the installed packages");
        executable(&bin.join("sudo"), "exec \"$@\"");
        executable(&bin.join("dpkg"), "echo amd64");
        executable(
            &bin.join("dpkg-query"),
            "for last; do :; done; grep -qx \"$last\" \"$FAKE/installed\" && printf installed || exit 1",
        );
        let mut record = String::from("# The apt packages KubaOS installed\n");
        for (package, group) in recorded {
            let _ = writeln!(record, "{package}\t{group}");
        }
        fs::write(m.path("apt-installed"), record).expect("write the record");
        fs::write(m.path("apt-groups"), choices).expect("write the choices");
        m
    }

    fn path(&self, name: &str) -> PathBuf {
        self.dir.path().join(name)
    }

    /// Runs the release's migration `slug` of `kind`, as the update would.
    fn migrate(&self, kind: &str, slug: &str) {
        let path = format!(
            "{}:{}",
            self.path("bin").display(),
            std::env::var("PATH").unwrap_or_default()
        );
        let out = Command::new("bash")
            .args(["-euo", "pipefail"])
            .arg(migration(kind, slug))
            .env("PATH", path)
            .env("KUBAOS_DIR", repo_root())
            .env("FAKE", self.dir.path())
            .env("HOME", self.path("home"))
            .env_remove("XDG_CONFIG_HOME")
            .env("KUBAOS_BACKUPS", self.path("backups"))
            .env("APT_CHOICES", self.path("apt-groups"))
            .env("APT_RECORD", self.path("apt-installed"))
            .env("TMPDIR", self.dir.path())
            .output()
            .expect("run bash");
        assert!(
            out.status.success(),
            "{kind}/{slug}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }

    /// The update both migrations are part of.
    fn update(&self) {
        self.migrate("system", "apps-catalog-handover");
        self.migrate("user", "base-apps-picked");
    }

    fn picks(&self) -> Picks {
        Picks::read(&self.path("home/.config/kubaos/apps.toml")).expect("read the picks")
    }

    fn dpkg(&self) -> Dpkg {
        let mut lines = String::new();
        for p in &self.installed {
            let _ = writeln!(lines, "installed install {p} 1.0");
        }
        Dpkg::parse(&lines)
    }
}

/// The next update's plans on `m`: what the package manifest's sync would remove, by a kubaos that
/// knows the catalog and by one from before it, and what the Apps phase would install.
fn next_update(m: &Machine) -> (Vec<String>, Vec<String>, Vec<String>) {
    let root = repo_root();
    let manifest = Manifest::read(&root.join("system/packages.toml"))
        .expect("read the manifest")
        .expect("a manifest");
    let catalog = Catalog::load(&root.join("apps")).expect("load the catalog");
    let choices = Choices::parse(&fs::read_to_string(m.path("apt-groups")).expect("read the choices"));
    let record = Record::parse(&fs::read_to_string(m.path("apt-installed")).expect("read the record"));
    let dpkg = m.dpkg();
    let owned = catalog.packages();
    let before_the_catalog = BTreeSet::new();
    let apt = |catalog| plan::Facts {
        manifest: &manifest,
        choices: &choices,
        record: &record,
        arch: "amd64",
        dpkg: &dpkg,
        catalog,
    };
    let picks = m.picks();
    let nix = BTreeSet::new();
    let apps = apps::plan::Facts {
        catalog: &catalog,
        picks: &picks,
        arch: "amd64",
        dpkg: &dpkg,
        nix: &nix,
    };
    (
        plan::removable(&apt(&owned), None),
        plan::removable(&apt(&before_the_catalog), None),
        apps::plan::installs(&apps).iter().map(|a| a.id.clone()).collect(),
    )
}

#[test]
fn a_machine_with_all_three_keeps_them_and_has_them_picked() {
    let mut installed = vec!["google-chrome-stable", "bitwarden", "curl"];
    installed.extend(DOCKER);
    let mut recorded = vec![("google-chrome-stable", "chrome"), ("curl", "tools")];
    recorded.extend(DOCKER.map(|p| (p, "docker")));
    let m = Machine::new(&installed, &recorded, "");
    // The update that brings the catalog runs the kubaos it started with, from before the
    // catalog, and its sync comes before the migrations.
    let (_, removed_on_the_way, _) = next_update(&m);
    assert!(removed_on_the_way.is_empty(), "{removed_on_the_way:?}");

    m.update();

    let (removed, removed_by_an_older_kubaos, installs) = next_update(&m);
    assert!(removed.is_empty(), "{removed:?}");
    assert!(removed_by_an_older_kubaos.is_empty(), "{removed_by_an_older_kubaos:?}");
    assert!(installs.is_empty(), "{installs:?}");
    let picks: Vec<String> = m.picks().iter().map(str::to_owned).collect();
    assert_eq!(picks, ["bitwarden", "docker", "google-chrome"]);
    let record = Record::parse(&fs::read_to_string(m.path("apt-installed")).expect("read the record"));
    assert!(record.has("curl"), "the base's packages stay the manifest's");
    assert!(
        !record.has("google-chrome-stable") && DOCKER.iter().all(|p| !record.has(p)),
        "the catalog's left the record"
    );
}

#[test]
fn a_group_the_user_turned_off_is_not_picked_and_nothing_is_installed() {
    let m = Machine::new(
        &["google-chrome-stable"],
        &[("google-chrome-stable", "chrome")],
        "docker=off\n",
    );

    m.update();

    let (removed, removed_by_an_older_kubaos, installs) = next_update(&m);
    assert!(removed.is_empty() && removed_by_an_older_kubaos.is_empty());
    assert!(installs.is_empty(), "{installs:?}");
    assert!(!m.picks().has("docker"));
    assert!(m.picks().has("google-chrome") && !m.picks().has("bitwarden"));
}

#[test]
fn the_picks_the_user_had_stay_and_a_second_run_changes_nothing() {
    let m = Machine::new(&["bitwarden"], &[], "");
    fs::create_dir_all(m.path("home/.config/kubaos")).expect("make the user layer");
    fs::write(m.path("home/.config/kubaos/apps.toml"), "apps = [\"t3-code\"]\n").expect("write the picks");

    m.update();
    let once = fs::read_to_string(m.path("home/.config/kubaos/apps.toml")).expect("read the picks");
    m.update();

    let twice = fs::read_to_string(m.path("home/.config/kubaos/apps.toml")).expect("read the picks");
    assert_eq!(once, twice);
    assert!(twice.ends_with("apps = [\"bitwarden\", \"t3-code\"]\n"), "{twice}");
    assert_eq!(Picks::read(&m.path("home/.config/kubaos/apps.toml")).unwrap(), {
        let mut picks = Picks::default();
        picks.add("bitwarden");
        picks.add("t3-code");
        picks
    });
}
