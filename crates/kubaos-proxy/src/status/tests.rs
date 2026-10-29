use super::*;
use crate::fake::Fake;
use crate::plan::tests::whole;
use crate::wireguard::tests::PRIVATE;

#[test]
fn an_empty_profile_is_not_set_up() {
    let dir = tempfile::tempdir().unwrap();
    let paths = Paths::under(dir.path());

    let now = read(&mut Fake::new(), &paths, &Ok(Profile::empty()), false);

    assert!(!now.configured && !now.on);
    assert!(rows(&now, 0)[0].value.starts_with("not set up"));
}

#[test]
fn a_profile_that_is_up_reads_live_with_its_handshake_and_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let paths = Paths::under(dir.path());
    let profile = whole(&paths.network, true);
    let mut fake = Fake::new();
    fake.keyring_has("key private-key", PRIVATE);
    fake.names
        .insert("vpn.example.com".to_owned(), vec!["198.51.100.7".parse().unwrap()]);
    apply::set_wanted(&paths, true).unwrap();
    apply::on(&mut fake, &paths, &Plan::for_profile(&profile), None).expect("on");
    let stats = paths.sys.join("class/net/wg-kubaos/statistics");
    fs::create_dir_all(&stats).unwrap();
    fs::write(stats.join("rx_bytes"), "12400000\n").unwrap();
    fs::write(stats.join("tx_bytes"), "512\n").unwrap();

    let now = read(&mut fake, &paths, &Ok(profile), true);

    assert!(now.configured && now.on && now.wanted);
    assert_eq!(now.parts, ["tunnel", "dns", "proxy", "ca", "kill-switch"]);
    let tunnel = now.tunnel.as_ref().expect("the tunnel");
    assert!(tunnel.up && tunnel.full && tunnel.kill_switch);
    assert_eq!(tunnel.handshake, Some(1_700_000_000));
    let rows = rows(&now, 1_700_000_042);
    let row = |label: &str| rows.iter().find(|row| row.label == label).map(|row| row.value.clone());
    assert_eq!(row("State").as_deref(), Some("on"));
    assert_eq!(
        row("VPN").as_deref(),
        Some("up: WireGuard, full tunnel to vpn.example.com:51820, kill switch")
    );
    assert_eq!(row("Last handshake").as_deref(), Some("42 s ago"));
    assert_eq!(row("Moved").as_deref(), Some("12.4 MB in, 512 B out"));
}

#[test]
fn the_local_network_reads_as_allowed_with_its_ranges_or_blocked() {
    let dir = tempfile::tempdir().unwrap();
    let paths = Paths::under(dir.path());
    let mut profile = whole(&paths.network, true);
    let mut fake = Fake::new();
    fake.keyring_has("key private-key", PRIVATE);
    fake.names
        .insert("vpn.example.com".to_owned(), vec!["198.51.100.7".parse().unwrap()]);
    fake.ip = r#"[{"ifname":"wlp0s20f3","addr_info":[{"local":"192.168.1.23","prefixlen":24}]}]"#.to_owned();
    let row = |now: &Status| {
        rows(now, 0)
            .into_iter()
            .find(|row| row.label == "Local network")
            .map(|row| row.value)
    };
    let off = read(&mut fake, &paths, &Ok(profile.clone()), false);
    apply::on(&mut fake, &paths, &Plan::for_profile(&profile), None).expect("on");

    let allowed = read(&mut fake, &paths, &Ok(profile.clone()), false);
    profile.tunnel.as_mut().unwrap().allow_local_network = false;
    apply::kill_switch_again(&mut fake, &paths, &Plan::for_profile(&profile)).expect("again");
    let blocked = read(&mut fake, &paths, &Ok(profile), false);

    assert_eq!(row(&off).as_deref(), Some("allowed"));
    assert_eq!(row(&allowed).as_deref(), Some("allowed: 192.168.1.0/24"));
    let tunnel = allowed.tunnel.as_ref().expect("the tunnel");
    assert!(tunnel.allow_local_network);
    assert_eq!(tunnel.local_networks, ["192.168.1.0/24"]);
    assert_eq!(row(&blocked).as_deref(), Some("blocked"));
    assert!(
        blocked
            .tunnel
            .as_ref()
            .is_some_and(|tunnel| tunnel.local_networks.is_empty())
    );
}

