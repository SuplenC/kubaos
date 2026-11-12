use super::lua_string;
use super::request;

#[test]
fn a_units_escapes_reach_lua_as_written() {
    assert_eq!(
        lua_string(r"app-kotodama-google\x2dchrome@1.service"),
        r#""app-kotodama-google\\x2dchrome@1.service""#
    );
}

#[test]
fn quotes_and_control_characters_stay_inside_the_string() {
    assert_eq!(lua_string("a\"b\nc"), r#""a\"b\010c""#);
}

#[test]
fn the_request_evaluates_the_launch_in_the_lua_config() {
    assert_eq!(
        request("app-kotodama-foot@1.service", "foot"),
        r#"/eval dot.launches.started("app-kotodama-foot@1.service", "foot")"#
    );
}
