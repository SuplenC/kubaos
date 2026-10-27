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
        row("Tunnel").as_deref(),
        Some("up: full tunnel to vpn.example.com:51820, kill switch")
    );
    assert_eq!(row("Last handshake").as_deref(), Some("42 s ago"));
    assert_eq!(row("Moved").as_deref(), Some("12.4 MB in, 512 B out"));
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
