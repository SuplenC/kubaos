use std::path::Path;

use super::camel;
use super::wrap;
use crate::Target;
use crate::rust::RustModule;

const TOKENS: &str = include_str!("../../../../design/tokens.toml");

fn module() -> String {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    RustModule
        .render(&crate::parse(TOKENS).expect("the repo's tokens hold"), &root)
        .expect("render the module")
}

#[test]
fn snake_case_names_become_camel_case_variants() {
    assert_eq!(camel("body_large"), "BodyLarge");
    assert_eq!(camel("control_md"), "ControlMd");
    assert_eq!(camel("display"), "Display");
}

#[test]
fn docs_wrap_at_spaces_within_the_width() {
    let lines = wrap("one two three four five", 9);

    assert_eq!(lines, ["one two", "three", "four five"]);
}

#[test]
fn every_spacing_step_is_a_variant_with_its_value() {
    let module = module();

    for step in [4, 8, 12, 16, 20, 24, 32, 40, 48, 64] {
        assert!(module.contains(&format!("Space::S{step} => {step},")), "S{step}");
    }
}

#[test]
fn radii_are_roles_not_numbers() {
    let module = module();

    for role in [
        "Radius::Small => Some(4),",
        "Radius::Control => Some(12),",
        "Radius::Surface => Some(24),",
    ] {
        assert!(module.contains(role), "{role}");
    }
    assert!(!module.contains("Radius::R"));
}

#[test]
fn the_pill_radius_has_no_fixed_length() {
    assert!(module().contains("Radius::Full => None,"));
}

#[test]
fn type_styles_run_largest_first() {
    let module = module();

    let display = module.find("Type::Display,").expect("Display");
    let caption = module.find("Type::Caption,").expect("Caption");
    assert!(display < caption);
}

#[test]
fn the_fallback_palettes_carry_matugens_default_colours() {
    let module = module();

    assert!(module.contains("primary: Rgb { r: 0xff, g: 0xb7, b: 0x79 },"));
    assert!(module.contains("primary: Rgb { r: 0x8b, g: 0x4c, b: 0x00 },"));
}

#[test]
fn no_line_of_the_module_passes_the_line_width() {
    for line in module().lines() {
        assert!(line.chars().count() <= 120, "{line}");
    }
}

#[test]
fn every_symbol_names_its_drawing() {
    let module = module();

    assert!(module.contains("Icon::ArrowBack => drawing!(\"arrow_back\"),"));
    assert!(module.contains("Icon::HardDrive => None,"));
    assert!(module.contains("Icon::ArrowBack => Some(\"arrow-left\"),"));
}
