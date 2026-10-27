use std::fs;
use std::path::Path;

use super::*;
use crate::wireguard::tests::PEER;

const CERT: &str = "-----BEGIN CERTIFICATE-----\nMIIB\n-----END CERTIFICATE-----\n";

/// A profile with every part, its tunnel full or split.
pub fn whole(dir: &Path, full: bool) -> Profile {
    let ips = if full { "0.0.0.0/0" } else { "10.0.0.0/8" };
    fs::create_dir_all(dir).unwrap();
    fs::write(
        dir.join("wireguard.conf"),
        format!(
            "[Interface]\nAddress = 10.8.0.2/32\nDNS = 10.0.0.53, corp.example.com\n\
             [Peer]\nPublicKey = {PEER}\nEndpoint = vpn.example.com:51820\nAllowedIPs = {ips}\n"
        ),
    )
    .unwrap();
    fs::write(dir.join("company-ca.crt"), CERT).unwrap();
    Profile::parse(
        "[tunnel]\nconf = \"wireguard.conf\"\n\
         [http]\nproxy = \"http://proxy.corp.example.com:3128\"\nno_proxy = [\".corp.example.com\"]\n\
         [ca]\nfile = \"company-ca.crt\"\n",
        dir,
    )
    .expect("the profile")
}

#[test]
fn every_part_goes_up_tunnel_first_and_the_proxies_last() {
    let dir = tempfile::tempdir().unwrap();

    let plan = Plan::for_profile(&whole(dir.path(), true));

    assert_eq!(
        plan.kinds(),
        [
            Kind::Tunnel,
            Kind::Root,
            Kind::Env,
            Kind::Gnome,
            Kind::DockerClient,
            Kind::Git
        ]
    );
}

#[test]
fn the_root_helper_gets_the_kill_switch_first_and_chrome_last() {
    let dir = tempfile::tempdir().unwrap();
    let plan = Plan::for_profile(&whole(dir.path(), true));

    let Some(Part::Root(args)) = plan.parts.iter().find(|part| part.kind() == Kind::Root) else {
        panic!("a root part");
    };

    let args: Vec<String> = args.iter().map(RootArg::arg).collect();
    let no_proxy = "localhost,127.0.0.1,::1,.corp.example.com";
    assert_eq!(
        args,
        [
            "killswitch=on".to_owned(),
            "ca=-".to_owned(),
            "apt=http://proxy.corp.example.com:3128".to_owned(),
            format!("docker=http://proxy.corp.example.com:3128|{no_proxy}"),
            format!("nix=http://proxy.corp.example.com:3128|{no_proxy}"),
            format!("chrome=proxy|http://proxy.corp.example.com:3128|{no_proxy}"),
        ]
    );
}

#[test]
fn a_split_tunnel_has_no_kill_switch() {
    let dir = tempfile::tempdir().unwrap();

    let plan = Plan::for_profile(&whole(dir.path(), false));

    let Some(Part::Root(args)) = plan.parts.iter().find(|part| part.kind() == Kind::Root) else {
        panic!("a root part");
    };
    assert!(!args.contains(&RootArg::KillSwitch));
}

#[test]
fn the_session_gets_the_proxy_in_both_cases() {
    let dir = tempfile::tempdir().unwrap();
    let plan = Plan::for_profile(&whole(dir.path(), false));

    let Some(Part::Env(vars)) = plan.parts.iter().find(|part| part.kind() == Kind::Env) else {
        panic!("an env part");
    };

    let names: Vec<&str> = vars.iter().map(|(name, _)| name.as_str()).collect();
    assert_eq!(names, ENV_NAMES);
    assert!(vars.contains(&(
        "HTTPS_PROXY".to_owned(),
        "http://proxy.corp.example.com:3128".to_owned()
    )));
    assert!(vars.contains(&(
        "no_proxy".to_owned(),
        "localhost,127.0.0.1,::1,.corp.example.com".to_owned()
    )));
}

#[test]
fn a_pac_file_goes_to_gnome_and_chrome_and_the_rest_keep_the_proxy() {
    let dir = tempfile::tempdir().unwrap();
    let profile = Profile::parse(
        "[http]\nproxy = \"http://proxy:3128\"\npac = \"http://wpad/proxy.pac\"\n",
        dir.path(),
    )
    .unwrap();

    let plan = Plan::for_profile(&profile);

    assert!(plan.parts.contains(&Part::Gnome(Gnome::Auto {
        pac: "http://wpad/proxy.pac".to_owned(),
        ignore: vec!["localhost".to_owned(), "127.0.0.1".to_owned(), "::1".to_owned()],
    })));
    let Some(Part::Root(args)) = plan.parts.iter().find(|part| part.kind() == Kind::Root) else {
        panic!("a root part");
    };
    assert!(args.contains(&RootArg::ChromePac("http://wpad/proxy.pac".to_owned())));
    assert!(args.contains(&RootArg::Apt("http://proxy:3128".to_owned())));
    assert_eq!(plan.warnings, Vec::<String>::new());
}

#[test]
fn a_pac_file_alone_says_what_it_doesnt_reach() {
    let dir = tempfile::tempdir().unwrap();
    let profile = Profile::parse("[http]\npac = \"http://wpad/proxy.pac\"\n", dir.path()).unwrap();

    let plan = Plan::for_profile(&profile);

    assert_eq!(plan.kinds(), [Kind::Root, Kind::Gnome]);
    assert_eq!(plan.warnings.len(), 1);
}

#[test]
fn a_ca_alone_is_one_root_part() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("ca.crt"), CERT).unwrap();
    let profile = Profile::parse("[ca]\nfile = \"ca.crt\"\n", dir.path()).unwrap();

    let plan = Plan::for_profile(&profile);

    assert_eq!(plan.parts, [Part::Root(vec![RootArg::Ca(CERT.to_owned())])]);
}

#[test]
fn a_full_tunnel_without_dns_says_names_wont_resolve() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("wg.conf"),
        format!("[Interface]\nAddress = 10.8.0.2\n[Peer]\nPublicKey = {PEER}\nAllowedIPs = 0.0.0.0/0\n"),
    )
    .unwrap();
    let profile = Profile::parse("[tunnel]\nconf = \"wg.conf\"\n", dir.path()).unwrap();

    let plan = Plan::for_profile(&profile);

    assert_eq!(plan.warnings.len(), 1, "{:?}", plan.warnings);
}

#[test]
fn a_kinds_name_reads_back() {
    for kind in Kind::ALL {
        assert_eq!(Kind::from_name(kind.name()), Some(kind));
    }
}
