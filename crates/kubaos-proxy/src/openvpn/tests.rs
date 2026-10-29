use std::error::Error as _;

use super::*;

pub const CA: &str = "-----BEGIN CERTIFICATE-----\nMIIBcaCAofTheServer\n-----END CERTIFICATE-----\n";
pub const CERT: &str = "-----BEGIN CERTIFICATE-----\nMIIBcLIENTcertificate\n-----END CERTIFICATE-----\n";
pub const KEY: &str = "-----BEGIN PRIVATE KEY-----\nMIIEvQIBADANclientKEYsecret\n-----END PRIVATE KEY-----\n";
pub const TLS_AUTH: &str = "-----BEGIN OpenVPN Static key V1-----\n0f1e2d3c4b5a69788796a5b4c3d2e1f0tlsAUTHsecret\n-----END OpenVPN Static key V1-----\n";
const ENCRYPTED_KEY: &str = "-----BEGIN ENCRYPTED PRIVATE KEY-----\nMIIFHDBOBgkqhkiG9w0BBQ0wQTApencryptedKEY\n-----END ENCRYPTED PRIVATE KEY-----\n";

/// IT's config with every certificate and key inline, as most VPN portals hand them out.
pub fn inline() -> String {
    format!(
        "# Work VPN, 2026\n\
         client\n\
         dev tun\n\
         proto udp\n\
         remote vpn.example.com 1194\n\
         remote vpn2.example.com 443 tcp\n\
         resolv-retry infinite\n\
         nobind\n\
         persist-key\n\
         persist-tun\n\
         remote-cert-tls server\n\
         verify-x509-name \"vpn.example.com\" name\n\
         cipher AES-256-GCM\n\
         data-ciphers AES-256-GCM:CHACHA20-POLY1305\n\
         auth SHA256\n\
         tls-version-min 1.2 or-highest\n\
         keepalive 10 60\n\
         key-direction 1\n\
         verb 3\n\
         up /etc/openvpn/update-resolv-conf\n\
         <ca>\n{CA}</ca>\n\
         <cert>\n{CERT}</cert>\n\
         <key>\n{KEY}</key>\n\
         <tls-auth>\n{TLS_AUTH}</tls-auth>\n"
    )
}

fn parse(text: &str) -> Config {
    Config::parse(text, Path::new("/nonexistent")).expect("parses")
}

fn problem(text: &str) -> Problem {
    Config::parse(text, Path::new("/nonexistent"))
        .expect_err("refused")
        .problem
}

#[test]
fn an_inline_config_gives_its_servers_ca_and_secrets() {
    let config = parse(&inline());

    assert_eq!(
        config.remotes.iter().map(ToString::to_string).collect::<Vec<_>>(),
        ["vpn.example.com:1194:udp", "vpn2.example.com:443:tcp"]
    );
    assert_eq!(config.ca.as_deref(), Some(CA));
    let held = &config.credentials;
    assert_eq!(held.cert.as_ref().map(Secret::expose), Some(CERT));
    assert_eq!(held.key.as_ref().map(Secret::expose), Some(KEY));
    assert_eq!(held.tls_auth.as_ref().map(Secret::expose), Some(TLS_AUTH));
    assert_eq!(config.key_direction, Some(1));
    assert!(!config.auth_user_pass);
    assert!(!config.is_full());
    assert_eq!(config.ignored, ["up"]);
}

#[test]
fn referenced_files_are_read_beside_the_config() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("ca.crt"), CA).unwrap();
    fs::write(dir.path().join("kuba.crt"), CERT).unwrap();
    fs::write(dir.path().join("kuba.key"), KEY).unwrap();
    fs::create_dir(dir.path().join("keys")).unwrap();
    fs::write(dir.path().join("keys/tc.key"), TLS_AUTH).unwrap();
    fs::write(dir.path().join("login.txt"), "kuba\nhunter2 with spaces\n").unwrap();
    let text = "client\nremote 198.51.100.7\nca ca.crt\ncert kuba.crt\nkey \"kuba.key\"\n\
                tls-crypt keys/tc.key\nauth-user-pass login.txt\n";

    let config = Config::parse(text, dir.path()).expect("parses");

    assert_eq!(config.ca.as_deref(), Some(CA));
    let held = &config.credentials;
    assert_eq!(held.cert.as_ref().map(Secret::expose), Some(CERT));
    assert_eq!(held.key.as_ref().map(Secret::expose), Some(KEY));
    assert_eq!(held.tls_crypt.as_ref().map(Secret::expose), Some(TLS_AUTH));
    assert_eq!(held.username.as_ref().map(Secret::expose), Some("kuba"));
    assert_eq!(held.password.as_ref().map(Secret::expose), Some("hunter2 with spaces"));
    assert!(config.auth_user_pass);
    assert_eq!(config.remotes[0].to_string(), "198.51.100.7:1194:udp");
}

