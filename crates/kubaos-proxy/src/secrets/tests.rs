use super::*;
use crate::fake::Fake;
use crate::openvpn::tests::CERT;
use crate::openvpn::tests::KEY;
use crate::openvpn::tests::inline;
use crate::wireguard::Config;
use crate::wireguard::tests::OTHER_PEER;
use crate::wireguard::tests::PEER;
use crate::wireguard::tests::PRESHARED as PRESHARED_KEY;
use crate::wireguard::tests::PRIVATE as PRIVATE_KEY;

fn config() -> Config {
    Config::parse(&format!(
        "[Interface]\nPrivateKey = {PRIVATE_KEY}\nAddress = 10.8.0.2\n\
         [Peer]\nPublicKey = {PEER}\nPresharedKey = {PRESHARED_KEY}\nAllowedIPs = 10.0.0.0/8\n\
         [Peer]\nPublicKey = {OTHER_PEER}\nAllowedIPs = 172.16.0.0/12\n"
    ))
    .expect("parses")
}

fn keys() -> Keys {
    Keys::of(&config()).expect("keys")
}

fn secrets() -> Secrets {
    Secrets::WireGuard(keys())
}

fn tunnel() -> vpn::Config {
    vpn::Config::WireGuard(config().sanitized())
}

fn session() -> Session {
    Session::new("the-session")
}

/// No secret is ever in a command's arguments, where any process could read it.
fn never_in_argv(fake: &Fake) {
    for line in &fake.ran {
        assert!(!line.contains(PRIVATE_KEY) && !line.contains(PRESHARED_KEY), "{line}");
        assert!(!line.contains("the-session"), "{line}");
    }
}

#[test]
fn the_keys_go_to_the_keyring_on_stdin_and_come_back() {
    let mut fake = Fake::new();

    store(&mut fake, &secrets()).expect("stored");
    let back = load(&mut fake, &tunnel()).expect("loaded");

    assert_eq!(back, secrets());
    assert!(matches!(&back, Secrets::WireGuard(keys) if keys.preshared.len() == 1));
    never_in_argv(&fake);
    assert!(fake.ran_like("secret-tool clear service kubaos-proxy"));
}

#[test]
fn an_empty_keyring_says_to_pull_the_keys() {
    let mut fake = Fake::new();

    let err = load(&mut fake, &tunnel()).expect_err("nothing there");

    assert!(matches!(err, Error::NotInKeyring), "{err:?}");
    assert!(err.to_string().contains("kubaos proxy pull"), "{err}");
}

#[test]
fn a_new_vault_item_holds_the_keys_as_hidden_fields() {
    let mut fake = Fake::new();

    to_vault(&mut fake, &session(), "kubaos-proxy", &secrets()).expect("stored");

    never_in_argv(&fake);
    let item = &fake.vault[0];
    assert_eq!(item["name"], "kubaos-proxy");
    assert_eq!(item["type"], 2);
    let fields = item["fields"].as_array().expect("fields");
    assert!(fields.iter().all(|field| field["type"] == 1), "{fields:?}");
    assert_eq!(fields[0]["name"], PRIVATE);
    assert_eq!(fields[0]["value"], PRIVATE_KEY);
    assert_eq!(fields[1]["name"], format!("{PRESHARED} {PEER}"));
    assert_eq!(fields[1]["value"], PRESHARED_KEY);
}

#[test]
fn importing_again_replaces_the_keys_and_keeps_the_items_other_fields() {
    let mut fake = Fake::new();
    to_vault(&mut fake, &session(), "kubaos-proxy", &secrets()).expect("stored");
    fake.vault[0]["fields"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({ "name": "note", "value": "IT ticket 42", "type": 0 }));
    let mut new = keys();
    new.preshared.clear();

    to_vault(&mut fake, &session(), "kubaos-proxy", &Secrets::WireGuard(new)).expect("stored again");

    assert_eq!(fake.vault.len(), 1);
    assert!(
        fake.ran.iter().any(|line| line.ends_with("edit item id-0")),
        "{:?}",
        fake.ran
    );
    let names: Vec<&str> = fake.vault[0]["fields"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|f| f["name"].as_str())
        .collect();
    assert_eq!(names, ["note", PRIVATE]);
}

