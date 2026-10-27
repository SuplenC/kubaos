use std::error::Error as _;

use super::*;
use crate::wireguard::tests::PEER;
use crate::wireguard::tests::PRIVATE;

const CERT: &str = "-----BEGIN CERTIFICATE-----\nMIIB\n-----END CERTIFICATE-----\n";

fn dir() -> tempfile::TempDir {
    tempfile::tempdir().expect("a temp dir")
}

fn conf(full: bool) -> String {
    let ips = if full { "0.0.0.0/0, ::/0" } else { "10.0.0.0/8" };
    format!(
        "[Interface]\nAddress = 10.8.0.2/32\nDNS = 10.0.0.53, corp.example.com\n\n\
         [Peer]\nPublicKey = {PEER}\nEndpoint = vpn.example.com:51820\nAllowedIPs = {ips}\n"
    )
}

fn parse(text: &str, dir: &Path) -> Result<Profile, LoadError> {
    Profile::parse(text, dir)
}

#[test]
fn no_profile_is_nothing_to_switch() {
    let dir = dir();

    let profile = Profile::load(dir.path()).expect("loads");

    assert!(profile.is_empty());
    assert_eq!(profile.vault_item, DEFAULT_VAULT_ITEM);
}

#[test]
fn the_seed_has_every_part_commented_out() {
    let seed = include_str!("../../../../user-layer/desktop/network/proxy.toml");
    let dir = dir();

    let profile = parse(seed, dir.path()).expect("the seed parses");

    assert!(profile.is_empty());
}

#[test]
fn every_part_reads_with_the_files_it_names() {
    let dir = dir();
    fs::write(dir.path().join("wireguard.conf"), conf(true)).unwrap();
    fs::write(dir.path().join("company-ca.crt"), CERT).unwrap();
    let text = r#"
        [tunnel]
        conf = "wireguard.conf"
        [dns]
        servers = ["10.0.0.54"]
        domains = ["~lab.example.com", "corp.example.com"]
        [http]
        proxy = "HTTP://Proxy.Corp.Example.com:3128/"
        pac = "http://wpad.corp.example.com/proxy.pac"
        no_proxy = [".corp.example.com", "localhost"]
        [ca]
        file = "company-ca.crt"
    "#;

    let profile = parse(text, dir.path()).expect("parses");

    let tunnel = profile.tunnel.as_ref().expect("a tunnel");
    assert!(tunnel.config.is_full());
    assert!(tunnel.config.interface.private_key.is_none());
    assert!(profile.kill_switch());
    assert_eq!(
        profile.dns.servers,
        ["10.0.0.53".parse::<IpAddr>().unwrap(), "10.0.0.54".parse().unwrap()]
    );
    assert_eq!(profile.dns.domains, ["corp.example.com", "lab.example.com"]);
    assert_eq!(
        profile.http.proxy.as_ref().map(ToString::to_string).as_deref(),
        Some("http://proxy.corp.example.com:3128")
    );
    assert_eq!(
        profile.http.no_proxy,
        ["localhost", "127.0.0.1", "::1", ".corp.example.com"]
    );
    assert_eq!(profile.ca.as_ref().map(|ca| ca.pem.as_str()), Some(CERT));
}

#[test]
fn a_split_tunnel_has_no_kill_switch_and_a_full_one_can_turn_it_off() {
    let dir = dir();
    fs::write(dir.path().join("split.conf"), conf(false)).unwrap();
    fs::write(dir.path().join("full.conf"), conf(true)).unwrap();

    let split = parse("[tunnel]\nconf = \"split.conf\"\n", dir.path()).unwrap();
    let full = parse("[tunnel]\nconf = \"full.conf\"\nkill_switch = false\n", dir.path()).unwrap();

    assert!(!split.kill_switch());
    assert!(full.is_full());
    assert!(!full.kill_switch());
}

#[test]
fn a_conf_with_its_keys_in_it_is_used_without_them_and_flagged() {
    let dir = dir();
    fs::write(
        dir.path().join("wireguard.conf"),
        conf(false).replacen("[Interface]\n", &format!("[Interface]\nPrivateKey = {PRIVATE}\n"), 1),
    )
    .unwrap();

    let profile = parse("[tunnel]\nconf = \"wireguard.conf\"\n", dir.path()).expect("parses");

    let tunnel = profile.tunnel.expect("a tunnel");
    assert!(tunnel.keys_on_disk);
    assert!(tunnel.config.interface.private_key.is_none());
}

#[test]
fn dns_without_the_tunnel_is_refused() {
    let dir = dir();

    let err = parse("[dns]\nservers = [\"10.0.0.53\"]\n", dir.path()).expect_err("refused");

    assert!(matches!(err, LoadError::DnsWithoutTunnel), "{err:?}");
}

