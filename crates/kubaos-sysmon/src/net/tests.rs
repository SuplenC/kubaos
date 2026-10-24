use std::net::Ipv4Addr;

use super::address;
use super::default_interface;
use super::parse_fib_locals;
use super::parse_if_inet6;
use super::parse_net_dev;
use super::parse_routes;

const NET_DEV: &str = "\
Inter-|   Receive                                                |  Transmit
 face |bytes    packets errs drop fifo frame compressed multicast|bytes    packets errs drop fifo colls carrier compressed
    lo: 1000000    1000    0    0    0     0          0         0  1000000    1000    0    0    0     0       0          0
wlp1s0: 52428800   40000    0    0    0     0          0         0  1048576   20000    0    0    0     0       0          0
";

const ROUTE: &str = "\
Iface\tDestination\tGateway \tFlags\tRefCnt\tUse\tMetric\tMask\t\tMTU\tWindow\tIRTT
enp5s0\t00000000\t0100A8C0\t0003\t0\t0\t700\t00000000\t0\t0\t0
wlp1s0\t00000000\t0101A8C0\t0003\t0\t0\t600\t00000000\t0\t0\t0
wlp1s0\t0001A8C0\t00000000\t0001\t0\t0\t600\t00FFFFFF\t0\t0\t0
docker0\t000011AC\t00000000\t0001\t0\t0\t0\t0000FFFF\t0\t0\t0
";

const FIB_TRIE: &str = "\
Main:
  +-- 0.0.0.0/0 3 0 5
     |-- 0.0.0.0
        /0 universe UNICAST
     +-- 127.0.0.0/8 2 0 2
        +-- 127.0.0.0/31 1 0 0
           |-- 127.0.0.0
              /8 host LOCAL
           |-- 127.0.0.1
              /32 host LOCAL
     +-- 172.17.0.0/16 2 0 2
        |-- 172.17.0.1
           /32 host LOCAL
     +-- 192.168.1.0/24 2 0 2
        |-- 192.168.1.0
           /24 link UNICAST
        |-- 192.168.1.42
           /32 host LOCAL
Local:
  +-- 0.0.0.0/0 3 0 5
        |-- 192.168.1.42
           /32 host LOCAL
";

#[test]
fn each_interface_counts_its_bytes_both_ways() {
    let counters = parse_net_dev(NET_DEV);

    assert_eq!(counters.len(), 2);
    assert_eq!(counters[1].name, "wlp1s0");
    assert_eq!((counters[1].received, counters[1].sent), (52_428_800, 1_048_576));
}

#[test]
fn the_active_interface_is_the_default_route_with_the_lowest_metric() {
    let routes = parse_routes(ROUTE);

    assert_eq!(default_interface(&routes), Some("wlp1s0"));
}

#[test]
fn no_default_route_is_no_active_interface() {
    let routes = parse_routes("Iface\tDestination\tGateway\tFlags\tRefCnt\tUse\tMetric\tMask\n");

    assert_eq!(default_interface(&routes), None);
}

#[test]
fn the_ip_is_the_local_address_on_the_interfaces_network() {
    let routes = parse_routes(ROUTE);
    let locals = parse_fib_locals(FIB_TRIE);

    assert_eq!(locals.len(), 3);
    assert_eq!(
        address(&routes, &locals, "wlp1s0"),
        Some(Ipv4Addr::new(192, 168, 1, 42))
    );
    assert_eq!(address(&routes, &locals, "eth9"), None);
}

#[test]
fn an_ipv6_only_interface_shows_its_global_address() {
    let text = "\
fe800000000000000000000000000001 02 40 20 80     eth0
20010db8000000000000000000000042 02 40 00 80     eth0
";

    assert_eq!(parse_if_inet6(text, "eth0").as_deref(), Some("2001:db8::42"));
    assert_eq!(parse_if_inet6(text, "wlan0"), None);
}
