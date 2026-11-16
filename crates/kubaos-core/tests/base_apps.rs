//! Google Chrome, Docker and Bitwarden desktop leaving the base for the Apps catalog, on a machine
//! that had them: the release's migrations `apps-catalog-handover` and `base-apps-setup` (system)
//! and `base-apps-picked` (user), run by bash in a sandbox (`kubaos_core::sandbox`) against a copy
//! of the release's scripts, manifest and catalog, with sudo, dpkg and dpkg-query stand-ins over a
//! list of installed packages, and
//! the removal of what an older base install left standing in as lines in a log. Then the package
//! manifest's plan of the next update reads the machine they left. Taking the apps in, the user
//! migration apps-adopt, is `kubaos app adopt`, which the updater's tests cover. Last, the Apps
//! catalog's polkit action, which `system/lib.sh` installs on a machine without nix too.

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use kubaos_core::apps::Catalog;
use kubaos_core::apps::Picks;
use kubaos_core::apt::Choices;
use kubaos_core::apt::Dpkg;
use kubaos_core::apt::Manifest;
use kubaos_core::apt::Record;
use kubaos_core::apt::plan;
use kubaos_core::sandbox::Sandbox;
use tempfile::TempDir;

/// Docker's packages, as the release's docker group and app list them.
const DOCKER: [&str; 5] = [
    "docker-ce",
    "docker-ce-cli",
    "containerd.io",
    "docker-buildx-plugin",
    "docker-compose-plugin",
];