#[test]
fn a_tls_auth_line_with_its_direction_takes_the_inline_block() {
    let text = format!(
        "client\nremote vpn.example.com\n<ca>\n{CA}</ca>\ntls-auth [inline] 0\n<tls-auth>\n{TLS_AUTH}</tls-auth>\n"
    );

    let config = parse(&text);

    assert_eq!(config.key_direction, Some(0));
    assert!(config.credentials.tls_auth.is_some());
}

#[test]
fn a_file_it_names_that_isnt_there_is_said_with_its_path() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("ca.crt"), CA).unwrap();

    let err = Config::parse(
        "client\nremote vpn.example.com\nca ca.crt\ncert gone.crt\nkey gone.key\n",
        dir.path(),
    )
    .expect_err("refused");

    assert_eq!(err.line, 4);
    assert!(
        matches!(&err.problem, Problem::Read(path, _) if path.ends_with("gone.crt")),
        "{err:?}"
    );
}

#[test]
fn redirect_gateway_or_a_route_to_everything_is_a_full_tunnel() {
    let base = format!("client\nremote vpn.example.com\n<ca>\n{CA}</ca>\n");

    assert!(parse(&format!("{base}redirect-gateway def1 bypass-dhcp\n")).is_full());
    assert!(parse(&format!("{base}route 0.0.0.0 0.0.0.0\n")).is_full());
    assert!(parse(&format!("{base}route-ipv6 ::/0\n")).is_full());
    assert!(!parse(&format!("{base}route 10.0.0.0 255.0.0.0\n")).is_full());
}

#[test]
fn routes_and_dhcp_options_are_the_tunnels_ranges_and_dns() {
    let text = format!(
        "client\nremote vpn.example.com\n<ca>\n{CA}</ca>\n\
         route 10.20.0.0 255.255.0.0 vpn_gateway\nroute 192.168.50.7\nroute-ipv6 fd00:20::/48\n\
         dhcp-option DNS 10.20.0.53\ndhcp-option DOMAIN corp.example.com\ndhcp-option NTP 10.20.0.1\n"
    );

    let config = parse(&text);

    assert_eq!(
        config.routes.iter().map(ToString::to_string).collect::<Vec<_>>(),
        ["10.20.0.0/16", "192.168.50.7/32", "fd00:20::/48"]
    );
    assert_eq!(config.dns_servers, ["10.20.0.53".parse::<IpAddr>().unwrap()]);
    assert_eq!(config.dns_domains, ["corp.example.com"]);
    assert_eq!(config.ignored, ["dhcp-option"]);
}

#[test]
fn the_plugin_gets_the_options_in_its_own_names() {
    let config = parse(&inline());

    let data = config.data();

    let get = |key: &str| data.iter().find(|(k, _)| *k == key).map(|(_, v)| v.as_str());
    assert_eq!(
        get("remote"),
        Some("vpn.example.com:1194:udp, vpn2.example.com:443:tcp")
    );
    assert_eq!(get("remote-cert-tls"), Some("server"));
    assert_eq!(get("verify-x509-name"), Some("name:vpn.example.com"));
    assert_eq!(get("cipher"), Some("AES-256-GCM"));
    assert_eq!(get("data-ciphers"), Some("AES-256-GCM:CHACHA20-POLY1305"));
    assert_eq!(get("auth"), Some("SHA256"));
    assert_eq!(get("tls-version-min"), Some("1.2"));
    assert_eq!(get("tls-version-min-or-highest"), Some("yes"));
    assert_eq!(get("ping"), Some("10"));
    assert_eq!(get("ping-restart"), Some("60"));
    assert_eq!(get("verb"), None);
    assert_eq!(get("up"), None);
}

#[test]
fn the_sanitized_config_keeps_the_ca_and_no_secret_and_reads_back_the_same() {
    let config = parse(&inline());

    let sanitized = config.sanitized();
    let text = sanitized.render();

    for secret in [CERT, KEY, TLS_AUTH] {
        assert!(!text.contains(secret.lines().nth(1).unwrap()), "{text}");
    }
    assert!(text.contains(CA.lines().nth(1).unwrap()), "{text}");
    assert!(sanitized.credentials.is_empty());
    let mut back = parse(&text);
    back.ignored.clear();
    assert_eq!(back, sanitized);
}

#[test]
fn the_whole_config_renders_back_with_its_secrets_inline() {
    let config = parse(&inline());

    let back = parse(&config.render());

    assert_eq!(back.credentials, config.credentials);
    assert_eq!(back.remotes, config.remotes);
    assert_eq!(back.options, config.options);
}

