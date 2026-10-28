use super::named_roles;
use crate::lint::source::Language;

#[test]
fn a_qml_colour_names_its_role() {
    let found = named_roles("color: K.Tokens.color.on_surface_variant\n", Language::Qml);

    assert_eq!(found, [(1, "on_surface_variant")]);
}

#[test]
fn a_rust_colour_names_its_role_and_a_longer_name_ending_in_c_doesnt() {
    let text = "text(t, Type::Body, c.primary)\nlet x = epic.error;\nlook.colors.surface\n";

    let found = named_roles(text, Language::Rust);

    assert_eq!(found, [(1, "primary"), (3, "surface")]);
}
