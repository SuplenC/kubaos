//! The scripts' side of the manifest, `system/manifest.sh`, run by bash against a fixture
//! manifest with apt, dpkg and sudo replaced by functions that keep their state in a folder.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use super::Choices;
use super::Record;
use super::Switch;

const MANIFEST: &str = r#"
[repo.docker]
key_url = "https://example.com/gpg"
fingerprint = "AB"
keyring = "/etc/apt/keyrings/docker.gpg"

[group.tools]
summary = "Tools"
packages = ["curl", "git"]

[group.docker]
summary = "Docker"
repo = "docker"
optional = "on"
packages = ["docker-ce"]

[group.piv]
summary = "PIV"
optional = "off"
packages = ["pcscd"]

[group.chrome]
summary = "Chrome"
arch = "amd64"
packages = ["google-chrome-stable"]

[group.hyprland]
summary = "Hyprland"
hold = true
if_available = true
packages = ["hyprland", "nope-guiutils"]
"#;

/// The functions that stand in for apt, dpkg and sudo. `installed` lists the installed
/// packages, `held` the held ones, and `log` what was asked.
const FAKES: &str = r#"
pkg_installed() { grep -qx "$1" "$FAKE/installed" 2>/dev/null; }
pkg_has_candidate() { [[ "$1" != nope-* ]]; }
arch() { echo "${FAKE_ARCH:-amd64}"; }
apt_install() {
  local p
  for p in "$@"; do pkg_installed "$p" || echo "$p" >>"$FAKE/installed"; done
  echo "install $*" >>"$FAKE/log"
}
apt-mark() {
  case "$1" in
    showhold) cat "$FAKE/held" 2>/dev/null || true ;;
    hold) shift; printf '%s\n' "$@" >>"$FAKE/held"; echo "hold $*" >>"$FAKE/log" ;;
    unhold)
      shift
      grep -vxF -f <(printf '%s\n' "$@") "$FAKE/held" >"$FAKE/held.new" || true
      mv "$FAKE/held.new" "$FAKE/held"
      echo "unhold $*" >>"$FAKE/log"
      ;;
  esac
}
repo_setup() { echo "repo $*" >>"$FAKE/log"; }
sudo_run() { if [[ $1 == apt-mark ]]; then "$@"; else run "$@" >/dev/null; fi; }
"#;

/// A folder with the fixture manifest, the fakes' state, and the choices and record files.
struct Machine {
    dir: tempfile::TempDir,
}

impl Machine {
    fn new(installed: &[&str]) -> Self {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("packages.toml"), MANIFEST).unwrap();
        fs::write(
            dir.path().join("installed"),
            installed.iter().flat_map(|p| [*p, "\n"]).collect::<String>(),
        )
        .unwrap();
        Self { dir }
    }

    fn path(&self, name: &str) -> PathBuf {
        self.dir.path().join(name)
    }

    /// Runs `script` after `system/lib.sh` and the fakes, on `arch`.
    fn bash(&self, arch: &str, script: &str) -> String {
        let system = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../system");
        let out = Command::new("bash")
            .args(["-c", &format!("source \"$SYSTEM/lib.sh\"\n{FAKES}\n{script}")])
            .env("SYSTEM", &system)
            .env("FAKE", self.dir.path())
            .env("FAKE_ARCH", arch)
            .env("APT_MANIFEST", self.path("packages.toml"))
            .env("APT_CHOICES", self.path("apt-groups"))
            .env("APT_RECORD", self.path("apt-installed"))
            .env("SUDO_USER", "")
            .output()
            .expect("run bash");
        assert!(
            out.status.success(),
            "{script}: {}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8(out.stdout).unwrap()
    }

    fn read(&self, name: &str) -> String {
        fs::read_to_string(self.path(name)).unwrap_or_default()
    }

    fn record(&self) -> Record {
        Record::parse(&self.read("apt-installed"))
    }
}

fn lines(text: &str) -> Vec<&str> {
    text.lines().collect()
}

#[test]
fn group_install_installs_what_the_wanted_groups_lack_and_records_it_under_its_group() {
    let m = Machine::new(&["git"]);

    m.bash("amd64", "group_install tools docker piv chrome");

    assert_eq!(
        lines(&m.read("log")),
        ["repo docker", "install curl git docker-ce google-chrome-stable"]
    );
    let record = m.record();
    assert_eq!(record.group_of("curl"), Some("tools"));
    assert_eq!(record.group_of("docker-ce"), Some("docker"));
    assert!(!record.has("git"), "git was there before KubaOS");
    assert!(!record.has("pcscd"), "piv is off unless turned on");
}

#[test]
fn a_group_for_another_architecture_is_skipped_with_the_reason() {
    let m = Machine::new(&[]);

    let out = m.bash("arm64", "group_install chrome");

    assert!(out.contains("skip group chrome: amd64 only"), "{out}");
    assert_eq!(m.read("log"), "");
}

