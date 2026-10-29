use serde_json::Value;

use super::session::gvariant_str;
use super::session::sh_quote;
use super::*;
use crate::fake::Fake;
use crate::plan::RootArg;
use crate::plan::tests::whole;
use crate::wireguard::tests::PRIVATE;

const ENDPOINT_IP: &str = "198.51.100.7";

struct World {
    _dir: tempfile::TempDir,
    paths: Paths,
    fake: Fake,
    profile: Profile,
    plan: Plan,
}

/// A home with a whole profile, its tunnel full or split, the tunnel's key in the keyring, and
/// the endpoint's name resolving.
fn world(full: bool) -> World {
    let dir = tempfile::tempdir().expect("a temp dir");
    let paths = Paths::under(dir.path());
    let profile = whole(&paths.network, full);
    let plan = Plan::for_profile(&profile);
    let mut fake = Fake::new();
    fake.keyring_has("key private-key", PRIVATE);
    fake.names
        .insert("vpn.example.com".to_owned(), vec![ENDPOINT_IP.parse().unwrap()]);
    World {
        _dir: dir,
        paths,
        fake,
        profile,
        plan,
    }
}

impl World {
    fn on(&mut self) -> Result<(), Failed> {
        on(&mut self.fake, &self.paths, &self.plan, Some("kuba"))
    }

    fn off(&mut self) -> Result<(), Failed> {
        off(&mut self.fake, &self.paths, Some(&self.plan), Some("kuba"))
    }

    fn reconcile(&mut self) -> Reconciled {
        reconcile(&mut self.fake, &self.paths, Ok(self.profile.clone()), Some("kuba"))
    }

    /// Nothing of the Proxy is left anywhere, and what the user had is back.
    fn all_off(&self) {
        assert!(self.fake.tunnel.is_none(), "the tunnel is up");
        assert!(self.fake.root.is_empty(), "root parts: {:?}", self.fake.root);
        assert!(self.fake.env.is_empty(), "env: {:?}", self.fake.env);
        assert!(self.fake.gsettings.is_empty(), "gsettings: {:?}", self.fake.gsettings);
        assert!(!self.paths.git().exists(), "git's include");
        assert!(!self.paths.env().exists(), "the shells' env file");
        assert!(!self.paths.docker_record().exists(), "the Docker record");
        assert!(!self.paths.gnome_before().exists(), "GNOME's saved settings");
        assert!(!self.paths.env_before().exists(), "the saved variables");
        assert_eq!(applied(&self.paths), []);
        let docker = fs::read_to_string(self.paths.docker_config()).unwrap_or_default();
        assert!(!docker.contains("proxies"), "{docker}");
    }

    /// Every part of the Proxy is in place.
    fn all_on(&self) {
        assert!(self.fake.tunnel.is_some(), "the tunnel is down");
        for part in ["killswitch", "ca", "apt", "docker", "nix", "chrome"] {
            assert!(
                self.fake.root.contains(part),
                "{part} isn't in place: {:?}",
                self.fake.root
            );
        }
        assert_eq!(
            self.fake.env.get("https_proxy").map(String::as_str),
            Some("http://proxy.corp.example.com:3128")
        );
        assert_eq!(
            self.fake
                .gsettings
                .get("org.gnome.system.proxy mode")
                .map(String::as_str),
            Some("'manual'")
        );
        assert!(self.paths.git().exists());
        assert_eq!(applied(&self.paths), Kind::ALL);
    }

    fn endpoint(&self) -> Option<String> {
        let connection = self.fake.tunnel.as_ref()?;
        let Some(tunnel::Prop::Dicts(peers)) = connection["wireguard"].get("peers") else {
            return None;
        };
        match peers[0].get("endpoint") {
            Some(tunnel::Prop::Str(endpoint)) => Some(endpoint.clone()),
            _ => None,
        }
    }
}

#[test]
fn on_puts_every_part_in_place_in_order() {
    let mut world = world(true);

    world.on().expect("on");

    world.all_on();
    let order: Vec<&str> = world
        .fake
        .ran
        .iter()
        .filter_map(|line| {
            [
                "nm up",
                "/usr/bin/pkexec",
                "systemctl --user set-environment",
                "gsettings set",
            ]
            .into_iter()
            .find(|prefix| line.starts_with(prefix))
        })
        .collect();
    assert_eq!(order.first(), Some(&"nm up"));
    assert_eq!(order.get(1), Some(&"/usr/bin/pkexec"));
    assert_eq!(order.last(), Some(&"gsettings set"));
}

