use super::*;

fn accept(kinds: &[&str]) -> Vec<String> {
    kinds.iter().map(|&kind| kind.to_owned()).collect()
}

#[test]
fn an_extension_is_a_glob_over_the_name_in_either_case() {
    let patterns = patterns(&accept(&["conf", "ovpn"]));

    assert_eq!(
        patterns,
        [
            (GLOB, "*.[cC][oO][nN][fF]".to_owned()),
            (GLOB, "*.[oO][vV][pP][nN]".to_owned())
        ]
    );
}

#[test]
fn a_leading_dot_is_left_off_an_extension() {
    assert_eq!(patterns(&accept(&[".jpg"])), [(GLOB, "*.[jJ][pP][gG]".to_owned())]);
}

#[test]
fn a_word_with_a_slash_is_a_mime_type() {
    assert_eq!(patterns(&accept(&["image/*"])), [(MIME, "image/*".to_owned())]);
}

#[test]
fn a_digit_in_an_extension_stays_as_it_is() {
    assert_eq!(patterns(&accept(&["mp4"])), [(GLOB, "*.[mM][pP]4".to_owned())]);
}

#[test]
fn no_accept_is_no_filter() {
    assert!(patterns(&[]).is_empty());
}