/// The user running the tests, by name.
fn me() -> String {
    let out = Command::new("id").arg("-un").output().expect("id").stdout;
    String::from_utf8(out).expect("a name").trim().to_owned()
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The migration of `kind` whose name ends in `-<slug>.sh`.
fn migration(kind: &str, slug: &str) -> PathBuf {
    fs::read_dir(repo_root().join("migrations").join(kind))
        .expect("read the migrations")
        .map(|e| e.expect("an entry").path())
        .find(|p| p.to_string_lossy().ends_with(&format!("-{slug}.sh")))
        .unwrap_or_else(|| panic!("no {kind} migration {slug}"))
}

/// What `system/lib.sh` takes away of an older base install, which the copy's stand-ins replace.
const HOOKS: [&str; 2] = ["remove_chrome_theme", "remove_bitwarden_autostart"];

/// A machine: its installed packages, the package manifest's record and choices, a home, and a
/// copy of the release's checkout.
struct Machine {
    dir: TempDir,
    installed: Vec<String>,
    sandbox: Sandbox,
}

impl Machine {
    /// A machine with `installed`, the record listing `recorded` under `group`, and `choices`.
    fn new(installed: &[&str], recorded: &[(&str, &str)], choices: &str) -> Self {
        let dir = tempfile::tempdir().expect("a temp dir");
        let home = dir.path().join("home");
        fs::create_dir_all(&home).expect("make the home");
        let sandbox = Sandbox::new(&dir.path().join("sandbox"), &home, me());
        let m = Self {
            dir,
            installed: installed.iter().map(|p| (*p).to_owned()).collect(),
            sandbox,
        };
        let mut packages = String::new();
        for p in installed {
            let _ = writeln!(packages, "{p}");
        }
        fs::write(m.path("installed"), packages).expect("write the installed packages");
        m.sandbox.stub("sudo", "exec \"$@\"");
        m.sandbox.stub("dpkg", "echo amd64");
        m.sandbox.stub(
            "dpkg-query",
            "for last; do :; done; grep -qx \"$last\" \"$FAKE/installed\" && printf installed || exit 1",
        );
        let mut record = String::from("# The apt packages KubaOS installed\n");
        for (package, group) in recorded {
            let _ = writeln!(record, "{package}\t{group}");
        }
        fs::write(m.path("apt-installed"), record).expect("write the record");
        fs::write(m.path("apt-groups"), choices).expect("write the choices");
        m.checkout();
        m
    }

    /// The release's migration helpers, system scripts, manifest and catalog, the hooks stand-ins
    /// that log their name and user.
    fn checkout(&self) {
        let root = repo_root();
        for dir in ["checkout/migrations", "checkout/system", "checkout/apps"] {
            fs::create_dir_all(self.path(dir)).expect("make the checkout");
        }
        for file in [
            "migrations/lib.sh",
            "system/lib.sh",
            "system/manifest.sh",
            "system/packages.toml",
        ] {
            fs::copy(root.join(file), self.path("checkout").join(file)).expect("copy the release's file");
        }
        copy_dir(&root.join("apps"), &self.path("checkout/apps"));
        let mut lib = fs::read_to_string(self.path("checkout/system/lib.sh")).expect("read lib.sh");
        for hook in HOOKS {
            let _ = writeln!(lib, "{hook}() {{ echo \"{hook} $TARGET_USER\" >>\"$FAKE/hooks\"; }}");
        }
        fs::write(self.path("checkout/system/lib.sh"), lib).expect("write the stand-ins");
        fs::write(self.path("hooks"), "").expect("start the hooks' log");
    }

    /// The hooks the migrations ran, with `me` for the user running the tests.
    fn hooks(&self) -> Vec<String> {
        let me = me();
        fs::read_to_string(self.path("hooks"))
            .expect("read the hooks' log")
            .lines()
            .map(|line| line.replace(&me, "me"))
            .collect()
    }

    fn path(&self, name: &str) -> PathBuf {
        self.dir.path().join(name)
    }

    /// Runs the release's migration `slug` of `kind`, as the update would.
    fn migrate(&self, kind: &str, slug: &str) {
        let out = self
            .sandbox
            .command("bash")
            .args(["-euo", "pipefail"])
            .arg(migration(kind, slug))
            .env("KUBAOS_DIR", self.path("checkout"))
            .env("FAKE", self.dir.path())
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

    /// The update the migrations are part of.
    fn update(&self) {
        self.migrate("system", "apps-catalog-handover");
        self.migrate("system", "base-apps-setup");
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

/// Copies the folder `from` to `to`, everything in it.
fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).expect("make the copy");
    for entry in fs::read_dir(from).expect("read the folder") {
        let path = entry.expect("an entry").path();
        let target = to.join(path.file_name().expect("a name"));
        if path.is_dir() {
            copy_dir(&path, &target);
        } else {
            fs::copy(&path, &target).expect("copy a file");
        }
    }
}

/// The next update's plans on `m`: what the package manifest's sync would remove, by a kubaos that
/// knows the catalog and by one from before it.
fn next_update(m: &Machine) -> (Vec<String>, Vec<String>) {
    let root = m.path("checkout");
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
    (
        plan::removable(&apt(&owned), None),
        plan::removable(&apt(&before_the_catalog), None),
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
    let (_, removed_on_the_way) = next_update(&m);
    assert!(removed_on_the_way.is_empty(), "{removed_on_the_way:?}");

    m.update();

    let (removed, removed_by_an_older_kubaos) = next_update(&m);
    assert!(removed.is_empty(), "{removed:?}");
    assert!(removed_by_an_older_kubaos.is_empty(), "{removed_by_an_older_kubaos:?}");
    let picks: Vec<String> = m.picks().iter().map(str::to_owned).collect();
    assert_eq!(picks, ["bitwarden", "docker", "google-chrome"]);
    let record = Record::parse(&fs::read_to_string(m.path("apt-installed")).expect("read the record"));
    assert!(record.has("curl"), "the base's packages stay the manifest's");
    assert!(
        !record.has("google-chrome-stable") && DOCKER.iter().all(|p| !record.has(p)),
        "the catalog's left the record"
    );
    assert!(m.hooks().is_empty(), "{:?}", m.hooks());
}

#[test]
fn a_group_the_user_turned_off_is_not_picked_and_nothing_is_installed_or_removed() {
    let mut installed = vec!["google-chrome-stable"];
    installed.extend(DOCKER);
    let m = Machine::new(&installed, &[("google-chrome-stable", "chrome")], "docker=off\n");

    m.update();

    let (removed, removed_by_an_older_kubaos) = next_update(&m);
    assert!(removed.is_empty() && removed_by_an_older_kubaos.is_empty());
    assert!(!m.picks().has("docker"));
    assert!(m.picks().has("google-chrome") && !m.picks().has("bitwarden"));
    assert_eq!(m.hooks(), ["remove_bitwarden_autostart me"]);
}

#[test]
fn without_the_apps_only_what_kubaos_alone_put_there_goes() {
    // Another Chrome build reads the theme's policy, and Ubuntu's Docker has the docker group
    // and the shutdown drop-in too.
    let m = Machine::new(&["google-chrome-beta", "docker.io"], &[], "");

    m.update();

    assert_eq!(m.hooks(), ["remove_bitwarden_autostart me"]);
    let bare = Machine::new(&[], &[], "");
    bare.update();
    assert_eq!(
        bare.hooks(),
        ["remove_chrome_theme me", "remove_bitwarden_autostart me"]
    );
}

#[test]
fn the_handover_goes_on_once_the_groups_are_gone_from_the_manifest() {
    let mut installed = vec!["google-chrome-stable", "curl"];
    installed.extend(DOCKER);
    let mut recorded = vec![("google-chrome-stable", "chrome"), ("curl", "tools")];
    recorded.extend(DOCKER.map(|p| (p, "docker")));
    let m = Machine::new(&installed, &recorded, "");
    let manifest = m.path("checkout/system/packages.toml");
    let text = fs::read_to_string(&manifest).expect("read the manifest");
    let without: String = text
        .split("\n[")
        .filter(|table| !table.starts_with("group.docker]") && !table.starts_with("group.chrome]"))
        .collect::<Vec<_>>()
        .join("\n[");
    assert!(!without.contains("[group.docker]") && !without.contains("[group.chrome]"));
    fs::write(&manifest, without).expect("write the manifest without them");

    m.migrate("system", "apps-catalog-handover");

    let record = Record::parse(&fs::read_to_string(m.path("apt-installed")).expect("read the record"));
    assert!(record.has("curl"));
    assert!(!record.has("google-chrome-stable") && DOCKER.iter().all(|p| !record.has(p)));
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

#[test]
fn a_machine_without_nix_still_gets_the_apps_polkit_action() {
    // Regression: the action came only with the helper, which 60-nix.sh skips without nix.
    let dir = tempfile::tempdir().expect("a temp dir");
    let sandbox = Sandbox::new(&dir.path().join("sandbox"), dir.path(), me());
    sandbox.without("nix");
    let lib = repo_root().join("system/lib.sh");
    let script = format!(
        "source '{}'\n\
         install_root_file() {{ echo \"file $2\"; }}\n\
         install_apps_helper() {{ echo helper; }}\n\
         install_apps_root",
        lib.display()
    );

    let out = sandbox
        .command("bash")
        .args(["-euo", "pipefail", "-c", &script])
        .output()
        .expect("run bash");

    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        "file /usr/share/polkit-1/actions/org.kubaos.apps.policy\n"
    );
}
