use super::*;
use crate::vpn;
use crate::wireguard::tests::PEER;

fn cidr(text: &str) -> Cidr {
    Cidr::parse(text).unwrap()
}

fn cidrs(texts: &[&str]) -> Vec<Cidr> {
    texts.iter().map(|text| cidr(text)).collect()
}

/// What a full tunnel whose peer routes `allowed` too routes.
fn tunnel(allowed: &str) -> Vec<Cidr> {
    let config = crate::wireguard::Config::parse(&format!(
        "[Interface]\nPrivateKey = {PEER}\nAddress = 10.8.0.2/24\n\
         [Peer]\nPublicKey = {PEER}\nEndpoint = vpn.example.com:51820\nAllowedIPs = 0.0.0.0/0, ::/0{allowed}\n"
    ))
    .expect("the tunnel");
    vpn::Config::WireGuard(config).routes()
}

/// What `ip -json address show up` says of a laptop on Wi-Fi with Docker running and the tunnel
/// up.
const IP: &str = r#"[
  {"ifindex":1,"ifname":"lo","addr_info":[
    {"family":"inet","local":"127.0.0.1","prefixlen":8,"scope":"host"},
    {"family":"inet6","local":"::1","prefixlen":128,"scope":"host"}]},
  {"ifindex":3,"ifname":"wlp0s20f3","addr_info":[
    {"family":"inet","local":"192.168.1.23","prefixlen":24,"broadcast":"192.168.1.255","scope":"global"},
    {"family":"inet6","local":"2a02:a311:1:2:5054:ff:fe12:3456","prefixlen":64,"scope":"global"},
    {"family":"inet6","local":"fe80::5054:ff:fe12:3456","prefixlen":64,"scope":"link"}]},
  {"ifindex":4,"ifname":"docker0","addr_info":[
    {"family":"inet","local":"172.17.0.1","prefixlen":16,"scope":"global"}]},
  {"ifindex":5,"ifname":"wg-kubaos","addr_info":[
    {"family":"inet","local":"10.8.0.2","prefixlen":24,"scope":"global"}]}
]"#;

#[test]
fn the_ranges_are_the_interfaces_but_loopback_the_tunnel_and_the_bridges() {
    let found = parse(IP);

    assert_eq!(
        found,
        cidrs(&[
            "192.168.1.23/24",
            "2a02:a311:1:2:5054:ff:fe12:3456/64",
            "fe80::5054:ff:fe12:3456/64"
        ])
    );
}

#[test]
fn what_ip_says_wrong_is_no_range() {
    assert!(parse("").is_empty());
    assert!(parse("{}").is_empty());
    assert!(parse(r#"[{"ifname":"eth0","addr_info":[{"local":"not an address","prefixlen":24}]}]"#).is_empty());
    assert!(parse(r#"[{"ifname":"eth0","addr_info":[{"local":"10.0.0.2","prefixlen":33}]}]"#).is_empty());
}

#[test]
fn the_kill_switch_lets_through_each_network_by_its_first_address() {
    let networks = let_through(&parse(IP), &tunnel(""));

    assert_eq!(networks, cidrs(&["192.168.1.0/24", "2a02:a311:1:2::/64"]));
}

#[test]
fn a_local_network_the_tunnel_routes_stays_behind_the_kill_switch() {
    let found = cidrs(&["192.168.1.23/24", "192.168.10.5/24", "10.20.0.4/16"]);

    let inside_a_route = let_through(&found, &tunnel(", 192.168.0.0/16"));
    let holding_a_route = let_through(&found, &tunnel(", 192.168.1.128/25"));
    let on_the_tunnels_own_range = let_through(&cidrs(&["10.8.0.77/16"]), &tunnel(""));

    assert_eq!(inside_a_route, cidrs(&["10.20.0.0/16"]));
    assert_eq!(holding_a_route, cidrs(&["10.20.0.0/16", "192.168.10.0/24"]));
    assert!(on_the_tunnels_own_range.is_empty());
}

#[test]
fn a_network_inside_another_is_let_through_once_by_the_wider() {
    let found = cidrs(&["172.17.2.3/16", "172.0.0.9/8", "172.0.0.10/8"]);

    let networks = let_through(&found, &tunnel(""));

    assert_eq!(networks, cidrs(&["172.0.0.0/8"]));
}

#[test]
fn a_whole_family_loopback_multicast_and_link_local_are_never_a_local_network() {
    let found = cidrs(&[
        "0.0.0.0/0",
        "::/0",
        "224.0.0.1/24",
        "169.254.3.4/16",
        "127.0.0.1/8",
        "::1/128",
        "::ffff:192.168.1.5/120",
        "ff02::1/16",
    ]);

    assert!(let_through(&found, &tunnel("")).is_empty());
}

#[test]
fn at_most_the_helpers_count_go_through() {
    let found: Vec<Cidr> = (0..40).map(|n| cidr(&format!("10.0.{n}.1/24"))).collect();

    assert_eq!(let_through(&found, &tunnel("")).len(), MAX_NETWORKS);
}

#[test]
fn the_routes_on_the_vpns_interface_are_its_ranges_but_the_default_and_the_kernels_own() {
    let json = r#"[
      {"dst":"default","dev":"tun-kubaos","flags":[]},
      {"dst":"10.20.0.0/16","gateway":"10.8.0.1","dev":"tun-kubaos","flags":[]},
      {"dst":"192.168.1.0/24","dev":"tun-kubaos","flags":[]},
      {"dst":"10.8.0.0/24","dev":"tun-kubaos","protocol":"kernel","scope":"link","prefsrc":"10.8.0.2","flags":[]},
      {"type":"local","dst":"10.8.0.2","table":"local","dev":"tun-kubaos","protocol":"kernel","scope":"host","flags":[]},
      {"type":"broadcast","dst":"10.8.0.255","table":"local","dev":"tun-kubaos","flags":[]},
      {"dst":"fe80::/64","dev":"tun-kubaos","protocol":"kernel","flags":[]}
    ]"#;

    let routed = parse_routes(json);

    assert_eq!(
        routed,
        cidrs(&["10.20.0.0/16", "192.168.1.0/24", "10.8.0.0/24", "fe80::/64"])
    );
    assert!(parse_routes("").is_empty());
}

#[test]
fn a_range_a_server_pushed_stays_behind_the_vpn() {
    let found = parse(IP);
    let pushed = parse_routes(r#"[{"dst":"192.168.0.0/16","dev":"tun-kubaos","flags":[]}]"#);

    let networks = let_through(&found, &pushed);

    assert_eq!(networks, cidrs(&["2a02:a311:1:2::/64"]));
}

#[test]
fn the_openvpn_interface_never_counts_as_a_local_network() {
    let json = r#"[{"ifname":"tun-kubaos","addr_info":[{"local":"10.8.0.2","prefixlen":24}]}]"#;

    assert!(parse(json).is_empty());
}
