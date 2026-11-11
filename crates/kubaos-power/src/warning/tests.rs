use super::*;

#[test]
fn above_upowers_level_the_toast_names_the_level() {
    assert_eq!(title(5), "Battery at 5 %");
    assert_eq!(body(5), "The laptop powers off at 2 %. Plug it in now.");
    assert_eq!(body(3), "The laptop powers off at 2 %. Plug it in now.");
}

#[test]
fn at_upowers_level_the_toast_says_it_is_seconds_away() {
    assert_eq!(body(2), "The laptop powers off within seconds. Plug it in now.");
    assert_eq!(body(1), "The laptop powers off within seconds. Plug it in now.");
}