#[test]
fn a_choice_set_by_a_flag_is_kept_and_followed() {
    let m = Machine::new(&[]);

    m.bash(
        "amd64",
        "choice_set docker off; choice_set piv on; choice_set docker off",
    );
    let out = m.bash("amd64", "group_install docker piv");

    let choices = Choices::parse(&m.read("apt-groups"));
    assert_eq!(choices.get("docker"), Some(Switch::Off));
    assert_eq!(choices.get("piv"), Some(Switch::On));
    assert!(m.read("apt-groups").ends_with("docker=off\npiv=on\n"));
    assert!(out.contains("skip group docker: turned off"), "{out}");
    assert_eq!(lines(&m.read("log")), ["install pcscd"]);
}

#[test]
fn group_wanted_and_group_packages_follow_the_choices_and_the_architecture() {
    let m = Machine::new(&[]);
    fs::write(m.path("apt-groups"), "docker=off\n").unwrap();

    let out = m.bash(
        "arm64",
        "for g in tools docker piv chrome; do group_wanted $g && echo $g; done; group_packages tools docker",
    );

    assert_eq!(lines(&out), ["tools", "curl", "git"]);
}

#[test]
fn a_held_group_unholds_what_it_installs_and_holds_it_again() {
    let m = Machine::new(&[]);
    fs::write(m.path("held"), "hyprland\n").unwrap();

    let out = m.bash("arm64", "group_install hyprland");

    assert!(out.contains("no nope-guiutils for arm64"), "{out}");
    assert_eq!(
        lines(&m.read("log")),
        ["unhold hyprland", "install hyprland", "hold hyprland"]
    );
    assert_eq!(m.record().group_of("hyprland"), Some("hyprland"));
}

#[test]
fn the_record_keeps_what_it_had_and_each_package_once() {
    let m = Machine::new(&[]);
    fs::write(m.path("apt-installed"), "# old header\nzsh\tshell\n").unwrap();

    m.bash("amd64", "group_install tools; record_installed nix curl");

    let text = m.read("apt-installed");
    assert!(text.starts_with("# The apt packages KubaOS installed"));
    assert_eq!(
        text.lines().filter(|l| !l.starts_with('#')).collect::<Vec<_>>(),
        ["curl\ttools", "git\ttools", "zsh\tshell"]
    );
}

/// Programs that stand in for sudo, dpkg and apt on `PATH`, keeping their state in `$FAKE`.
const FAKE_PROGRAMS: [(&str, &str); 6] = [
    ("sudo", "exec \"$@\""),
    ("dpkg", "echo amd64"),
    (
        "dpkg-query",
        "for last; do :; done; grep -qx \"$last\" \"$FAKE/installed\" && echo installed || exit 1",
    ),
    (
        "apt-cache",
        "[[ $2 == nope-* ]] || printf '%s:\\n  Candidate: 1.0\\n' \"$2\"",
    ),
    (
        "apt-get",
        "echo \"apt-get $*\" >>\"$FAKE/log\"; [[ $1 == install ]] || exit 0; \
         for p in \"$@\"; do [[ $p == -* || $p == install ]] || echo \"$p\" >>\"$FAKE/installed\"; done",
    ),
    ("apt-mark", "true"),
];

#[test]
fn the_bridge_migration_keeps_what_the_install_left_out_and_installs_the_rest() {
    let m = Machine::new(&["curl", "pcscd"]);
    let bin = m.path("bin");
    fs::create_dir_all(&bin).unwrap();
    for (name, body) in FAKE_PROGRAMS {
        let path = bin.join(name);
        fs::write(&path, format!("#!/usr/bin/env bash\n{body}\n")).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    }
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let migration = fs::read_dir(root.join("migrations/system"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .find(|p| p.to_string_lossy().ends_with("-package-manifest.sh"))
        .expect("the package-manifest migration");
    let path = format!("{}:{}", bin.display(), std::env::var("PATH").unwrap());

    let out = Command::new("bash")
        .args(["-euo", "pipefail"])
        .arg(&migration)
        .env("PATH", path)
        .env("KUBAOS_DIR", &root)
        .env("FAKE", m.dir.path())
        .env("APT_MANIFEST", m.path("packages.toml"))
        .env("APT_CHOICES", m.path("apt-groups"))
        .env("APT_RECORD", m.path("apt-installed"))
        .output()
        .unwrap();

    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let choices = Choices::parse(&m.read("apt-groups"));
    assert_eq!(choices.get("docker"), Some(Switch::Off), "docker was left out");
    assert_eq!(choices.get("piv"), Some(Switch::On), "pcscd was installed");
    assert_eq!(
        m.read("log"),
        "apt-get install -y --no-install-recommends git google-chrome-stable hyprland\n",
        "pcscd and curl were there, docker is off, nope-guiutils has no candidate"
    );
    assert!(m.record().has("git") && m.record().has("hyprland"));
}

#[test]
fn a_dry_run_records_nothing() {
    let m = Machine::new(&[]);

    m.bash("amd64", "DRY_RUN=1; group_install tools");

    assert!(!Path::new(&m.path("apt-installed")).exists());
}