#[test]
fn secrets_never_print_nor_reach_an_error() {
    let config = parse(&inline());
    let bad_key = inline().replace("-----BEGIN PRIVATE KEY-----\n", "");

    let printed = format!("{config:?}");
    let err = Config::parse(&bad_key, Path::new("/nonexistent")).expect_err("refused");

    for secret in ["clientKEYsecret", "tlsAUTHsecret", "MIIBcLIENTcertificate"] {
        assert!(!printed.contains(secret), "{printed}");
    }
    let said = format!("{err}: {}", err.source().map(ToString::to_string).unwrap_or_default());
    assert!(!said.contains("clientKEYsecret"), "{said}");
    assert!(said.contains("<key> isn't PEM"), "{said}");
}

/// Whether a refusal is the one expected.
type Expected = fn(&Problem) -> bool;

#[test]
fn what_kubaos_cant_run_is_refused_and_says_why() {
    let ca = format!("<ca>\n{CA}</ca>\n");
    let cases: [(String, Expected); 11] = [
        (format!("client\ndev tap\nremote vpn.example.com\n{ca}"), |p| {
            matches!(p, Problem::Tap)
        }),
        (format!("client\nremote vpn.example.com\npkcs12 me.p12\n{ca}"), |p| {
            matches!(p, Problem::Pkcs12)
        }),
        (
            format!("client\nremote vpn.example.com\nsecret static.key\n{ca}"),
            |p| matches!(p, Problem::StaticKey),
        ),
        (
            format!("client\n<connection>\nremote vpn.example.com\n</connection>\n{ca}"),
            |p| matches!(p, Problem::Connection),
        ),
        (format!("remote vpn.example.com\n{ca}"), |p| {
            matches!(p, Problem::NotClient)
        }),
        (format!("client\n{ca}"), |p| matches!(p, Problem::NoRemote)),
        ("client\nremote vpn.example.com\n".to_owned(), |p| {
            matches!(p, Problem::NoCa)
        }),
        (
            format!("client\nremote vpn.example.com\n{ca}<cert>\n{CERT}</cert>\n"),
            |p| matches!(p, Problem::HalfPair),
        ),
        (format!("client\nremote vpn.example.com\n{ca}<key>\n{KEY}"), |p| {
            matches!(p, Problem::Unclosed(_))
        }),
        (format!("client\nremote $(reboot)\n{ca}"), |p| {
            matches!(p, Problem::BadValue { .. })
        }),
        (format!("client\nremote vpn.example.com 1194 sctp\n{ca}"), |p| {
            matches!(p, Problem::BadValue { .. })
        }),
    ];
    for (text, expected) in cases {
        let problem = problem(&text);

        assert!(expected(&problem), "{text:?}: {problem:?}");
    }
}

#[test]
fn quotes_and_comments_split_as_openvpn_splits_them() {
    assert_eq!(
        words(r#"verify-x509-name "CN=VPN \"main\"" subject # IT's"#).unwrap(),
        ["verify-x509-name", "CN=VPN \"main\"", "subject"]
    );
    assert_eq!(
        words("remote vpn.example.com ; the backup").unwrap(),
        ["remote", "vpn.example.com"]
    );
    assert_eq!(words("cipher 'AES-256-GCM'").unwrap(), ["cipher", "AES-256-GCM"]);
    assert!(matches!(words("cipher \"AES"), Err(Problem::Quote)));
    assert_eq!(quoted("CN=VPN \"main\""), r#""CN=VPN \"main\"""#);
}

#[test]
fn a_login_or_a_passphrase_it_lacks_is_named() {
    let base = format!("client\nremote vpn.example.com\n<ca>\n{CA}</ca>\n");
    let login = parse(&format!("{base}auth-user-pass\n"));
    let encrypted = parse(&format!("{base}<cert>\n{CERT}</cert>\n<key>\n{ENCRYPTED_KEY}</key>\n"));
    let plain = parse(&format!("{base}<cert>\n{CERT}</cert>\n<key>\n{KEY}</key>\n"));

    assert_eq!(login.credentials.missing(&login), ["username", "password"]);
    assert_eq!(encrypted.credentials.missing(&encrypted), ["cert-pass"]);
    assert!(plain.credentials.missing(&plain).is_empty());
}

#[test]
fn a_static_key_keeps_its_key_and_leaves_the_comments_genkey_writes_above_it() {
    let commented = format!("#\n# 2048 bit OpenVPN static key\n#\n{TLS_AUTH}");
    let text = format!("client\nremote vpn.example.com\n<ca>\n{CA}</ca>\n<tls-auth>\n{commented}</tls-auth>\n");

    let config = parse(&text);

    assert_eq!(config.credentials.tls_auth.as_ref().map(Secret::expose), Some(TLS_AUTH));
}

#[test]
fn text_before_a_pem_block_that_isnt_a_comment_is_refused() {
    let text = format!("client\nremote vpn.example.com\n<ca>\nsurprise\n{CA}</ca>\n");

    assert!(matches!(problem(&text), Problem::NotPem(name) if name == "ca"));
}
