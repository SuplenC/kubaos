use super::*;

#[test]
fn an_app_is_named_with_or_without_desktop() {
    assert_eq!(desktop_id("solaar"), "solaar.desktop");
    assert_eq!(desktop_id("solaar.desktop"), "solaar.desktop");
    assert_eq!(desktop_id("org.gnome.Nautilus"), "org.gnome.Nautilus.desktop");
}
