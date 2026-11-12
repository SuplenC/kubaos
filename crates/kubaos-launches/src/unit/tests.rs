use super::app_id;

#[test]
fn a_launchers_service_names_its_app() {
    assert_eq!(
        app_id(r"app-kotodama-google\x2dchrome@3f2a9c0b11d4e5f6.service").as_deref(),
        Some("google-chrome")
    );
}

#[test]
fn a_service_with_no_launcher_named_names_its_app() {
    assert_eq!(app_id("app-firefox@autostart.service").as_deref(), Some("firefox"));
}

#[test]
fn a_scope_names_its_app() {
    assert_eq!(
        app_id("app-gnome-org.gnome.Nautilus-4021.scope").as_deref(),
        Some("org.gnome.Nautilus")
    );
}

#[test]
fn an_app_id_keeps_its_dots_and_dashes() {
    assert_eq!(
        app_id(r"app-kotodama-com.mitchellh.ghostty\x2dnightly@1.service").as_deref(),
        Some("com.mitchellh.ghostty-nightly")
    );
}

#[test]
fn units_that_run_no_app_name_none() {
    for unit in [
        "kubaos-automount.service",
        "app.slice",
        "app-kotodama-foot.service",
        "app-kotodama-foot.socket",
        "app-@1.service",
        "dbus.service",
    ] {
        assert_eq!(app_id(unit), None, "{unit}");
    }
}
