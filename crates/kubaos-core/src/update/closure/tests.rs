use super::parse_diff;
use crate::update::event::PackageChange;

fn change(name: &str, from: Option<&str>, to: Option<&str>) -> PackageChange {
    PackageChange {
        name: name.into(),
        from: from.map(Into::into),
        to: to.map(Into::into),
    }
}

#[test]
fn version_changes_are_read_and_size_only_lines_left_out() {
    let output = "btop: 1.4.7 → 1.4.8, \x1b[32;1m-68.3 KiB\x1b[0m\n\
                  kubaos: \x1b[32;1m-858.7 KiB\x1b[0m\n\
                  kubaos-sysmon: \x1b[31;1m16.8 KiB\x1b[0m\n\
                  firefox: 130.0 → 131.0, +1.2 MiB\n";

    assert_eq!(
        parse_diff(output),
        [
            change("btop", Some("1.4.7"), Some("1.4.8")),
            change("firefox", Some("130.0"), Some("131.0")),
        ]
    );
}

#[test]
fn a_new_or_gone_package_has_no_version_on_that_side() {
    let output = "jq: ∅ → 1.7.1, +412.0 KiB\nold-tool: 2.1 → ∅, -1.1 MiB\n";

    assert_eq!(
        parse_diff(output),
        [change("jq", None, Some("1.7.1")), change("old-tool", Some("2.1"), None)]
    );
}

#[test]
fn a_set_of_versions_stays_whole_and_its_size_is_dropped() {
    let output = "glibc: 2.38-44, 2.38-77 → 2.39-52, 2.39-60, +2.0 MiB\n";

    assert_eq!(
        parse_diff(output),
        [change("glibc", Some("2.38-44, 2.38-77"), Some("2.39-52, 2.39-60"))]
    );
}

#[test]
fn a_change_without_a_size_keeps_its_last_version() {
    assert_eq!(
        parse_diff("source: 1.0 → 1.1\n"),
        [change("source", Some("1.0"), Some("1.1"))]
    );
}

#[test]
fn lines_that_arent_a_package_are_left_out() {
    let output = "warning: Git tree '/home/kuba/.config/kubaos/home' is dirty\n\nPlease ensure: that → this\n";

    assert!(parse_diff(output).is_empty());
}
