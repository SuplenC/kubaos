use std::error::Error as _;

use super::*;

/// wg(8)'s example keys.
pub const PRIVATE: &str = "yAnz5TF+lXXJte14tji3zlMNq+hd2rYUIgJBgB3fBmk=";
pub const PEER: &str = "xTIBA5rboUvnH4htodjb6e697QjLERt1NAB4mZqp8Dg=";
pub const PRESHARED: &str = "HIgo9xNzJMWLKASShiTqIybxZ0U3wGLiUeJ1PKf8ykw=";
pub const OTHER_PEER: &str = "gN65BkIKy1eCE9pP1wdc8ROUtkHLF2PfAqYdyYBz6EA=";

fn split() -> String {
    format!(
        "# IT's tunnel, 2026\n\
         [Interface]\n\
         PrivateKey = {PRIVATE}\n\
         Address = 10.8.0.2/32, fd00:8::2/128\n\
         DNS = 10.0.0.53, corp.example.com\n\
         PostUp = iptables -A FORWARD -i %i -j ACCEPT\n\
         \n\
         [Peer]\n\
         PublicKey = {PEER}\n\
         PresharedKey = {PRESHARED}\n\
         Endpoint = vpn.example.com:51820\n\
         AllowedIPs = 10.0.0.0/8\n\
         AllowedIPs = 192.168.50.0/24 # the lab\n\
         PersistentKeepalive = 25\n"
    )
}

fn problem(text: &str) -> ParseError {
    Config::parse(text).expect_err("the config is refused")
}

#[test]
fn a_split_tunnel_config_reads_whole() {
    let config = Config::parse(&split()).expect("parses");

    assert_eq!(config.interface.private_key.as_ref().map(Secret::expose), Some(PRIVATE));
    assert_eq!(
        config.interface.addresses,
        [
            Cidr::parse("10.8.0.2/32").unwrap(),
            Cidr::parse("fd00:8::2/128").unwrap()
        ]
    );
    assert_eq!(config.interface.dns_servers, ["10.0.0.53".parse::<IpAddr>().unwrap()]);
    assert_eq!(config.interface.dns_domains, ["corp.example.com"]);
    let peer = &config.peers[0];
    assert_eq!(peer.public_key.to_string(), PEER);
    assert_eq!(peer.preshared_key.as_ref().map(Secret::expose), Some(PRESHARED));
    assert_eq!(
        peer.endpoint.as_ref().map(ToString::to_string).as_deref(),
        Some("vpn.example.com:51820")
    );
    assert_eq!(peer.allowed_ips.len(), 2);
    assert_eq!(peer.keepalive, Some(25));
    assert_eq!(config.ignored, ["PostUp"]);
    assert!(!config.is_full());
}

#[test]
fn every_address_of_a_family_makes_a_full_tunnel() {
    for all in ["0.0.0.0/0", "::/0", "10.0.0.0/8, ::/0"] {
        let text = format!("[Interface]\nAddress = 10.8.0.2\n[Peer]\nPublicKey = {PEER}\nAllowedIPs = {all}\n");

        assert!(Config::parse(&text).expect("parses").is_full(), "{all}");
    }
}

#[test]
fn keys_and_sections_match_whatever_their_case() {
    let text = format!(
        "[interface]\nprivatekey={PRIVATE}\naddress=10.8.0.2\n[PEER]\npublickey={PEER}\nallowedips=10.0.0.0/8\n"
    );

    let config = Config::parse(&text).expect("parses");

    assert_eq!(config.interface.addresses[0].to_string(), "10.8.0.2/32");
}

#[test]
fn the_sanitized_config_keeps_everything_but_the_secrets() {
    let config = Config::parse(&split()).expect("parses");

    let text = config.sanitized().render();

    assert!(!text.contains(PRIVATE), "{text}");
    assert!(!text.contains(PRESHARED), "{text}");
    assert!(!text.contains("PostUp"), "{text}");
    let again = Config::parse(&text).expect("the sanitized config parses");
    assert_eq!(again, config.sanitized());
}

#[test]
fn a_secret_never_prints() {
    let config = Config::parse(&split()).expect("parses");

    let debug = format!("{config:?}");

    assert!(!debug.contains(PRIVATE), "{debug}");
    assert!(!debug.contains(PRESHARED), "{debug}");
}

