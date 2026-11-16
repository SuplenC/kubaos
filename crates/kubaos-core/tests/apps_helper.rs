//! The Apps catalog's root helper, `system/files/apps/kubaos-apps`, run as this user on a fake
//! checkout (`KUBAOS_APPS_CHECKOUT`): sudo, dpkg, dpkg-query, apt-get, apt-cache and curl are
//! stand-ins that write down how they were called, and apt's packages are a file. The checkout's
//! `system/lib.sh` and `system/manifest.sh` are the repo's own; its `apt-repo.sh` is a stand-in.

use std::fs;
use std::os::unix::fs::PermissionsExt as _;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use tempfile::TempDir;

/// The sha256 of what the curl stand-in downloads: `desk deb` and a newline.
const DESK_SHA: &str = "5c592f3fd27f92f3b28fc8040c77bc558d1f022708768cf36ee4787d47fe9d9e";

const AGENT: &str = r#"
name = "Agent"
summary = "A coding agent"

[apt]
repo = "vendor"
packages = ["agent", "agent-data"]
"#;

const EDITOR: &str = r#"
name = "Editor"
summary = "An editor"

[nix]
packages = ["editor"]
"#;

fn desk(sha: &str) -> String {
    format!(
        r#"
name = "Desk"
summary = "A desktop app"

[deb]
package = "desk"
version = "1.2.0"

[deb.amd64]
url = "https://example.com/desk-1.2.0-amd64.deb"
sha256 = "{sha}"
"#
    )
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn executable(path: &Path, body: &str) {
    fs::write(path, format!("#!/usr/bin/env bash\n{body}\n")).expect("write a stand-in");
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("make it run");
}

/// A checkout with a catalog, and a machine of stand-ins.
struct Machine {
    dir: TempDir,
}

/// What a run of the helper did.
struct Run {
    code: Option<i32>,
    stderr: String,
}

impl Machine {
    /// The catalog of agent (apt, from a vendor repo), desk (a pinned .deb, its sha `desk_sha`)
    /// and editor (nixpkgs), and a machine with `installed` (`name version` each).
    fn new(desk_sha: &str, installed: &[&str]) -> Self {
        let m = Self {
            dir: tempfile::tempdir().expect("a temp dir"),
        };
        let root = m.dir.path();
        for dir in ["checkout/apps", "checkout/system", "bin"] {
            fs::create_dir_all(root.join(dir)).expect("make the folders");
        }
        for (id, text) in [("agent", AGENT), ("desk", &desk(desk_sha)), ("editor", EDITOR)] {
            fs::write(root.join(format!("checkout/apps/{id}.toml")), text).expect("write a manifest");
        }
        for script in ["lib.sh", "manifest.sh"] {
            fs::copy(
                repo_root().join("system").join(script),
                root.join("checkout/system").join(script),
            )
            .expect("copy the system scripts");
        }
        fs::write(root.join("checkout/system/packages.toml"), "").expect("write the package manifest");
        let log = m.log_path();
        let log = log.display();
        let state = root.join("dpkg");
        let state = state.display();
        let line = |name: &str| format!("echo \"{name} $*\" >>'{log}'");
        executable(&root.join("checkout/system/apt-repo.sh"), &line("apt-repo.sh"));
        executable(&root.join("bin/sudo"), "exec \"$@\"");
        executable(&root.join("bin/dpkg"), "echo amd64");
        executable(
            &root.join("bin/dpkg-query"),
            &format!(
                "p=\"${{@: -1}}\"\n\
                 v=\"$(awk -v p=\"$p\" '$1 == p {{ print $2 }}' '{state}')\"\n\
                 [[ -n $v ]] || exit 1\n\
                 case \"$*\" in *Status-Status*) printf installed ;; *) printf '%s' \"$v\" ;; esac"
            ),
        );
        executable(
            &root.join("bin/apt-cache"),
            "for p in \"${@:2}\"; do printf '%s:\\n  Candidate: 1.0\\n' \"$p\"; done",
        );
        executable(
            &root.join("bin/apt-get"),
            &format!(
                "{}\n\
                 for a in \"$@\"; do\n\
                 case \"$a\" in\n\
                 -* | install | remove | Dpkg::*) ;;\n\
                 *.deb) [[ $1 == install ]] && echo \"$(basename \"$a\" .deb) 1.2.0\" >>'{state}' ;;\n\
                 *) if [[ $1 == install ]]; then echo \"$a 1.0\" >>'{state}';\n\
                 else sed -i \"/^$a /d\" '{state}'; fi ;;\n\
                 esac\n\
                 done",
                line("apt-get")
            ),
        );
        executable(
            &root.join("bin/curl"),
            &format!(
                "{}\n\
                 while (($#)); do [[ $1 == -o ]] && out=\"$2\"; shift; done\n\
                 printf 'desk deb\\n' >\"$out\"",
                line("curl")
            ),
        );
        // The helper refuses a catalog or a script others may write, whatever the umask made.
        for dir in ["checkout", "checkout/apps", "checkout/system"] {
            for entry in fs::read_dir(root.join(dir)).expect("list the checkout") {
                let path = entry.expect("an entry").path();
                let mode = if path.is_dir() || path.ends_with("apt-repo.sh") {
                    0o755
                } else {
                    0o644
                };
                fs::set_permissions(&path, fs::Permissions::from_mode(mode)).expect("set its mode");
            }
        }
        let packages: String = installed.iter().flat_map(|p| [*p, "\n"]).collect();
        fs::write(root.join("dpkg"), packages).expect("write the machine's packages");
        fs::write(m.log_path(), "").expect("start the log");
        m
    }

    fn log_path(&self) -> PathBuf {
        self.dir.path().join("log")
    }

    /// The stand-ins' lines, with the temporary folder as `…`.
    fn log(&self) -> Vec<String> {
        let dir = self.dir.path().display().to_string();
        fs::read_to_string(self.log_path())
            .expect("read the log")
            .lines()
            .map(|line| line.replace(&dir, "…"))
            .collect()
    }

    fn installed(&self) -> String {
        fs::read_to_string(self.dir.path().join("dpkg")).expect("read the machine's packages")
    }

    fn run(&self, args: &[&str]) -> Run {
        let path = format!(
            "{}:{}",
            self.dir.path().join("bin").display(),
            std::env::var("PATH").unwrap_or_default()
        );
        let out = Command::new(repo_root().join("system/files/apps/kubaos-apps"))
            .args(args)
            .env("PATH", path)
            .env("KUBAOS_APPS_CHECKOUT", self.dir.path().join("checkout"))
            .env("TMPDIR", self.dir.path())
            .output()
            .expect("run the helper");
        Run {
            code: out.status.code(),
            stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        }
    }
}

#[test]
fn it_refuses_anything_but_an_id_of_the_catalog_and_changes_nothing() {
    let m = Machine::new(DESK_SHA, &[]);

    for (args, why) in [
        (vec!["install", "../../etc/passwd"], "is not an app id"),
        (vec!["install", "/usr/share/kubaos/apps/agent.toml"], "is not an app id"),
        (vec!["install", "agent.toml"], "is not an app id"),
        (vec!["install", "--force"], "is not an app id"),
        (vec!["install", "agent;reboot"], "is not an app id"),
        (vec!["install", "Agent"], "is not an app id"),
        (vec!["install", "agent-data"], "the catalog has no app agent-data"),
        (vec!["install", "curl"], "the catalog has no app curl"),
        (vec!["remove", "agent", "https://example.com/x.deb"], "is not an app id"),
        (vec!["install", "agent", "editor"], "editor comes from nixpkgs"),
    ] {
        let run = m.run(&args);

        assert_eq!(run.code, Some(2), "{args:?}: {}", run.stderr);
        assert!(run.stderr.contains(why), "{args:?}: {}", run.stderr);
    }
    assert!(m.log().is_empty(), "something ran: {:?}", m.log());
    assert_eq!(m.run(&["upgrade", "agent"]).code, Some(2));
    assert_eq!(m.run(&["install"]).code, Some(2));
}

#[test]
fn an_apt_app_whose_repo_isnt_a_name_is_refused_before_anything_runs() {
    let m = Machine::new(DESK_SHA, &[]);
    fs::write(
        m.dir.path().join("checkout/apps/agent.toml"),
        AGENT.replace("repo = \"vendor\"", "repo = \"--dry-run\""),
    )
    .expect("write a manifest");

    let run = m.run(&["install", "agent"]);

    assert_eq!(run.code, Some(2), "{}", run.stderr);
    assert!(run.stderr.contains("not a repository name"), "{}", run.stderr);
    assert!(m.log().is_empty(), "something ran: {:?}", m.log());
}

#[test]
fn a_catalog_or_a_script_others_may_write_is_never_run() {
    for path in [
        "checkout/system/lib.sh",
        "checkout/system",
        "checkout/apps/agent.toml",
        "checkout/apps",
    ] {
        let m = Machine::new(DESK_SHA, &[]);
        let path = m.dir.path().join(path);
        let mode = fs::metadata(&path).expect("read its mode").permissions().mode();
        fs::set_permissions(&path, fs::Permissions::from_mode(mode | 0o020)).expect("let the group write it");

        let run = m.run(&["install", "agent"]);

        assert_eq!(run.code, Some(1), "{}: {}", path.display(), run.stderr);
        assert!(run.stderr.contains("is writable by others"), "{}", run.stderr);
        assert!(m.log().is_empty(), "something ran: {:?}", m.log());
    }
}

#[test]
fn it_refuses_to_run_as_a_user_without_a_test_checkout() {
    if running_as_root() {
        return;
    }

    let out = Command::new(repo_root().join("system/files/apps/kubaos-apps"))
        .args(["install", "claude-code"])
        .env_remove("KUBAOS_APPS_CHECKOUT")
        .output()
        .expect("run the helper");

    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("runs as root"));
}

