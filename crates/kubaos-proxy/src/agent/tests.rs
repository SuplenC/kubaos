use super::*;
use crate::profile::Dns;
use crate::secrets::Keys;
use crate::tunnel;
use crate::tunnel::ID;
use crate::wireguard::Config;
use crate::wireguard::tests::OTHER_PEER;
use crate::wireguard::tests::PEER;
use crate::wireguard::tests::PRESHARED;
use crate::wireguard::tests::PRIVATE;

const UUID: &str = "6f1c2b9e-4a63-4c1e-9d0b-2a7e5c3f8d41";

fn connection() -> Connection {
    let config = Config::parse(&format!(
        "[Interface]\nPrivateKey = {PRIVATE}\nAddress = 10.8.0.2/32\n\
         [Peer]\nPublicKey = {PEER}\nPresharedKey = {PRESHARED}\nEndpoint = 192.0.2.1:51820\n\
         AllowedIPs = 10.0.0.0/8\n\
         [Peer]\nPublicKey = {OTHER_PEER}\nAllowedIPs = 172.16.0.0/12\n"
    ))
    .expect("parses");
    let keys = Keys::of(&config).expect("the keys");
    tunnel::wireguard(&config, &keys, &Dns::default(), Some("kuba"))
}

fn ours() -> Request {
    Request {
        from_network_manager: true,
        id: Some(ID.to_owned()),
        uuid: Some(UUID.to_owned()),
        setting: WIREGUARD.to_owned(),
    }
}

fn keys_of(answer: Answer) -> Connection {
    let Answer::Keys(keys) = answer else {
        panic!("no keys: {answer:?}");
    };
    keys
}

#[test]
fn networkmanager_gets_the_private_key_for_the_tunnel_it_brings_up() {
    let agent = Agent::new(&connection(), UUID);

    let keys = keys_of(agent.answer(&ours()));

    let wireguard = &keys[WIREGUARD];
    assert_eq!(wireguard.get("private-key"), Some(&Prop::Secret(PRIVATE.to_owned())));
    assert_eq!(keys.len(), 1, "{keys:?}");
}

#[test]
fn each_preshared_key_goes_back_with_the_public_key_of_its_peer_and_nothing_else() {
    let agent = Agent::new(&connection(), UUID);

    let keys = keys_of(agent.answer(&ours()));

    let Some(Prop::Dicts(peers)) = keys[WIREGUARD].get("peers") else {
        panic!("peers: {keys:?}");
    };
    let expected = BTreeMap::from([
        ("public-key".to_owned(), Prop::Str(PEER.to_owned())),
        ("preshared-key".to_owned(), Prop::Secret(PRESHARED.to_owned())),
    ]);
    assert_eq!(peers, &vec![expected]);
    let wireguard: Vec<&String> = keys[WIREGUARD].keys().collect();
    assert_eq!(wireguard, ["peers", "private-key"]);
}

#[test]
fn a_caller_other_than_networkmanager_gets_nothing() {
    let agent = Agent::new(&connection(), UUID);
    let request = Request {
        from_network_manager: false,
        ..ours()
    };

    assert_eq!(agent.answer(&request), Answer::Nothing(Refusal::NotNetworkManager));
}

#[test]
fn another_connection_gets_nothing_even_under_the_tunnels_name() {
    let agent = Agent::new(&connection(), UUID);
    let other_uuid = Request {
        uuid: Some("0b5e7a1d-3c2f-4e8a-b6d9-1f4c7e2a9b30".to_owned()),
        ..ours()
    };
    let no_uuid = Request { uuid: None, ..ours() };
    let other_id = Request {
        id: Some("Home Wi-Fi".to_owned()),
        ..ours()
    };

    for request in [other_uuid, no_uuid, other_id] {
        assert_eq!(
            agent.answer(&request),
            Answer::Nothing(Refusal::OtherConnection),
            "{request:?}"
        );
    }
}

#[test]
fn a_setting_other_than_wireguard_gets_nothing() {
    let agent = Agent::new(&connection(), UUID);
    let request = Request {
        setting: "802-11-wireless-security".to_owned(),
        ..ours()
    };

    assert_eq!(agent.answer(&request), Answer::Nothing(Refusal::OtherSetting));
}

#[test]
fn the_agent_and_its_answer_never_print_a_key() {
    let agent = Agent::new(&connection(), UUID);

    let printed = format!("{agent:?} {:?}", agent.answer(&ours()));

    assert!(!printed.contains(PRIVATE) && !printed.contains(PRESHARED), "{printed}");
    assert!(printed.contains("<redacted>"), "{printed}");
}

/// An OpenVPN connection with a login, its password agent-owned.
fn openvpn_connection() -> Connection {
    let text = crate::openvpn::tests::inline() + "auth-user-pass\n";
    let config = crate::openvpn::Config::parse(&text, std::path::Path::new("/nonexistent")).expect("parses");
    let mut credentials = config.credentials.clone();
    credentials.username = Some(crate::secrets::Secret::new("kuba"));
    credentials.password = Some(crate::secrets::Secret::new("hunter2"));
    tunnel::openvpn(
        &config.sanitized(),
        &credentials,
        &tunnel::Files::default(),
        &Dns::default(),
        Some("kuba"),
    )
}

fn ours_for(setting: &str) -> Request {
    Request {
        setting: setting.to_owned(),
        ..ours()
    }
}

#[test]
fn networkmanager_gets_an_openvpn_password_in_the_vpn_setting() {
    let agent = Agent::new(&openvpn_connection(), UUID);

    let keys = keys_of(agent.answer(&ours_for(VPN)));

    assert_eq!(keys.len(), 1, "{keys:?}");
    let Some(Prop::Secrets(secrets)) = keys[VPN].get("secrets") else {
        panic!("no secrets: {keys:?}");
    };
    assert_eq!(secrets.get("password").map(String::as_str), Some("hunter2"));
    assert_eq!(keys[VPN].len(), 1, "only the secrets go back: {keys:?}");
}

#[test]
fn an_openvpn_agent_gives_nothing_for_another_setting_or_caller() {
    let agent = Agent::new(&openvpn_connection(), UUID);
    let stranger = Request {
        from_network_manager: false,
        ..ours_for(VPN)
    };

    assert_eq!(
        agent.answer(&ours_for(WIREGUARD)),
        Answer::Nothing(Refusal::OtherSetting)
    );
    assert_eq!(agent.answer(&stranger), Answer::Nothing(Refusal::NotNetworkManager));
}

#[test]
fn an_openvpn_agent_and_its_answer_never_print_the_password() {
    let agent = Agent::new(&openvpn_connection(), UUID);

    let printed = format!("{agent:?} {:?}", agent.answer(&ours_for(VPN)));

    assert!(!printed.contains("hunter2"), "{printed}");
    assert!(printed.contains("<redacted>"), "{printed}");
}
