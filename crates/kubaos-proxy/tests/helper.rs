//! The root helper, `system/files/proxy/kubaos-proxy-write`, on a fake `/`, run as itself and as
//! `kubaos-proxy-pin`: systemctl, nft, update-ca-certificates and wg are stand-ins that write down
//! how they were called (and fail while a marker file says so), openssl is the machine's own.

use std::fs;
use std::io::Write as _;
use std::os::unix::fs::PermissionsExt as _;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;

use tempfile::TempDir;

const PROXY: &str = "http://proxy.corp.example.com:3128";
const NO_PROXY: &str = "localhost,127.0.0.1,::1,.corp.example.com";
const PAC: &str = "http://wpad.corp.example.com/proxy.pac?v=2";

struct Root {
    dir: TempDir,
    bin: TempDir,
}

/// What a run of the helper did.
struct Run {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

fn script() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../system/files/proxy/kubaos-proxy-write")
}

fn stub(bin: &Path, name: &str, body: &str) {
    let path = bin.join(name);
    fs::write(&path, format!("#!/usr/bin/env bash\n{body}\n")).expect("write a stub");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).expect("make it run");
}

impl Root {
    /// A machine with Docker and Nix's daemons installed, nothing of the Proxy, and the proxy,
    /// its `no_proxy` and the PAC file pinned, but no CA.
    fn new() -> Root {
        let root = Root {
            dir: tempfile::tempdir().expect("a temp dir"),
            bin: tempfile::tempdir().expect("a temp dir"),
        };
        let log = root.log_path();
        let log = log.display();
        let dir = root.dir.path().display();
        let tables = root.dir.path().join("nft-tables");
        let tables = tables.display();
        stub(
            root.bin.path(),
            "systemctl",
            &format!(
                "echo \"systemctl $*\" >>'{log}'\n\
                 [[ $1 == try-restart && -e '{dir}/fail-restart' ]] && exit 1\n\
                 exit 0"
            ),
        );
        stub(
            root.bin.path(),
            "update-ca-certificates",
            &format!("echo \"update-ca-certificates $*\" >>'{log}'\n[[ ! -e '{dir}/fail-ca' ]]"),
        );
        stub(
            root.bin.path(),
            "nft",
            &format!(
                "echo \"nft $*\" >>'{log}'\n\
                 case \"$1\" in\n\
                   -f) cat >'{tables}' ;;\n\
                   list) [[ -s '{tables}' ]] ;;\n\
                   delete) : >'{tables}' ;;\n\
                 esac"
            ),
        );
        stub(
            root.bin.path(),
            "wg",
            "printf 'xTIBA5rboUvnH4htodjb6e697QjLERt1NAB4mZqp8Dg=\\t1700000000\\n'",
        );
        fs::copy(script(), root.bin.path().join("kubaos-proxy-pin")).expect("the pin helper");
        root.write("usr/lib/systemd/system/docker.service", "[Service]\n");
        root.write("etc/systemd/system/nix-daemon.service", "[Service]\n");
        let pinned = root.pin(
            &[
                "ca=none",
                &format!("proxy={PROXY}"),
                &format!("no-proxy={NO_PROXY}"),
                &format!("pac={PAC}"),
            ],
            "",
        );
        assert_eq!(pinned.code, Some(0), "{}", pinned.stderr);
        root
    }

    /// The same pins and `cert`'s fingerprint.
    fn pin_ca(&self, cert: &str) {
        let run = self.pin(
            &[
                "ca=-",
                &format!("proxy={PROXY}"),
                &format!("no-proxy={NO_PROXY}"),
                &format!("pac={PAC}"),
            ],
            cert,
        );
        assert_eq!(run.code, Some(0), "{}", run.stderr);
    }

    fn log_path(&self) -> PathBuf {
        self.dir.path().join("calls")
    }

    fn log(&self) -> String {
        fs::read_to_string(self.log_path()).unwrap_or_default()
    }

    fn tables(&self) -> String {
        fs::read_to_string(self.dir.path().join("nft-tables")).unwrap_or_default()
    }

