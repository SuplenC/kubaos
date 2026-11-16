//! The Apps catalog's user helper, `apps/user-helper`, and the setups' user hooks,
//! `apps/user-hooks.sh`, copied into a checkout of their own with `system/manifest.sh` and run as
//! this user on a home in a temporary folder. curl, dpkg, pgrep, setsid, systemctl, the desktop
//! app (`t3code`) and T3's `t3` are stand-ins that write down how they were called; tar, jq and
//! sha256sum are the real ones.

use std::fs;
use std::os::unix::fs::PermissionsExt as _;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use tempfile::TempDir;

/// An app with a setup whose parts are stand-in hooks, and a setup without parts.
fn lab(sha: &str) -> String {
    format!(
        r#"
name = "Lab"
summary = "An app for agents"

[deb]
package = "lab"
version = "2.0"

[deb.amd64]
url = "https://example.com/lab.deb"
sha256 = "{sha}"

[setup.desktop]
name = "Lab"
summary = "The desktop app"

[setup.service]
name = "Lab as a service"
summary = "Its agents in a service"
user_hooks = "lab_service"
marker = ".config/systemd/user/lab.service"
version = "2.0"
self_updates = true

[setup.service.amd64]
url = "https://example.com/lab-cli-2.0.tar.gz"
sha256 = "{sha}"

[setup.missing]
name = "Lab elsewhere"
summary = "Hooks nobody wrote"
user_hooks = "nowhere"
marker = ".lab-elsewhere"
"#
    )
}