/// Whether the tests run as root, where the helper takes the real catalog instead.
fn running_as_root() -> bool {
    Command::new("id")
        .arg("-u")
        .output()
        .is_ok_and(|out| out.stdout.trim_ascii() == b"0")
}

#[test]
fn install_sets_up_the_repo_then_installs_the_apt_packages_and_the_checked_deb() {
    let m = Machine::new(DESK_SHA, &[]);

    let run = m.run(&["install", "agent", "desk"]);

    assert_eq!(run.code, Some(0), "{}", run.stderr);
    let log = m.log();
    assert_eq!(log[0], "apt-repo.sh vendor");
    assert!(
        log[1].starts_with("apt-get install -y --no-install-recommends agent agent-data"),
        "{log:?}"
    );
    assert!(
        log[2].starts_with("curl -fsSL --proto =https --tlsv1.2 --retry 2 -o …/")
            && log[2].ends_with("/desk.deb https://example.com/desk-1.2.0-amd64.deb"),
        "{log:?}"
    );
    assert!(
        log[3].starts_with("apt-get install -y --no-install-recommends") && log[3].ends_with("/desk.deb"),
        "{log:?}"
    );
    assert_eq!(log.len(), 4, "{log:?}");
    assert_eq!(m.installed(), "agent 1.0\nagent-data 1.0\ndesk 1.2.0\n");

    let again = m.run(&["install", "agent", "desk"]);

    assert_eq!(again.code, Some(0), "{}", again.stderr);
    assert_eq!(m.log()[4..], ["apt-repo.sh vendor"], "only the repo is looked at again");
}

