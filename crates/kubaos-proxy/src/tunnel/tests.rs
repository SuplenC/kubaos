use super::*;
use crate::wireguard::tests::OTHER_PEER;
use crate::wireguard::tests::PEER;
use crate::wireguard::tests::PRESHARED;
use crate::wireguard::tests::PRIVATE;

fn config(ips: &str) -> Config {
    Config::parse(&format!(
        "[Interface]\nPrivateKey = {PRIVATE}\nAddress = 10.8.0.2/32\nMTU = 1380\n\
         [Peer]\nPublicKey = {PEER}\nPresharedKey = {PRESHARED}\nEndpoint = vpn.example.com:51820\n\
         AllowedIPs = {ips}\nPersistentKeepalive = 25\n\
         [Peer]\nPublicKey = {OTHER_PEER}\nAllowedIPs = 172.16.0.0/12\n"
    ))
    .expect("parses")
}

fn keys(config: &Config) -> Keys {
    Keys::of(config).expect("the keys")
}

fn dns() -> Dns {
    Dns {
        servers: vec!["10.0.0.53".parse().unwrap()],
        domains: vec!["corp.example.com".to_owned()],
    }
}

fn get<'a>(connection: &'a Connection, setting: &str, key: &str) -> Option<&'a Prop> {
    connection.get(setting)?.get(key)
}

#[test]
fn the_tunnel_never_starts_by_itself_and_belongs_to_the_user() {
    let config = config("10.0.0.0/8");

    let connection = connection(&config, &keys(&config), &dns(), Some("kuba"));

    assert_eq!(get(&connection, "connection", "id"), Some(&Prop::Str(ID.to_owned())));
    assert_eq!(
        get(&connection, "connection", "type"),
        Some(&Prop::Str("wireguard".to_owned()))
    );
    assert_eq!(
        get(&connection, "connection", "interface-name"),
        Some(&Prop::Str(INTERFACE.to_owned()))
    );
    assert_eq!(get(&connection, "connection", "autoconnect"), Some(&Prop::Bool(false)));
    assert_eq!(
        get(&connection, "connection", "permissions"),
        Some(&Prop::Strs(vec!["user:kuba".to_owned()]))
    );
}

#[test]
fn the_keys_are_agent_owned_so_networkmanager_saves_them_nowhere() {
    let config = config("10.0.0.0/8");

    let connection = connection(&config, &keys(&config), &dns(), None);

    assert_eq!(
        get(&connection, "wireguard", "private-key"),
        Some(&Prop::Secret(PRIVATE.to_owned()))
    );
    assert_eq!(
        get(&connection, "wireguard", "private-key-flags"),
        Some(&Prop::U32(AGENT_OWNED))
    );
    let Some(Prop::Dicts(peers)) = get(&connection, "wireguard", "peers") else {
        panic!("peers");
    };
    assert_eq!(peers[0].get("preshared-key"), Some(&Prop::Secret(PRESHARED.to_owned())));
    assert_eq!(peers[0].get("preshared-key-flags"), Some(&Prop::U32(AGENT_OWNED)));
    assert_eq!(peers[1].get("preshared-key"), None);
    let printed = format!("{connection:?}");
    assert!(!printed.contains(PRIVATE) && !printed.contains(PRESHARED), "{printed}");
}

#[test]
fn each_peer_keeps_its_endpoint_ranges_and_keepalive() {
    let config = config("10.0.0.0/8, 192.168.50.0/24");

    let connection = connection(&config, &keys(&config), &dns(), None);

    let Some(Prop::Dicts(peers)) = get(&connection, "wireguard", "peers") else {
        panic!("peers");
    };
    assert_eq!(peers[0].get("public-key"), Some(&Prop::Str(PEER.to_owned())));
    assert_eq!(
        peers[0].get("endpoint"),
        Some(&Prop::Str("vpn.example.com:51820".to_owned()))
    );
    assert_eq!(
        peers[0].get("allowed-ips"),
        Some(&Prop::Strs(vec!["10.0.0.0/8".to_owned(), "192.168.50.0/24".to_owned()]))
    );
    assert_eq!(peers[0].get("persistent-keepalive"), Some(&Prop::U32(25)));
    assert_eq!(get(&connection, "wireguard", "mtu"), Some(&Prop::U32(1380)));
    assert_eq!(get(&connection, "wireguard", "fwmark"), Some(&Prop::U32(FWMARK)));
}

#[test]
fn a_split_tunnel_sends_only_the_work_domains_to_the_work_dns() {
    let config = config("10.0.0.0/8");

    let connection = connection(&config, &keys(&config), &dns(), None);

    assert_eq!(
        get(&connection, "ipv4", "dns-data"),
        Some(&Prop::Strs(vec!["10.0.0.53".to_owned()]))
    );
    assert_eq!(
        get(&connection, "ipv4", "dns-search"),
        Some(&Prop::Strs(vec!["~corp.example.com".to_owned()]))
    );
    assert_eq!(get(&connection, "ipv4", "dns-priority"), None);
    assert_eq!(
        get(&connection, "ipv6", "method"),
        Some(&Prop::Str("disabled".to_owned()))
    );
}

#[test]
fn a_full_tunnel_sends_every_name_to_the_work_dns() {
    let config = config("0.0.0.0/0");

    let connection = connection(&config, &keys(&config), &dns(), None);

    assert_eq!(
        get(&connection, "ipv4", "dns-search"),
        Some(&Prop::Strs(vec!["~corp.example.com".to_owned(), "~.".to_owned()]))
    );
    assert_eq!(
        get(&connection, "ipv4", "dns-priority"),
        Some(&Prop::I32(FULL_TUNNEL_DNS_PRIORITY))
    );
}

#[test]
fn the_addresses_go_to_their_family() {
    let config = Config::parse(&format!(
        "[Interface]\nPrivateKey = {PRIVATE}\nAddress = 10.8.0.2/24, fd00:8::2/64\n\
         [Peer]\nPublicKey = {PEER}\nAllowedIPs = ::/0\n"
    ))
    .unwrap();

    let connection = connection(&config, &keys(&config), &Dns::default(), None);

    let address = |family: &str| {
        let Some(Prop::Dicts(list)) = get(&connection, family, "address-data") else {
            panic!("{family} addresses");
        };
        (list[0].get("address").cloned(), list[0].get("prefix").cloned())
    };
    assert_eq!(
        address("ipv4"),
        (Some(Prop::Str("10.8.0.2".to_owned())), Some(Prop::U32(24)))
    );
    assert_eq!(
        address("ipv6"),
        (Some(Prop::Str("fd00:8::2".to_owned())), Some(Prop::U32(64)))
    );
    assert_eq!(
        get(&connection, "ipv6", "method"),
        Some(&Prop::Str("manual".to_owned()))
    );
}

#[test]
fn the_kill_switch_lets_the_same_mark_out_as_the_tunnel_sets() {
    let helper = include_str!("../../../../system/files/proxy/kubaos-proxy-write");

    assert!(
        helper.contains(&format!("FWMARK={FWMARK:#x}\n")),
        "kubaos-proxy-write's FWMARK isn't {FWMARK:#x}"
    );
    assert!(helper.contains(&format!("INTERFACE={INTERFACE}\n")));
}
