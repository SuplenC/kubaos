use super::*;
use crate::openvpn::tests::CA;
use crate::openvpn::tests::inline;
use crate::wireguard::tests::PEER;
use crate::wireguard::tests::PRIVATE;

fn wireguard() -> String {
    format!(
        "# IT's tunnel\n[Interface]\nPrivateKey = {PRIVATE}\nAddress = 10.8.0.2/32\n\
         [Peer]\nPublicKey = {PEER}\nEndpoint = vpn.example.com:51820\nAllowedIPs = 10.0.0.0/8\n"
    )
}

#[test]
fn a_wireguard_config_is_told_by_its_sections() {
    assert_eq!(detect(&wireguard()), Some(Type::WireGuard));
    assert_eq!(detect("[peer]\npublickey = x\n"), Some(Type::WireGuard));
}

#[test]
fn an_openvpn_config_is_told_by_its_directives() {
    assert_eq!(detect(&inline()), Some(Type::OpenVpn));
    assert_eq!(detect("remote vpn.example.com 1194\n"), Some(Type::OpenVpn));
    assert_eq!(detect("--client\n"), Some(Type::OpenVpn));
    assert_eq!(detect(&format!("<ca>\n{CA}</ca>\n")), Some(Type::OpenVpn));
}

#[test]
fn anything_else_is_neither() {
    assert_eq!(detect(""), None);
    assert_eq!(detect("# client\n; remote vpn.example.com\n"), None);
    assert_eq!(detect("[Unit]\nDescription=client\n"), None);
    assert_eq!(detect("clients = 3\nremoteness = far\n"), None);
}

#[test]
fn each_type_has_its_own_interface_and_file() {
    assert_eq!(Type::WireGuard.interface(), "wg-kubaos");
    assert_eq!(Type::OpenVpn.interface(), "tun-kubaos");
    assert_eq!(Type::WireGuard.file_name(), "vpn.conf");
    assert_eq!(Type::OpenVpn.file_name(), "vpn.ovpn");
    for kind in Type::ALL {
        assert_eq!(Type::from_name(kind.name()), Some(kind));
    }
}

#[test]
fn either_type_says_its_server_and_routes() {
    let wireguard = Config::parse(Type::WireGuard, &wireguard(), Path::new("/")).unwrap();
    let openvpn = Config::parse(Type::OpenVpn, &inline(), Path::new("/")).unwrap();

    assert_eq!(
        wireguard.endpoint().map(ToString::to_string).as_deref(),
        Some("vpn.example.com:51820")
    );
    assert_eq!(
        openvpn.endpoint().map(ToString::to_string).as_deref(),
        Some("vpn.example.com:1194")
    );
    assert_eq!(
        wireguard.routes().iter().map(ToString::to_string).collect::<Vec<_>>(),
        ["10.0.0.0/8", "10.8.0.2/32"]
    );
    assert!(wireguard.has_secrets() && openvpn.has_secrets());
    assert!(!wireguard.sanitized().has_secrets() && !openvpn.sanitized().has_secrets());
}
