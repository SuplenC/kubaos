use super::Version;

fn v(text: &str) -> Version {
    Version::parse(text).unwrap()
}

#[test]
fn a_version_parses_with_or_without_its_v_and_newline() {
    assert_eq!(v("0.1.2"), v("v0.1.2\n"));
    assert_eq!(v("0.1.2").tag(), "v0.1.2");
    assert_eq!(v("v0.2.0-rc.1").to_string(), "0.2.0-rc.1");
}

#[test]
fn text_that_is_not_three_numbers_is_refused() {
    for bad in ["", "0.1", "0.1.2.3", "0.x.1", "v", "0.1.2-", "0..1", "+1.0.0"] {
        assert!(Version::parse(bad).is_err(), "{bad:?} parsed");
    }
}

#[test]
fn versions_order_by_number_and_a_pre_release_comes_first() {
    assert!(v("0.1.10") > v("0.1.9"));
    assert!(v("0.2.0") > v("0.1.99"));
    assert!(v("0.2.0-rc.1") < v("0.2.0"));
    assert!(v("0.2.0-rc.1") > v("0.1.5"));
    assert!(v("0.2.0-rc.2") > v("0.2.0-rc.1"));
}

#[test]
fn a_patch_is_in_its_series_and_a_minor_is_not() {
    assert!(v("0.1.0").same_series(&v("0.1.7")));
    assert!(!v("0.1.0").same_series(&v("0.2.0")));
    assert!(!v("0.1.0").same_series(&v("1.1.0")));
    assert_eq!(v("0.1.7").series(), "0.1");
}