/// T3 Code's service setup as the catalog ships it, at a version of the stand-in runtime.
fn t3_code(sha: &str) -> String {
    format!(
        r#"
name = "T3 Code"
summary = "A desktop app for coding agents"

[deb]
package = "t3code"
version = "9.9.9"
self_updates = true

[deb.amd64]
url = "https://example.com/t3code.deb"
sha256 = "{sha}"

[setup.desktop]
name = "T3 Code"
summary = "The desktop app"

[setup.service]
name = "T3 Code as a service"
summary = "Agents in a service"
user_hooks = "t3_service"
marker = ".config/systemd/user/t3code.service"
version = "9.9.9"
self_updates = true

[setup.service.amd64]
url = "https://example.com/t3-9.9.9-linux-x64.tar.gz"
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

/// A checkout with the real helper and hooks, a home, and a machine of stand-ins.
struct Machine {
    dir: TempDir,
}

/// What a run of the helper did.
struct Run {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

impl Machine {
    /// The catalog of lab and t3-code, both pinned to the archive the curl stand-in downloads
    /// unless `pinned` names another sha256.
    fn new(pinned: Option<&str>) -> Self {
        let m = Self {
            dir: tempfile::tempdir().expect("a temp dir"),
        };
        let root = m.dir.path();
        for dir in ["checkout/apps", "checkout/system", "bin", "home", "runtime/t3-9.9.9-linux-x64"] {
            fs::create_dir_all(root.join(dir)).expect("make the folders");
        }
        fs::copy(repo_root().join("apps/user-helper"), root.join("checkout/apps/user-helper")).expect("copy the helper");
        fs::copy(
            repo_root().join("system/manifest.sh"),
            root.join("checkout/system/manifest.sh"),
        )
        .expect("copy manifest.sh");
        let log = m.log_path();
        let log = log.display();
        let hooks = fs::read_to_string(repo_root().join("apps/user-hooks.sh")).expect("read the hooks");
        let stand_ins = format!(
            "\ninstall_lab_service() {{ echo \"install_lab_service $*\" >>'{log}'; }}\n\
             remove_lab_service() {{ echo \"remove_lab_service $*\" >>'{log}'; }}\n"
        );
        fs::write(root.join("checkout/apps/user-hooks.sh"), hooks + &stand_ins).expect("write the hooks");

        let line = |name: &str| format!("echo \"{name} $*\" >>'{log}'");
        executable(
            &root.join("runtime/t3-9.9.9-linux-x64/t3"),
            &format!(
                "{}\nhome=\"${{T3CODE_HOME:-$HOME/.t3}}\"\n\
                 case \"$*\" in\n\
                 --version) echo 't3 v9.9.9' ;;\n\
                 install) mkdir -p \"$HOME/.local/bin\" && echo \"exec $0\" >\"$HOME/.local/bin/t3\" ;;\n\
                 'service install') mkdir -p \"$HOME/.config/systemd/user\" \"$home/userdata\"\n\
                   touch \"$HOME/.config/systemd/user/t3code.service\"\n\
                   echo '{{\"version\":1,\"pid\":{pid},\"port\":3773,\"origin\":\"http://127.0.0.1:3773\",\"serviceManaged\":true}}' \
                   >\"$home/userdata/server-runtime.json\" ;;\n\
                 'auth pairing create'*) echo '{{\"id\":\"1\",\"credential\":\"abc\",\"pairUrl\":\"http://127.0.0.1:3773/pair#token=abc\"}}' ;;\n\
                 'uninstall --yes') rm -rf \"$HOME/.config/systemd/user/t3code.service\" \"$HOME/.local/bin/t3\" \"$home/runtime\" ;;\n\
                 esac",
                line("t3"),
                pid = std::process::id()
            ),
        );
        let archive = root.join("archive.tar.gz");
        let tar = Command::new("tar")
            .arg("-czf")
            .arg(&archive)
            .arg("-C")
            .arg(root.join("runtime"))
            .arg("t3-9.9.9-linux-x64")
            .status()
            .expect("run tar");
        assert!(tar.success());
        let sha = pinned.map_or_else(|| sha256(&archive), str::to_owned);
        fs::write(root.join("checkout/apps/lab.toml"), lab(&sha)).expect("write lab");
        fs::write(root.join("checkout/apps/t3-code.toml"), t3_code(&sha)).expect("write t3-code");

        executable(
            &root.join("bin/curl"),
            &format!(
                "{}\nwhile (($#)); do [[ $1 == -o ]] && out=\"$2\"; shift; done\ncp '{}' \"$out\"",
                line("curl"),
                archive.display()
            ),
        );
        executable(&root.join("bin/dpkg"), "echo amd64");
        executable(
            &root.join("bin/pgrep"),
            &format!("{}\n[[ -e '{}' ]]", line("pgrep"), root.join("t3code-runs").display()),
        );
        executable(&root.join("bin/t3code"), &line("t3code"));
        executable(&root.join("bin/setsid"), "[[ $1 == -f ]] && shift\nexec \"$@\"");
        executable(&root.join("bin/systemctl"), &line("systemctl"));
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

    fn home(&self) -> PathBuf {
        self.dir.path().join("home")
    }

    /// Runs the helper with `args`, in a graphical session when `wayland`.
    fn run(&self, args: &[&str], wayland: bool) -> Run {
        let path = format!(
            "{}:{}",
            self.dir.path().join("bin").display(),
            std::env::var("PATH").unwrap_or_default()
        );
        let mut cmd = Command::new(self.dir.path().join("checkout/apps/user-helper"));
        cmd.args(args)
            .env("PATH", path)
            .env("HOME", self.home())
            .env("TMPDIR", self.dir.path())
            .env_remove("T3CODE_HOME")
            .env_remove("XDG_CONFIG_HOME");
        if wayland {
            cmd.env("WAYLAND_DISPLAY", "wayland-1");
        } else {
            cmd.env_remove("WAYLAND_DISPLAY");
        }
        let out = cmd.output().expect("run the helper");
        Run {
            code: out.status.code(),
            stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        }
    }

    fn desktop_settings(&self) -> serde_json::Value {
        let text = fs::read_to_string(self.home().join(".t3/userdata/desktop-settings.json")).expect("read the settings");
        serde_json::from_str(&text).expect("the settings are JSON")
    }
}

fn sha256(path: &Path) -> String {
    let out = Command::new("sha256sum").arg(path).output().expect("run sha256sum");
    String::from_utf8_lossy(&out.stdout)
        .split_whitespace()
        .next()
        .expect("a sum")
        .to_owned()
}

#[test]
fn it_refuses_anything_but_a_setup_of_the_catalog_with_parts_and_hooks() {
    let m = Machine::new(None);

    for args in [
        ["install", "../lab", "service"],
        ["install", "lab", "--force"],
        ["install", "gone", "service"],
        ["install", "lab", "server"],
        ["install", "lab", "desktop"],
        ["install", "lab", "missing"],
        ["update", "lab", "service"],
    ] {
        let run = m.run(&args, false);

        assert_eq!(run.code, Some(2), "{args:?}: {}", run.stderr);
    }
    assert!(m.log().is_empty(), "{:?}", m.log());
}

#[test]
fn install_hands_the_hook_the_pinned_download_once_its_sha256_is_the_manifests() {
    let m = Machine::new(None);

    let run = m.run(&["install", "lab", "service"], false);

    assert_eq!(run.code, Some(0), "{}", run.stderr);
    let log = m.log();
    assert!(log[0].starts_with("curl ") && log[0].contains("--proto =https"), "{log:?}");
    assert!(log[0].ends_with("https://example.com/lab-cli-2.0.tar.gz"), "{log:?}");
    assert!(
        log[1].starts_with("install_lab_service …/") && log[1].ends_with("/download 2.0"),
        "{log:?}"
    );
}

#[test]
fn a_download_that_isnt_the_pinned_one_never_reaches_the_hook() {
    let m = Machine::new(Some(&"0".repeat(64)));

    let run = m.run(&["install", "lab", "service"], false);

    assert_eq!(run.code, Some(1));
    assert!(run.stderr.contains("is not the download its manifest pins"), "{}", run.stderr);
    assert!(!m.log().iter().any(|l| l.starts_with("install_lab_service")), "{:?}", m.log());
}

#[test]
fn remove_runs_the_setups_remove_hook() {
    let m = Machine::new(None);

    let run = m.run(&["remove", "lab", "service"], false);

    assert_eq!(run.code, Some(0), "{}", run.stderr);
    assert_eq!(m.log(), ["remove_lab_service "]);
}

#[test]
fn t3_codes_service_is_t3s_runtime_its_service_and_the_desktop_app_paired_to_it() {
    let m = Machine::new(None);
    let settings = m.home().join(".t3/userdata/desktop-settings.json");
    fs::create_dir_all(settings.parent().unwrap()).unwrap();
    fs::write(&settings, "{\"mainWindowMaximized\": true}").unwrap();

    let run = m.run(&["install", "t3-code", "service"], true);

    assert_eq!(run.code, Some(0), "{}", run.stderr);
    let runtime = m.home().join(".t3/runtime/versions/9.9.9");
    assert_eq!(fs::read_to_string(runtime.join(".install-complete")).unwrap(), "9.9.9\n");
    assert!(runtime.join("t3").is_file());
    let t3: Vec<String> = m.log().into_iter().filter(|l| l.starts_with("t3 ")).collect();
    assert_eq!(
        t3,
        [
            "t3 --version",
            "t3 install",
            "t3 service install",
            "t3 auth pairing create --json --ttl 15m --label T3 Code on this machine --base-url http://127.0.0.1:3773",
        ]
    );
    assert!(m.home().join(".config/systemd/user/t3code.service").exists());
    assert_eq!(
        m.desktop_settings(),
        serde_json::json!({"mainWindowMaximized": true, "localEnvironmentEnabled": false})
    );
    assert!(
        m.log()
            .contains(&"t3code t3code://pair?pairingUrl=http%3A%2F%2F127.0.0.1%3A3773%2Fpair%23token%3Dabc".to_owned()),
        "{:?}",
        m.log()
    );
    assert!(run.stdout.contains("asks to add the backend on this machine"), "{}", run.stdout);
}

#[test]
fn without_a_graphical_session_the_service_is_set_up_and_the_pairing_is_left_to_the_user() {
    let m = Machine::new(None);

    let run = m.run(&["install", "t3-code", "service"], false);

    assert_eq!(run.code, Some(0), "{}", run.stderr);
    assert!(!m.log().iter().any(|l| l.starts_with("t3code ")), "{:?}", m.log());
    assert!(run.stdout.contains("run `t3 pair`"), "{}", run.stdout);
    assert_eq!(m.desktop_settings(), serde_json::json!({"localEnvironmentEnabled": false}));
}

#[test]
fn t3_codes_service_waits_for_the_desktop_app_to_quit() {
    let m = Machine::new(None);
    fs::write(m.dir.path().join("t3code-runs"), "").unwrap();

    let run = m.run(&["install", "t3-code", "service"], true);

    assert_eq!(run.code, Some(1));
    assert!(run.stderr.contains("T3 Code is open: quit it"), "{}", run.stderr);
    assert!(!m.home().join(".t3/runtime").exists());
    assert!(!m.log().iter().any(|l| l.starts_with("t3 ")), "{:?}", m.log());
}

#[test]
fn removing_t3_codes_service_is_t3s_uninstall_and_gives_the_desktop_app_its_server_back() {
    let m = Machine::new(None);
    assert_eq!(m.run(&["install", "t3-code", "service"], false).code, Some(0));
    let settings = m.home().join(".t3/userdata/desktop-settings.json");
    fs::write(&settings, "{\"localEnvironmentEnabled\": false, \"mainWindowMaximized\": true}").unwrap();
    fs::write(m.log_path(), "").unwrap();

    let run = m.run(&["remove", "t3-code", "service"], false);

    assert_eq!(run.code, Some(0), "{}", run.stderr);
    assert_eq!(m.log(), ["t3 uninstall --yes"]);
    assert!(!m.home().join(".config/systemd/user/t3code.service").exists());
    assert_eq!(m.desktop_settings(), serde_json::json!({"mainWindowMaximized": true}));
}

#[test]
fn removing_a_service_whose_runtime_is_gone_stops_and_removes_its_unit() {
    let m = Machine::new(None);
    let unit = m.home().join(".config/systemd/user/t3code.service");
    fs::create_dir_all(unit.parent().unwrap()).unwrap();
    fs::write(&unit, "").unwrap();

    let run = m.run(&["remove", "t3-code", "service"], false);

    assert_eq!(run.code, Some(0), "{}", run.stderr);
    assert!(!unit.exists());
    assert_eq!(
        m.log(),
        [
            "systemctl --user disable --now t3code.service",
            "systemctl --user daemon-reload"
        ]
    );
}