#[test]
fn not_live_the_helper_is_never_asked() {
    let dir = tempfile::tempdir().unwrap();
    let paths = Paths::under(dir.path());
    let profile = whole(&paths.network, true);
    let mut fake = Fake::new();

    let now = read(&mut fake, &paths, &Ok(profile), false);

    assert!(!now.on);
    assert!(fake.ran.is_empty(), "{:?}", fake.ran);
}

#[test]
fn the_shell_reads_camel_case_json() {
    let now = Status {
        configured: true,
        tunnel: Some(Tunnel {
            kill_switch: true,
            rx_bytes: Some(1),
            ..Tunnel::default()
        }),
        ..Status::default()
    };

    let json = serde_json::to_string(&now).unwrap();

    assert!(json.contains("\"killSwitch\":true"), "{json}");
    assert!(json.contains("\"allowLocalNetwork\":false"), "{json}");
    assert!(json.contains("\"localNetworks\":[]"), "{json}");
    assert!(json.contains("\"rxBytes\":1"), "{json}");
    assert!(json.contains("\"dnsServers\":[]"), "{json}");
}

#[test]
fn time_and_bytes_read_naturally() {
    assert_eq!(ago(0, 100), "never");
    assert_eq!(ago(100, 3700), "1 h ago");
    assert_eq!(ago(100, 220), "2 min ago");
    assert_eq!(bytes(999), "999 B");
    assert_eq!(bytes(1_500), "1.5 kB");
}

#[test]
fn an_openvpn_vpn_reads_its_type_and_its_interfaces_bytes_with_no_handshake() {
    let dir = tempfile::tempdir().unwrap();
    let paths = Paths::under(dir.path());
    let text = crate::openvpn::tests::inline() + "redirect-gateway def1\n";
    let config = crate::openvpn::Config::parse(&text, dir.path()).unwrap();
    fs::create_dir_all(&paths.network).unwrap();
    fs::write(paths.network.join("vpn.ovpn"), config.sanitized().render()).unwrap();
    fs::write(paths.profile(), "[tunnel]\ntype = \"openvpn\"\nconf = \"vpn.ovpn\"\n").unwrap();
    let profile = Profile::load(&paths.network).unwrap();
    let mut fake = Fake::new();
    fake.tunnel = Some(crate::tunnel::Connection::new());
    let stats = paths.sys.join("class/net/tun-kubaos/statistics");
    fs::create_dir_all(&stats).unwrap();
    fs::write(stats.join("rx_bytes"), "2048\n").unwrap();
    fs::write(stats.join("tx_bytes"), "1000\n").unwrap();

    let now = read(&mut fake, &paths, &Ok(profile), true);

    let tunnel = now.tunnel.as_ref().expect("the VPN");
    assert_eq!(
        (tunnel.kind, tunnel.type_name, tunnel.interface),
        ("openvpn", "OpenVPN", "tun-kubaos")
    );
    assert_eq!((tunnel.rx_bytes, tunnel.tx_bytes), (Some(2048), Some(1000)));
    assert_eq!(tunnel.handshake, None);
    assert!(!fake.ran.iter().any(|line| line.ends_with(" stats")), "{:?}", fake.ran);
    let json = serde_json::to_string(&now).unwrap();
    assert!(json.contains("\"type\":\"openvpn\""), "{json}");
    assert!(json.contains("\"typeName\":\"OpenVPN\""), "{json}");
    assert!(json.contains("\"interface\":\"tun-kubaos\""), "{json}");
    let row = rows(&now, 0)
        .into_iter()
        .find(|row| row.label == "VPN")
        .map(|row| row.value);
    assert_eq!(
        row.as_deref(),
        Some("up: OpenVPN, full tunnel to vpn.example.com:1194, kill switch")
    );
}