#[test]
fn a_value_that_could_break_where_it_goes_is_refused() {
    let dir = dir();
    let cases = [
        ("[http]\nproxy = \"http://user:secret@proxy:3128\"\n", "http.proxy"),
        ("[http]\nproxy = \"http://proxy\"\n", "http.proxy"),
        ("[http]\nproxy = \"socks5://proxy:1080\"\n", "http.proxy"),
        ("[http]\nproxy = \"http://proxy:3128/\\\"; rm -rf /\"\n", "http.proxy"),
        ("[http]\nproxy = \"http://pro\\nxy:3128\"\n", "http.proxy"),
        ("[http]\npac = \"http://wpad/proxy.pac' --x\"\n", "http.pac"),
        ("[http]\npac = \"file:///etc/shadow\"\n", "http.pac"),
        (
            "[http]\nproxy = \"http://proxy:3128\"\nno_proxy = [\"a b\"]\n",
            "http.no_proxy",
        ),
        (
            "[http]\nproxy = \"http://proxy:3128\"\nno_proxy = [\"%h\"]\n",
            "http.no_proxy",
        ),
        ("[tunnel]\nvault_item = \"\"\n", "tunnel.vault_item"),
    ];
    for (text, key) in cases {
        let err = parse(text, dir.path()).expect_err(text);

        assert!(
            matches!(&err, LoadError::Value { key: k, .. } if *k == key),
            "{text}: {err:?}"
        );
    }
}

#[test]
fn an_unknown_key_is_refused_rather_than_ignored() {
    let dir = dir();

    let err = parse("[http]\nproxys = \"http://proxy:3128\"\n", dir.path()).expect_err("refused");

    assert!(matches!(err, LoadError::Toml(_)), "{err:?}");
}

#[test]
fn a_ca_that_isnt_a_certificate_is_refused() {
    let dir = dir();
    fs::write(dir.path().join("key.pem"), "-----BEGIN PRIVATE KEY-----\n").unwrap();

    let err = parse("[ca]\nfile = \"key.pem\"\n", dir.path()).expect_err("refused");

    assert!(matches!(err, LoadError::NotPem(_)), "{err:?}");
}

#[test]
fn a_conf_with_a_problem_names_the_file_and_the_line() {
    let dir = dir();
    fs::write(
        dir.path().join("wireguard.conf"),
        format!("[Interface]\nPrivateKey = {PRIVATE}x\n"),
    )
    .unwrap();

    let err = parse("[tunnel]\nconf = \"wireguard.conf\"\n", dir.path()).expect_err("refused");

    assert!(err.to_string().contains("wireguard.conf"), "{err}");
    let cause = err.source().map(ToString::to_string).unwrap_or_default();
    assert_eq!(cause, "line 2 of the WireGuard config");
    assert!(!format!("{err:?}").contains(PRIVATE), "{err:?}");
}

#[test]
fn naming_the_conf_keeps_the_users_comments_and_settings() {
    let dir = dir();
    let path = dir.path().join("proxy.toml");
    fs::write(
        &path,
        "# Mine.\n[tunnel]\n# The switch.\nkill_switch = false\n\n[http]\nproxy = \"http://proxy:3128\"\n",
    )
    .unwrap();

    set_conf(&path, "wireguard.conf").expect("written");

    let text = fs::read_to_string(&path).unwrap();
    assert!(text.contains("# Mine."), "{text}");
    assert!(text.contains("# The switch."), "{text}");
    assert!(text.contains("kill_switch = false"), "{text}");
    assert!(text.contains("conf = \"wireguard.conf\""), "{text}");
    assert!(text.contains("proxy = \"http://proxy:3128\""), "{text}");
}

#[test]
fn naming_the_conf_without_a_profile_makes_one() {
    let dir = dir();
    let path = dir.path().join("network/proxy.toml");

    set_conf(&path, "wireguard.conf").expect("written");

    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "[tunnel]\nconf = \"wireguard.conf\"\n"
    );
}

#[test]
fn a_proxy_is_a_scheme_a_host_and_a_port() {
    let url = |text| ProxyUrl::parse(text).map(|u| u.to_string());

    assert_eq!(url("http://10.0.0.1:8080").as_deref(), Some("http://10.0.0.1:8080"));
    assert_eq!(
        url("https://[2001:db8::1]:443").as_deref(),
        Some("https://[2001:db8::1]:443")
    );
    assert_eq!(url("http://proxy:0"), None);
    assert_eq!(url("http://proxy:99999"), None);
    assert_eq!(url("http://proxy:3128/path"), None);
    assert_eq!(url("http://proxy:3128?x=1"), None);
}