#[test]
fn the_first_on_pins_its_values_with_the_password_and_the_next_needs_none() {
    let mut world = world(true);

    world.on().expect("on");
    world.on().expect("on again");

    assert_eq!(world.fake.count("kubaos-proxy-pin"), 1, "{:?}", world.fake.ran);
    assert_eq!(
        world.fake.pins.get("proxy").map(String::as_str),
        Some("http://proxy.corp.example.com:3128")
    );
    assert_eq!(
        world.fake.pins.get("no-proxy").map(String::as_str),
        Some("localhost,127.0.0.1,::1,.corp.example.com")
    );
    assert_eq!(world.fake.pins.get("pac").map(String::as_str), Some("none"));
    assert!(
        world
            .fake
            .pins
            .get("ca")
            .is_some_and(|pem| pem.starts_with("-----BEGIN CERTIFICATE-----"))
    );
}

#[test]
fn a_changed_proxy_pins_again() {
    let mut world = world(false);
    world.on().expect("on");
    for part in &mut world.plan.parts {
        if let Part::Root(args) = part {
            for arg in args.iter_mut() {
                if let RootArg::Apt(proxy)
                | RootArg::Docker { proxy, .. }
                | RootArg::Nix { proxy, .. }
                | RootArg::ChromeProxy { proxy, .. } = arg
                {
                    *proxy = "http://other.example.com:8080".to_owned();
                }
            }
        }
    }

    world.on().expect("on again");

    assert_eq!(world.fake.count("kubaos-proxy-pin"), 2);
}

#[test]
fn a_dismissed_password_prompt_for_the_pins_leaves_nothing_up() {
    let mut world = world(true);
    world.fake.fail_on("kubaos-proxy-pin");

    let err = world.on().expect_err("not pinned");

    assert_eq!(err.kind, Kind::Root);
    world.all_off();
}

#[test]
fn the_ca_reaches_the_helper_on_stdin_never_in_its_arguments() {
    let mut world = world(true);

    world.on().expect("on");

    let (line, stdin) = world
        .fake
        .stdins
        .iter()
        .find(|(line, _)| line.contains("kubaos-proxy-write"))
        .expect("the helper's stdin");
    assert!(line.contains(" ca=- "), "{line}");
    assert!(stdin.starts_with("-----BEGIN CERTIFICATE-----"));
    assert!(!line.contains("BEGIN CERTIFICATE"));
}

#[test]
fn the_shells_env_file_and_gits_include_say_the_proxy() {
    let mut world = world(false);

    world.on().expect("on");

    let env = fs::read_to_string(world.paths.env()).unwrap();
    assert!(
        env.contains("export http_proxy='http://proxy.corp.example.com:3128'\n"),
        "{env}"
    );
    assert!(
        env.contains("export NO_PROXY='localhost,127.0.0.1,::1,.corp.example.com'\n"),
        "{env}"
    );
    let git = fs::read_to_string(world.paths.git()).unwrap();
    assert!(
        git.contains("[http]\n\tproxy = http://proxy.corp.example.com:3128\n"),
        "{git}"
    );
}

#[test]
fn off_takes_every_part_away() {
    let mut world = world(true);
    world.on().expect("on");

    world.off().expect("off");

    world.all_off();
    let first = world.fake.ran.iter().rposition(|line| line == "nm down").unwrap();
    let helper = world
        .fake
        .ran
        .iter()
        .rposition(|line| line.ends_with("kubaos-proxy-write off"))
        .unwrap();
    assert!(
        helper < first,
        "the kill switch went after the tunnel: {:?}",
        world.fake.ran
    );
}