#[test]
fn the_vault_gives_the_keys_back_for_a_new_keyring() {
    let mut fake = Fake::new();
    to_vault(&mut fake, &session(), "kubaos-proxy", &secrets()).expect("stored");

    let back = from_vault(&mut fake, &session(), "kubaos-proxy").expect("read");

    assert_eq!(back, secrets());
}

#[test]
fn an_item_named_like_but_not_exactly_is_not_the_tunnels() {
    let mut fake = Fake::new();
    to_vault(&mut fake, &session(), "kubaos-proxy-old", &secrets()).expect("stored");

    let err = from_vault(&mut fake, &session(), "kubaos-proxy").expect_err("no such item");

    assert!(matches!(err, Error::NotInVault(_)), "{err:?}");
}

#[test]
fn keys_never_print() {
    let printed = format!("{:?}", keys());

    assert!(
        !printed.contains(PRIVATE_KEY) && !printed.contains(PRESHARED_KEY),
        "{printed}"
    );
}

fn openvpn() -> (vpn::Config, Secrets) {
    let text = inline() + "auth-user-pass\n";
    let config = crate::openvpn::Config::parse(&text, std::path::Path::new("/nonexistent")).expect("parses");
    let mut credentials = config.credentials.clone();
    credentials.username = Some(Secret::new("kuba"));
    credentials.password = Some(Secret::new("hunter2"));
    (vpn::Config::OpenVpn(config.sanitized()), Secrets::OpenVpn(credentials))
}

#[test]
fn an_openvpn_certificate_key_and_login_go_to_the_keyring_on_stdin_and_come_back() {
    let mut fake = Fake::new();
    let (config, secrets) = openvpn();

    store(&mut fake, &secrets).expect("stored");
    let back = load(&mut fake, &config).expect("loaded");

    assert_eq!(back, secrets);
    for line in &fake.ran {
        assert!(!line.contains("hunter2") && !line.contains("clientKEYsecret"), "{line}");
    }
    assert_eq!(
        fake.keyring.get("service kubaos-proxy key key").map(String::as_str),
        Some(KEY)
    );
    assert_eq!(
        fake.keyring.get("service kubaos-proxy key cert").map(String::as_str),
        Some(CERT)
    );
}

#[test]
fn an_openvpn_vpn_without_its_password_in_the_keyring_says_to_pull() {
    let mut fake = Fake::new();
    let (config, secrets) = openvpn();
    store(&mut fake, &secrets).expect("stored");
    fake.keyring.remove("service kubaos-proxy key password");

    let err = load(&mut fake, &config).expect_err("no password");

    assert!(matches!(err, Error::NotInKeyring), "{err:?}");
}

#[test]
fn the_vault_holds_openvpn_secrets_as_hidden_fields_and_gives_them_back() {
    let mut fake = Fake::new();
    let (_, secrets) = openvpn();

    to_vault(&mut fake, &session(), "kubaos-proxy", &secrets).expect("stored");
    let back = from_vault(&mut fake, &session(), "kubaos-proxy").expect("read");

    assert_eq!(back, secrets);
    let names: Vec<&str> = fake.vault[0]["fields"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|f| f["name"].as_str())
        .collect();
    assert_eq!(names, ["cert", "key", "tls-auth", "username", "password"]);
    assert!(
        fake.vault[0]["fields"]
            .as_array()
            .unwrap()
            .iter()
            .all(|f| f["type"] == 1)
    );
}

#[test]
fn importing_another_type_replaces_the_old_types_secrets_in_the_vault() {
    let mut fake = Fake::new();
    to_vault(&mut fake, &session(), "kubaos-proxy", &secrets()).expect("WireGuard's");
    let (_, openvpn) = openvpn();

    to_vault(&mut fake, &session(), "kubaos-proxy", &openvpn).expect("OpenVPN's");
    let back = from_vault(&mut fake, &session(), "kubaos-proxy").expect("read");

    assert_eq!(back, openvpn);
    let fields = fake.vault[0]["fields"].as_array().unwrap();
    assert!(
        !fields
            .iter()
            .any(|f| f["name"].as_str().is_some_and(|n| n.starts_with("pr"))),
        "{fields:?}"
    );
}

#[test]
fn openvpn_secrets_never_print() {
    let (_, secrets) = openvpn();

    let printed = format!("{secrets:?}");

    assert!(
        !printed.contains("hunter2") && !printed.contains("clientKEYsecret"),
        "{printed}"
    );
}