    fn path(&self, path: &str) -> PathBuf {
        self.dir.path().join(path)
    }

    fn write(&self, path: &str, text: &str) {
        let path = self.path(path);
        fs::create_dir_all(path.parent().expect("a parent")).expect("make the folder");
        fs::write(path, text).expect("write the file");
    }

    fn read(&self, path: &str) -> String {
        fs::read_to_string(self.path(path)).unwrap_or_default()
    }

    fn exists(&self, path: &str) -> bool {
        self.path(path).exists()
    }

    fn command(&self, program: &Path, args: &[&str]) -> Command {
        let path = format!(
            "{}:{}",
            self.bin.path().display(),
            std::env::var("PATH").unwrap_or_default()
        );
        let mut command = Command::new("bash");
        command
            .arg(program)
            .args(args)
            .env("KUBAOS_PROXY_ROOT", self.dir.path())
            .env("PATH", path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        command
    }

    fn finish(mut child: std::process::Child, stdin: &str) -> Run {
        let _ = child.stdin.take().expect("stdin").write_all(stdin.as_bytes());
        let out = child.wait_with_output().expect("the helper's output");
        Run {
            code: out.status.code(),
            stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        }
    }

    /// Runs the write helper as pkexec would, with `stdin` on its stdin.
    fn run(&self, args: &[&str], stdin: &str) -> Run {
        Root::finish(self.command(&script(), args).spawn().expect("run the helper"), stdin)
    }

    /// Runs the pin helper, the same script by its other name.
    fn pin(&self, args: &[&str], stdin: &str) -> Run {
        let pin = self.bin.path().join("kubaos-proxy-pin");
        Root::finish(self.command(&pin, args).spawn().expect("run the pin helper"), stdin)
    }
}

/// A throwaway self-signed certificate, from the machine's openssl.
fn certificate() -> String {
    let out = Command::new("openssl")
        .args([
            "req",
            "-x509",
            "-newkey",
            "ec",
            "-pkeyopt",
            "ec_paramgen_curve:prime256v1",
        ])
        .args([
            "-nodes",
            "-keyout",
            "/dev/null",
            "-subj",
            "/CN=KubaOS test CA",
            "-days",
            "1",
        ])
        .output()
        .expect("run openssl");
    assert!(
        out.status.success(),
        "openssl: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).expect("PEM is text")
}

fn every_part() -> Vec<String> {
    vec![
        "killswitch=on".to_owned(),
        "ca=-".to_owned(),
        format!("apt={PROXY}"),
        format!("docker={PROXY}|{NO_PROXY}"),
        format!("nix={PROXY}|{NO_PROXY}"),
        format!("chrome=proxy|{PROXY}|{NO_PROXY}"),
    ]
}

fn strs(args: &[String]) -> Vec<&str> {
    args.iter().map(String::as_str).collect()
}

/// Nothing of the Proxy is anywhere on the fake `/`.
fn nothing_written(root: &Root) {
    for path in [
        "etc/apt/apt.conf.d/80kubaos-proxy",
        "etc/systemd/system/docker.service.d/kubaos-proxy.conf",
        "etc/systemd/system/nix-daemon.service.d/kubaos-proxy.conf",
        "usr/local/share/ca-certificates/kubaos-proxy/company.crt",
        "etc/opt/chrome/policies/managed/kubaos-proxy.json",
        "etc/opt/chrome/policies/managed/kubaos-proxy-ca.json",
    ] {
        assert!(!root.exists(path), "{path} is there");
    }
    assert!(root.tables().trim().is_empty(), "{}", root.tables());
}

#[test]
fn every_pinned_part_goes_in_place_and_each_is_named_done() {
    let root = Root::new();
    let args = every_part();
    let cert = certificate();
    root.pin_ca(&cert);

    let run = root.run(&strs(&args), &cert);

    assert_eq!(run.code, Some(0), "{}", run.stderr);
    let done: Vec<String> = args.iter().map(|arg| format!("done {arg}\n")).collect();
    assert_eq!(run.stdout, done.concat());
    assert_eq!(
        root.read("etc/apt/apt.conf.d/80kubaos-proxy")
            .lines()
            .skip(1)
            .collect::<Vec<_>>(),
        [
            format!("Acquire::http::Proxy \"{PROXY}\";"),
            format!("Acquire::https::Proxy \"{PROXY}\";")
        ]
    );
    let docker = root.read("etc/systemd/system/docker.service.d/kubaos-proxy.conf");
    assert!(
        docker.contains(&format!("\"HTTPS_PROXY={PROXY}\" \"NO_PROXY={NO_PROXY}\"")),
        "{docker}"
    );
    assert!(root.exists("etc/systemd/system/nix-daemon.service.d/kubaos-proxy.conf"));
    let log = root.log();
    assert_eq!(
        log.matches("systemctl try-restart docker.service\n").count(),
        1,
        "{log}"
    );
    assert!(log.contains("systemctl try-restart nix-daemon.service\n"), "{log}");
    assert!(log.contains("update-ca-certificates \n"), "{log}");
    assert_eq!(
        root.read("usr/local/share/ca-certificates/kubaos-proxy/company.crt"),
        cert
    );
    let policy: serde_json::Value =
        serde_json::from_str(&root.read("etc/opt/chrome/policies/managed/kubaos-proxy.json")).expect("JSON");
    assert_eq!(policy["ProxySettings"]["ProxyServer"], PROXY);
    let ca: serde_json::Value =
        serde_json::from_str(&root.read("etc/opt/chrome/policies/managed/kubaos-proxy-ca.json")).expect("JSON");
    assert!(ca["CACertificates"][0].as_str().is_some_and(|der| der.len() > 100));
    let tables = root.tables();
    assert!(tables.contains("table inet kubaos_proxy {"), "{tables}");
    assert!(tables.contains("oifname \"wg-kubaos\" accept"), "{tables}");
    assert!(tables.contains("meta mark 0x6b6f accept"), "{tables}");
}

#[test]
fn the_kill_switch_lets_no_lookup_out_beside_the_tunnel() {
    let root = Root::new();

    let run = root.run(&["killswitch=on"], "");

    assert_eq!(run.code, Some(0), "{}", run.stderr);
    let tables = root.tables();
    assert!(!tables.contains("skuid"), "{tables}");
    assert!(!tables.contains("dport { 53"), "{tables}");
}

/// The output chain's rules, one a line, trimmed.
fn output_rules(tables: &str) -> Vec<String> {
    let chain = tables.split("chain output {").nth(1).unwrap_or_default();
    let chain = chain.split("chain forward {").next().unwrap_or_default();
    chain
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && *line != "}")
        .map(str::to_owned)
        .collect()
}

#[test]
fn the_strict_kill_switch_lets_no_local_network_through() {
    let root = Root::new();

    let run = root.run(&["killswitch=on"], "");

    assert_eq!(run.code, Some(0), "{}", run.stderr);
    let rules = output_rules(&root.tables());
    assert!(!rules.iter().any(|rule| rule.contains("daddr")), "{rules:#?}");
    assert_eq!(
        rules.last().map(String::as_str),
        Some("reject with icmpx admin-prohibited")
    );
}

#[test]
fn the_kill_switch_lets_the_local_network_through_but_no_lookup() {
    let root = Root::new();

    let run = root.run(&["killswitch=lan|192.168.1.0/24,2a02:a311:1:2::/64"], "");

    assert_eq!(run.code, Some(0), "{}", run.stderr);
    assert_eq!(run.stdout, "done killswitch=lan|192.168.1.0/24,2a02:a311:1:2::/64\n");
    let rules = output_rules(&root.tables());
    let at = |rule: &str| {
        rules
            .iter()
            .position(|r| r == rule)
            .unwrap_or_else(|| panic!("no `{rule}` in {rules:#?}"))
    };
    let lookups = at("udp dport { 53, 853 } reject with icmpx admin-prohibited");
    assert!(at("tcp dport { 53, 853 } reject with tcp reset") < lookups);
    for rule in [
        "ip daddr { 169.254.0.0/16, 224.0.0.0/24, 239.255.255.250, 255.255.255.255 } accept",
        "ip6 daddr { fe80::/10, ff02::/16 } accept",
        "ip daddr 192.168.1.0/24 accept",
        "ip6 daddr 2a02:a311:1:2::/64 accept",
    ] {
        assert!(lookups < at(rule), "{rule} comes before the lookups are refused");
        assert!(at(rule) < at("meta l4proto tcp reject with tcp reset"));
    }
    assert!(at("oifname \"wg-kubaos\" accept") < lookups);
    assert!(
        !rules
            .iter()
            .any(|rule| rule.contains("dport") && rule.contains("53") && rule.contains("accept")),
        "{rules:#?}"
    );
}

#[test]
fn with_no_range_the_kill_switch_lets_only_link_local_and_discovery_through() {
    let root = Root::new();

    let run = root.run(&["killswitch=lan|"], "");

    assert_eq!(run.code, Some(0), "{}", run.stderr);
    let rules = output_rules(&root.tables());
    let through: Vec<&String> = rules.iter().filter(|rule| rule.contains("daddr")).collect();
    assert_eq!(
        through,
        [
            "ip daddr { 169.254.0.0/16, 224.0.0.0/24, 239.255.255.250, 255.255.255.255 } accept",
            "ip6 daddr { fe80::/10, ff02::/16 } accept",
        ]
    );
}

#[test]
fn every_range_by_its_first_address_is_taken() {
    let taken = [
        "lan|10.0.0.0/8",
        "lan|192.168.1.7/32",
        "lan|0.0.0.0/1",
        "lan|fd00::/8",
        "lan|2a02:a311:1:2::/64",
        "lan|1:2:3:4:5:6:7:8/128",
        "lan|1:2:3:4:5:6:7::/112",
        "lan|::1/128",
        "lan|fd00:0:0:1::/64,192.168.1.0/24,10.20.0.0/16",
    ];
    for value in taken {
        let root = Root::new();

        let run = root.run(&[&format!("killswitch={value}")], "");

        assert_eq!(run.code, Some(0), "{value}: {}", run.stderr);
    }
}

#[test]
fn a_range_that_isnt_a_networks_first_address_is_refused() {
    let thirty_three = vec!["10.0.0.0/24"; 33].join(",");
    let refused = [
        "lan|192.168.1.5/24".to_owned(),
        "lan|0.0.0.0/0".to_owned(),
        "lan|::/0".to_owned(),
        "lan|fe80::1/64".to_owned(),
        "lan|192.168.1.0/24,".to_owned(),
        "lan|,192.168.1.0/24".to_owned(),
        "lan|192.168.1.0/24,,10.0.0.0/8".to_owned(),
        "lan|1::2::3/64".to_owned(),
        "lan|a::b:/64".to_owned(),
        "lan|:a::b/64".to_owned(),
        "lan|1:2:3:4:5:6:7:8:9/128".to_owned(),
        "lan|1:2:3:4:5:6:7/112".to_owned(),
        "lan|12345::/16".to_owned(),
        "lan|192.168.01.0/24".to_owned(),
        "lan|256.0.0.0/8".to_owned(),
        "lan|10.0.0.0/33".to_owned(),
        "lan|fd00::/129".to_owned(),
        "lan|10.0.0.0/08".to_owned(),
        "lan|10.0.0.0".to_owned(),
        "lan|FD00::/8".to_owned(),
        "lan|10.0.0.0/8 accept".to_owned(),
        "lan|10.0.0.0/8\naccept".to_owned(),
        "lan|10.0.0.0/8; flush ruleset".to_owned(),
        "lan|$(reboot)".to_owned(),
        "lan|10.0.0.0/8|10.0.0.0/8".to_owned(),
        "lan".to_owned(),
        format!("lan|{thirty_three}"),
    ];
    for value in refused {
        let root = Root::new();

        let run = root.run(&[&format!("killswitch={value}")], "");

        assert_eq!(run.code, Some(1), "{value:?}: {}", run.stderr);
        assert_eq!(run.stdout, "", "{value:?}");
        nothing_written(&root);
    }
}

#[test]
fn what_an_admin_didnt_pin_changes_nothing_and_says_so() {
    let root = Root::new();
    let evil = certificate();
    let cases: [(&[&str], &str); 5] = [
        (&["ca=-"], &evil),
        (&["chrome=proxy|http://evil.example.com:8080|"], ""),
        (&["apt=http://evil.example.com:8080"], ""),
        (&[&format!("docker={PROXY}|*")], ""),
        (&["chrome=pac|http://evil.example.com/proxy.pac"], ""),
    ];
    for (args, stdin) in cases {
        let apt = format!("apt={PROXY}");
        let run = root.run(&[&[apt.as_str()][..], args].concat(), stdin);

        assert_eq!(run.code, Some(3), "{args:?}: {}", run.stderr);
        assert_eq!(run.stdout, "", "{args:?}");
        assert!(run.stderr.contains("isn't what an admin pinned"), "{}", run.stderr);
        nothing_written(&root);
    }
}

#[test]
fn a_ca_pinned_to_another_certificate_is_refused() {
    let root = Root::new();
    root.pin_ca(&certificate());

    let run = root.run(&["ca=-"], &certificate());

    assert_eq!(run.code, Some(3), "{}", run.stderr);
    nothing_written(&root);
}

#[test]
fn the_pin_helper_writes_every_pin_and_only_pins() {
    let root = Root::new();
    let cert = certificate();

    let run = root.pin(&["ca=-", "proxy=none", "no-proxy=", "pac=none"], &cert);

    assert_eq!(run.code, Some(0), "{}", run.stderr);
    let pins = root.read("etc/kubaos/proxy-pins");
    let ca = pins
        .lines()
        .find_map(|line| line.strip_prefix("ca="))
        .expect("a CA pin");
    assert!(ca.len() == 64 && ca.bytes().all(|b| b.is_ascii_hexdigit()), "{pins}");
    assert!(pins.contains("\nproxy=none\n"), "{pins}");
    assert_eq!(root.run(&[&format!("apt={PROXY}")], "").code, Some(3));
    assert_eq!(root.run(&["ca=-"], &cert).code, Some(0));
}

#[test]
fn the_pin_helper_refuses_what_isnt_a_pin_and_a_bad_value() {
    let root = Root::new();
    let before = root.read("etc/kubaos/proxy-pins");
    let cases: [(&[&str], i32); 5] = [
        (&[&format!("apt={PROXY}")], 2),
        (&["ca=none", "proxy=none", "no-proxy="], 2),
        (
            &["ca=none", "proxy=http://user:pw@proxy:3128", "no-proxy=", "pac=none"],
            1,
        ),
        (&["ca=none", "proxy=none", "no-proxy=a b", "pac=none"], 1),
        (&["ca=/etc/shadow", "proxy=none", "no-proxy=", "pac=none"], 1),
    ];
    for (args, code) in cases {
        let run = root.pin(args, "");

        assert_eq!(run.code, Some(code), "{args:?}: {}", run.stderr);
        assert_eq!(root.read("etc/kubaos/proxy-pins"), before);
    }
}

#[test]
fn the_write_helper_never_pins() {
    let root = Root::new();

    let run = root.run(&["proxy=http://evil.example.com:8080"], "");

    assert_eq!(run.code, Some(1), "{}", run.stderr);
    assert!(root.read("etc/kubaos/proxy-pins").contains(&format!("proxy={PROXY}")));
}

#[test]
fn off_takes_every_part_away_and_again_is_fine() {
    let root = Root::new();
    let args = every_part();
    let cert = certificate();
    root.pin_ca(&cert);
    assert_eq!(root.run(&strs(&args), &cert).code, Some(0));

    let first = root.run(&["off"], "");
    let again = root.run(&["off"], "");

    assert_eq!(first.code, Some(0), "{}", first.stderr);
    assert_eq!(again.code, Some(0), "{}", again.stderr);
    nothing_written(&root);
    assert!(root.log().contains("update-ca-certificates --fresh"), "{}", root.log());
}

#[test]
fn the_same_parts_again_restart_nothing() {
    let root = Root::new();
    let args = [format!("docker={PROXY}|{NO_PROXY}")];
    assert_eq!(root.run(&strs(&args), "").code, Some(0));
    fs::remove_file(root.log_path()).unwrap();

    let run = root.run(&strs(&args), "");

    assert_eq!(run.code, Some(0), "{}", run.stderr);
    assert_eq!(root.log(), "");
}

#[test]
fn a_value_it_refuses_anywhere_stops_it_before_any_part() {
    let root = Root::new();
    let args = [
        format!("apt={PROXY}"),
        format!("docker={PROXY}|a b"),
        format!("nix={PROXY}|{NO_PROXY}"),
    ];

    let run = root.run(&strs(&args), "");

    assert_eq!(run.code, Some(1));
    assert_eq!(run.stdout, "");
    assert!(run.stderr.contains("isn't a no_proxy list"), "{}", run.stderr);
    nothing_written(&root);
}

#[test]
fn a_daemon_that_wont_restart_loses_its_drop_in_so_the_next_run_writes_it_again() {
    let root = Root::new();
    root.write("fail-restart", "");
    let args = [
        format!("apt={PROXY}"),
        format!("docker={PROXY}|{NO_PROXY}"),
        format!("nix={PROXY}|{NO_PROXY}"),
    ];

    let run = root.run(&strs(&args), "");

    assert_eq!(run.code, Some(1));
    assert_eq!(run.stdout, format!("done apt={PROXY}\n"));
    assert!(!root.exists("etc/systemd/system/docker.service.d/kubaos-proxy.conf"));
    assert!(!root.exists("etc/systemd/system/nix-daemon.service.d/kubaos-proxy.conf"));
    fs::remove_file(root.path("fail-restart")).unwrap();
    assert_eq!(root.run(&strs(&args), "").code, Some(0));
    assert!(
        root.log().ends_with("systemctl try-restart nix-daemon.service\n"),
        "{}",
        root.log()
    );
}

#[test]
fn a_trust_store_that_wont_update_loses_the_ca_so_the_next_run_adds_it_again() {
    let root = Root::new();
    let cert = certificate();
    root.pin_ca(&cert);
    root.write("fail-ca", "");

    let run = root.run(&["ca=-"], &cert);

    assert_eq!(run.code, Some(1));
    assert!(!root.exists("usr/local/share/ca-certificates/kubaos-proxy/company.crt"));
    fs::remove_file(root.path("fail-ca")).unwrap();
    assert_eq!(root.run(&["ca=-"], &cert).code, Some(0));
    assert!(root.exists("usr/local/share/ca-certificates/kubaos-proxy/company.crt"));
}

#[test]
fn a_new_ca_restarts_the_docker_daemon_which_reads_its_roots_once() {
    let root = Root::new();
    let cert = certificate();
    root.pin_ca(&cert);

    let run = root.run(&["ca=-"], &cert);
    let off = root.run(&["ca=off"], "");

    assert_eq!(run.code, Some(0), "{}", run.stderr);
    assert_eq!(off.code, Some(0), "{}", off.stderr);
    assert_eq!(
        root.log().matches("systemctl try-restart docker.service\n").count(),
        2,
        "{}",
        root.log()
    );
}

#[test]
fn a_caller_that_stops_reading_never_stops_it_half_way() {
    let root = Root::new();
    let args = [format!("apt={PROXY}"), format!("chrome=proxy|{PROXY}|{NO_PROXY}")];
    let mut child = root.command(&script(), &strs(&args)).spawn().expect("run the helper");
    drop(child.stdout.take());
    drop(child.stdin.take());

    let status = child.wait().expect("it ends");

    assert_eq!(status.code(), Some(0));
    assert!(root.exists("etc/apt/apt.conf.d/80kubaos-proxy"));
    assert!(root.exists("etc/opt/chrome/policies/managed/kubaos-proxy.json"));
}

#[test]
fn values_that_could_break_out_of_where_they_go_are_refused() {
    let refused = [
        "apt=http://user:secret@proxy:3128",
        "apt=http://proxy:3128/path",
        "apt=http://proxy",
        "apt=http://proxy:99999",
        "apt=http://proxy:3128\";Dir \"/tmp",
        "apt=$(reboot)",
        "apt=ftp://proxy:21",
        "docker=http://proxy:3128",
        "docker=http://proxy:3128|a%b",
        "docker=http://proxy:3128|a\"b",
        "nix=http://pro\nxy:3128|",
        "chrome=pac|http://wpad/proxy.pac\"}",
        "chrome=pac|file:///etc/shadow",
        "chrome=socks|socks5://proxy:1080",
        "chrome=proxy|http://proxy:3128",
        "killswitch=maybe",
        "ca=/etc/shadow",
        "fans=max",
        "../../etc/passwd=x",
    ];
    for arg in refused {
        let root = Root::new();

        let run = root.run(&[arg], "");

        assert_eq!(run.code, Some(1), "{arg}: {}", run.stderr);
        assert_eq!(run.stdout, "", "{arg}");
        nothing_written(&root);
    }
}

#[test]
fn only_a_certificate_reaches_the_trust_store() {
    for stdin in [
        "not a certificate",
        "-----BEGIN CERTIFICATE-----\nnope\n-----END CERTIFICATE-----\n",
        "-----BEGIN PRIVATE KEY-----\nMIIB\n-----END PRIVATE KEY-----\n",
        "",
    ] {
        let root = Root::new();

        let run = root.run(&["ca=-"], stdin);

        assert_eq!(run.code, Some(1), "{stdin:?}");
        assert!(run.stderr.contains("isn't a PEM certificate"), "{}", run.stderr);
        nothing_written(&root);
        assert!(!root.log().contains("update-ca-certificates"));
    }
}

#[test]
fn a_certificate_is_written_as_openssl_reads_it_whatever_comes_after() {
    let root = Root::new();
    let cert = certificate();
    root.pin_ca(&cert);

    let run = root.run(&["ca=-"], &format!("{cert}-----BEGIN PRIVATE KEY-----\nsecret\n"));

    assert_eq!(run.code, Some(0), "{}", run.stderr);
    assert_eq!(
        root.read("usr/local/share/ca-certificates/kubaos-proxy/company.crt"),
        cert
    );
}

#[test]
fn a_daemon_that_isnt_installed_is_left_alone() {
    let root = Root::new();
    fs::remove_file(root.path("usr/lib/systemd/system/docker.service")).unwrap();

    let run = root.run(&[&format!("docker={PROXY}|{NO_PROXY}")], "");

    assert_eq!(run.code, Some(0), "{}", run.stderr);
    assert!(!root.exists("etc/systemd/system/docker.service.d"));
    assert_eq!(root.log(), "");
}

#[test]
fn a_pac_file_goes_to_chrome_as_its_policy() {
    let root = Root::new();

    let run = root.run(&[&format!("chrome=pac|{PAC}")], "");

    assert_eq!(run.code, Some(0), "{}", run.stderr);
    let policy: serde_json::Value =
        serde_json::from_str(&root.read("etc/opt/chrome/policies/managed/kubaos-proxy.json")).expect("JSON");
    assert_eq!(policy["ProxySettings"]["ProxyMode"], "pac_script");
    assert_eq!(policy["ProxySettings"]["ProxyPacUrl"], PAC);
}

#[test]
fn stats_names_each_peers_latest_handshake() {
    let root = Root::new();

    let run = root.run(&["stats"], "");

    assert_eq!(run.code, Some(0), "{}", run.stderr);
    assert_eq!(run.stdout, "xTIBA5rboUvnH4htodjb6e697QjLERt1NAB4mZqp8Dg=\t1700000000\n");
}

#[test]
fn a_usage_error_exits_2_and_does_nothing() {
    for args in [
        &[][..],
        &["off", "apt=off"],
        &["stats", "off"],
        &["ca=-", "ca=-"],
        &["apt"],
    ] {
        let root = Root::new();

        let run = root.run(args, "");

        assert_eq!(run.code, Some(2), "{args:?}");
        nothing_written(&root);
    }
}