#[test]
fn off_puts_back_the_users_gnome_proxy() {
    let mut world = world(false);
    world
        .fake
        .gsettings
        .insert("org.gnome.system.proxy mode".to_owned(), "'auto'".to_owned());
    world.fake.gsettings.insert(
        "org.gnome.system.proxy autoconfig-url".to_owned(),
        "'http://home.example.com/proxy.pac'".to_owned(),
    );
    let before = world.fake.gsettings.clone();

    world.on().expect("on");
    world.on().expect("on again");
    world.off().expect("off");

    assert_eq!(world.fake.gsettings, before);
}

#[test]
fn off_puts_back_the_users_own_proxy_variables() {
    let mut world = world(false);
    world
        .fake
        .env
        .insert("no_proxy".to_owned(), "mine.example.com".to_owned());
    world.fake.env.insert("PATH".to_owned(), "/usr/bin".to_owned());

    world.on().expect("on");
    assert_eq!(
        world.fake.env.get("no_proxy").map(String::as_str),
        Some("localhost,127.0.0.1,::1,.corp.example.com")
    );
    world.off().expect("off");

    assert_eq!(
        world.fake.env.get("no_proxy").map(String::as_str),
        Some("mine.example.com")
    );
    assert_eq!(world.fake.env.get("http_proxy"), None);
    assert_eq!(world.fake.env.get("PATH").map(String::as_str), Some("/usr/bin"));
}

#[test]
fn docker_gets_back_the_default_proxy_the_user_had() {
    let mut world = world(false);
    fs::create_dir_all(&world.paths.docker).unwrap();
    fs::write(
        world.paths.docker_config(),
        r#"{"auths": {"ghcr.io": {"auth": "c2VjcmV0"}}, "proxies": {"default": {"httpProxy": "http://mine:1"}}}"#,
    )
    .unwrap();

    world.on().expect("on");
    world.on().expect("on again");
    let config: Value = serde_json::from_str(&fs::read_to_string(world.paths.docker_config()).unwrap()).unwrap();
    assert_eq!(
        config["proxies"]["default"]["httpProxy"],
        "http://proxy.corp.example.com:3128"
    );
    world.off().expect("off");

    let config: Value = serde_json::from_str(&fs::read_to_string(world.paths.docker_config()).unwrap()).unwrap();
    assert_eq!(config["proxies"]["default"]["httpProxy"], "http://mine:1");
    assert_eq!(config["auths"]["ghcr.io"]["auth"], "c2VjcmV0");
}

#[test]
fn a_part_that_fails_takes_every_part_before_it_away_again() {
    for failing in [
        "nm up",
        "killswitch=on",
        "--user set-environment",
        "gsettings set org.gnome.system.proxy mode 'manual'",
    ] {
        let mut world = world(true);
        if failing == "nm up" {
            world.fake.tunnel_fails = true;
        } else {
            world.fake.fail_on(failing);
        }

        let err = world.on().expect_err(failing);

        assert!(err.going_up);
        assert!(err.left.is_empty(), "{failing}: {:?}", err.left);
        world.all_off();
    }
}

#[test]
fn a_part_is_recorded_before_it_goes_up() {
    let mut world = world(true);
    world.fake.watch = Some(world.paths.applied());

    world.on().expect("on");

    assert!(
        world.fake.seen.iter().all(|applied| applied.contains("root\n")),
        "{:?}",
        world.fake.seen
    );
}

#[test]
fn a_dismissed_polkit_prompt_leaves_nothing_up_and_asks_nothing_more() {
    let mut world = world(true);
    world.fake.pkexec_refuses = true;

    let err = world.on().expect_err("refused");

    assert!(matches!(err.cause, StepError::NotRun), "{err:?}");
    assert!(err.left.is_empty(), "{:?}", err.left);
    assert_eq!(world.fake.count("pkexec"), 1);
    world.all_off();
}

#[test]
fn a_tunnel_without_its_key_in_the_keyring_brings_nothing_up() {
    let mut world = world(true);
    world.fake.keyring.clear();

    let err = world.on().expect_err("no key");

    assert_eq!(err.kind, Kind::Tunnel);
    assert!(matches!(err.cause, StepError::Keys(_)), "{err:?}");
    assert!(!world.fake.ran_like("/usr/bin/pkexec"));
    world.all_off();
}