#[test]
fn a_bad_private_key_is_named_but_never_shown() {
    let text = "[Interface]\nPrivateKey = not-a-key-but-a-secret\nAddress = 10.8.0.2\n";

    let err = problem(text);

    assert_eq!(err.line, 2);
    assert_eq!(err.problem, Problem::BadKey("PrivateKey"));
    let said = format!("{err}: {}", err.source().map(ToString::to_string).unwrap_or_default());
    assert!(!said.contains("not-a-key"), "{said}");
}

#[test]
fn what_isnt_a_wireguard_config_is_refused_by_line() {
    let cases = [
        ("[Interfaces]\n", 1, "Section"),
        ("Address = 10.8.0.2\n", 1, "Outside"),
        ("[Interface]\nAddress 10.8.0.2\n", 2, "NotKeyValue"),
        ("[Interface]\nAddres = 10.8.0.2\n", 2, "Key"),
        ("[Interface]\nAddress = 10.8.0.300\n", 2, "BadValue"),
        ("[Interface]\nAddress = 10.8.0.2/33\n", 2, "BadValue"),
        ("[Interface]\nAddress = 10.8.0.2\nMTU = 70000\n", 3, "BadValue"),
        ("[Interface]\nAddress = 10.8.0.2\n[Interface]\n", 3, "TwoInterfaces"),
    ];
    for (text, line, kind) in cases {
        let err = problem(text);

        assert_eq!(err.line, line, "{text:?}: {err:?}");
        assert!(format!("{:?}", err.problem).starts_with(kind), "{text:?}: {err:?}");
    }
}

#[test]
fn a_config_must_have_an_address_and_a_peer_with_its_key_and_ranges() {
    let no_peer = "[Interface]\nAddress = 10.8.0.2\n";
    let no_key = "[Interface]\nAddress = 10.8.0.2\n[Peer]\nAllowedIPs = 10.0.0.0/8\n";
    let no_ips = format!("[Interface]\nAddress = 10.8.0.2\n[Peer]\nPublicKey = {PEER}\n");
    let no_address = format!("[Interface]\n[Peer]\nPublicKey = {PEER}\nAllowedIPs = 10.0.0.0/8\n");
    let twice = format!(
        "[Interface]\nAddress = 10.8.0.2\n[Peer]\nPublicKey = {PEER}\nAllowedIPs = 10.0.0.0/8\n\
         [Peer]\nPublicKey = {PEER}\nAllowedIPs = 10.1.0.0/16\n"
    );

    assert_eq!(problem(no_peer).problem, Problem::NoPeer);
    assert_eq!(problem(no_key).problem, Problem::NoPublicKey);
    assert_eq!(problem(&no_ips).problem, Problem::NoAllowedIps);
    assert_eq!(problem(&no_address).problem, Problem::NoAddress);
    assert_eq!(problem(&twice).problem, Problem::SamePeer);
}

#[test]
fn two_peers_each_keep_their_own_ranges() {
    let text = format!(
        "[Interface]\nAddress = 10.8.0.2\n[Peer]\nPublicKey = {PEER}\nAllowedIPs = 10.0.0.0/8\n\
         [Peer]\nPublicKey = {OTHER_PEER}\nEndpoint = [2001:db8::1]:51820\nAllowedIPs = 172.16.0.0/12\n"
    );

    let config = Config::parse(&text).expect("parses");

    assert_eq!(config.peers.len(), 2);
    assert_eq!(config.peers[1].allowed_ips[0].to_string(), "172.16.0.0/12");
    assert_eq!(
        config.peers[1].endpoint.as_ref().map(ToString::to_string).as_deref(),
        Some("[2001:db8::1]:51820")
    );
}

#[test]
fn an_endpoint_is_a_host_or_an_address_and_a_port() {
    assert!(Endpoint::parse("vpn.example.com:51820").is_some());
    assert!(Endpoint::parse("192.0.2.1:51820").is_some());
    assert!(Endpoint::parse("vpn.example.com").is_none());
    assert!(Endpoint::parse("vpn.example.com:0").is_none());
    assert!(Endpoint::parse("vpn example.com:51820").is_none());
    assert!(Endpoint::parse("$(reboot):51820").is_none());
}

#[test]
fn a_key_is_44_characters_of_base64_for_32_bytes() {
    assert!(Key::parse(PEER).is_some());
    assert!(Key::parse("xTIBA5rboUvnH4htodjb6e697QjLERt1NAB4mZqp8Dg").is_none());
    assert!(Key::parse("xTIBA5rboUvnH4htodjb6e697QjLERt1NAB4mZqp8D==").is_none());
    assert!(Key::parse("xTIBA5rboUvnH4htodjb6e697QjLERt1NAB4mZq\"8Dg=").is_none());
}