#[test]
fn a_deb_whose_sha256_isnt_the_pin_is_never_installed() {
    let m = Machine::new(&"0".repeat(64), &[]);

    let run = m.run(&["install", "desk"]);

    assert_eq!(run.code, Some(1));
    assert!(
        run.stderr.contains("not the download its manifest pins"),
        "{}",
        run.stderr
    );
    assert!(m.log().iter().all(|line| !line.starts_with("apt-get")), "{:?}", m.log());
    assert_eq!(m.installed(), "");
}

#[test]
fn a_deb_at_another_version_than_its_pin_is_installed_again() {
    let m = Machine::new(DESK_SHA, &["desk 1.1.0"]);

    let run = m.run(&["install", "desk"]);

    assert_eq!(run.code, Some(0), "{}", run.stderr);
    assert!(m.log().iter().any(|line| line.starts_with("curl")), "{:?}", m.log());
}

#[test]
fn remove_takes_the_apps_installed_packages_and_nothing_else() {
    let m = Machine::new(DESK_SHA, &["agent 1.0", "desk 1.2.0", "vim 9.1"]);

    let run = m.run(&["remove", "agent", "desk"]);

    assert_eq!(run.code, Some(0), "{}", run.stderr);
    assert_eq!(
        m.log(),
        ["apt-get remove -y -o Dpkg::Options::=--force-confdef -o Dpkg::Options::=--force-confold agent desk"]
    );
    assert_eq!(m.installed(), "vim 9.1\n");
}