#[test]
fn networkmanager_failing_says_how() {
    let mut world = world(true);
    world.fake.tunnel_fails = true;

    let err = world.on().expect_err("dropped");

    assert!(matches!(err.cause, StepError::Tunnel(TunnelError::Dropped)), "{err:?}");
}

#[test]
fn networkmanager_dropping_the_tunnel_names_it_once() {
    let mut world = world(true);
    world.fake.tunnel_fails = true;

    let err = world.on().expect_err("dropped");

    assert_eq!(
        chain(&err),
        "The tunnel couldn't be put in place: NetworkManager dropped it on the way up (journalctl -u \
         NetworkManager says why)"
    );
}

#[test]
fn a_tunnel_without_its_key_names_it_once() {
    let mut world = world(true);
    world.fake.keyring.clear();

    let err = world.on().expect_err("no key");

    assert_eq!(
        chain(&err),
        "The tunnel couldn't be put in place: its keys aren't in the keyring: run kubaos proxy pull (or \
         import IT's config again)"
    );
}

#[test]
fn a_part_that_wont_go_puts_the_others_back_so_the_proxy_stays_whole() {
    let mut world = world(true);
    world.on().expect("on");
    world.fake.fail_on("kubaos-proxy-write off");

    let err = world.off().expect_err("the helper fails");

    assert_eq!(err.kind, Kind::Root);
    assert!(!err.going_up);
    assert!(err.left.is_empty(), "{:?}", err.left);
    world.fake.fail.clear();
    world.all_on();
}

#[test]
fn the_tunnel_takes_its_keys_from_the_keyring() {
    let mut world = world(true);

    world.on().expect("on");

    let connection = world.fake.tunnel.as_ref().unwrap();
    assert_eq!(
        connection["wireguard"]["private-key"],
        tunnel::Prop::Secret(PRIVATE.to_owned())
    );
    assert!(world.fake.ran.iter().all(|line| !line.contains(PRIVATE)));
}

#[test]
fn networkmanager_gets_the_endpoints_address_never_its_name() {
    let mut world = world(true);

    world.on().expect("on");

    assert_eq!(world.endpoint().as_deref(), Some("198.51.100.7:51820"));
}

#[test]
fn with_the_kill_switch_up_the_tunnel_comes_back_on_the_kept_address_without_a_lookup() {
    let mut world = world(true);
    set_wanted(&world.paths, true).unwrap();
    world.on().expect("on");
    world.fake.tunnel = None;
    world.fake.resolver_down = true;
    world.fake.ran.clear();

    let done = world.reconcile();

    assert!(matches!(done, Reconciled::BroughtUp), "{done:?}");
    assert_eq!(world.endpoint().as_deref(), Some("198.51.100.7:51820"));
    assert!(!world.fake.ran_like("resolve"), "{:?}", world.fake.ran);
}

#[test]
fn an_endpoint_that_never_resolved_stops_on() {
    let mut world = world(true);
    world.fake.resolver_down = true;

    let err = world.on().expect_err("no address");

    assert!(matches!(err.cause, StepError::Endpoint(_)), "{err:?}");
    world.all_off();
}

#[test]
fn docker_leaves_a_default_proxy_the_user_changed() {
    let mut world = world(false);
    world.on().expect("on");
    let mut config: Value = serde_json::from_str(&fs::read_to_string(world.paths.docker_config()).unwrap()).unwrap();
    config["proxies"]["default"]["httpProxy"] = Value::from("http://mine:1");
    fs::write(world.paths.docker_config(), config.to_string()).unwrap();

    world.off().expect("off");

    let config: Value = serde_json::from_str(&fs::read_to_string(world.paths.docker_config()).unwrap()).unwrap();
    assert_eq!(config["proxies"]["default"]["httpProxy"], "http://mine:1");
}

#[test]
fn docker_config_that_isnt_json_stops_on_without_touching_it() {
    let mut world = world(false);
    fs::create_dir_all(&world.paths.docker).unwrap();
    fs::write(world.paths.docker_config(), "{ not json").unwrap();

    let err = world.on().expect_err("refused");

    assert_eq!(err.kind, Kind::DockerClient);
    assert_eq!(fs::read_to_string(world.paths.docker_config()).unwrap(), "{ not json");
    world.all_off();
}

