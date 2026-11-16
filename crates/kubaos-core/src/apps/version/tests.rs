use std::cmp::Ordering;

use super::compare;
use super::is_version;

#[test]
fn versions_order_as_dpkg_orders_them() {
    let ordered = [
        "0.9",
        "0.10~rc1",
        "0.10",
        "0.10-1",
        "0.10-1ubuntu1",
        "0.10a",
        "0.10+git1",
        "0.10.0",
        "1.0",
        "1:0.1",
    ];

    for (i, a) in ordered.iter().enumerate() {
        for (j, b) in ordered.iter().enumerate() {
            assert_eq!(compare(a, b), i.cmp(&j), "{a} against {b}");
        }
    }
    assert_eq!(compare("0.010", "0.10"), Ordering::Equal);
}

#[test]
fn a_version_starts_with_a_digit_and_has_dpkgs_characters() {
    for good in ["0.10", "1:2.3-1ubuntu1", "130.0.6723.58-1", "2.4~beta"] {
        assert!(is_version(good), "{good}");
    }
    for bad in ["", "v1.0", "1.0 beta", "x:1.0", "1.0/2", ":1.0"] {
        assert!(!is_version(bad), "{bad}");
    }
}