#[test]
fn a_part_dropped_from_the_profile_goes_at_the_next_on() {
    let mut world = world(false);
    world.on().expect("on");
    world.plan.parts.retain(|part| part.kind() != Kind::Git);

    world.on().expect("on again");

    assert!(!world.paths.git().exists());
    assert!(!applied(&world.paths).contains(&Kind::Git));
}

#[test]
fn a_root_part_dropped_from_the_profile_goes_at_the_next_on() {
    let mut world = world(true);
    world.on().expect("on");
    for part in &mut world.plan.parts {
        if let Part::Root(args) = part {
            args.retain(|arg| *arg != RootArg::KillSwitch);
        }
    }

    world.on().expect("on again");

    assert!(!world.fake.root.contains("killswitch"), "{:?}", world.fake.root);
    assert!(world.fake.root.contains("apt"));
}

#[test]
fn a_login_takes_away_what_a_session_left_up() {
    let mut world = world(true);
    world.on().expect("on");
    let after_reboot = Fake {
        root: world.fake.root.clone(),
        pins: world.fake.pins.clone(),
        ..Fake::new()
    };
    world.fake = after_reboot;

    let done = world.reconcile();

    assert!(
        matches!(&done, Reconciled::TookDown(left) if left.is_empty()),
        "{done:?}"
    );
    world.all_off();
}

#[test]
fn nothing_switches_on_by_itself() {
    let mut world = world(true);

    let done = world.reconcile();

    assert!(matches!(done, Reconciled::Nothing), "{done:?}");
    assert!(world.fake.ran.is_empty(), "{:?}", world.fake.ran);
}

#[test]
fn after_a_resume_the_tunnel_networkmanager_dropped_comes_back() {
    let mut world = world(true);
    set_wanted(&world.paths, true).unwrap();
    world.on().expect("on");
    world.fake.tunnel = None;

    let done = world.reconcile();

    assert!(matches!(done, Reconciled::BroughtUp), "{done:?}");
    world.all_on();
}

#[test]
fn a_resume_with_everything_up_changes_nothing() {
    let mut world = world(true);
    set_wanted(&world.paths, true).unwrap();
    world.on().expect("on");
    world.fake.ran.clear();

    let done = world.reconcile();

    assert!(matches!(done, Reconciled::Nothing), "{done:?}");
    assert!(
        !world.fake.ran.iter().any(|line| line.contains("pkexec")),
        "{:?}",
        world.fake.ran
    );
}

#[test]
fn a_tunnel_that_cant_come_back_keeps_the_kill_switch_up_and_the_wish() {
    let mut world = world(true);
    set_wanted(&world.paths, true).unwrap();
    world.on().expect("on");
    world.fake.tunnel = None;
    world.fake.tunnel_fails = true;

    let done = world.reconcile();

    assert!(
        matches!(
            done,
            Reconciled::Failed {
                kill_switch_kept: true,
                ..
            }
        ),
        "{done:?}"
    );
    assert!(wanted(&world.paths));
    assert_eq!(world.fake.root.iter().collect::<Vec<_>>(), ["killswitch"]);
    assert_eq!(applied(&world.paths), [Kind::Root]);
    assert!(world.fake.env.is_empty() && world.fake.gsettings.is_empty());

    world.fake.tunnel_fails = false;
    world.off().expect("Proxy off lifts it");
    world.all_off();
}

#[test]
fn a_split_tunnel_that_cant_come_back_takes_everything_down() {
    let mut world = world(false);
    set_wanted(&world.paths, true).unwrap();
    world.on().expect("on");
    world.fake.tunnel = None;
    world.fake.tunnel_fails = true;

    let done = world.reconcile();

    assert!(
        matches!(
            done,
            Reconciled::Failed {
                kill_switch_kept: false,
                ..
            }
        ),
        "{done:?}"
    );
    assert!(!wanted(&world.paths));
    world.all_off();
}

#[test]
fn values_reach_gsettings_as_gvariant_strings() {
    assert_eq!(gvariant_str("proxy.corp"), "'proxy.corp'");
    assert_eq!(gvariant_str(r"a'b\c"), r"'a\'b\\c'");
    assert_eq!(sh_quote("it's"), r"'it'\''s'");
}
