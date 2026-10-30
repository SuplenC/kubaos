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
fn a_shadow_s_light_opacity_falls_back_to_its_opacity() {
    let module = module();

    assert!(module.contains("Shadow::Tucked => 0.16,"));
    assert!(module.contains("pub const fn light_opacity(self) -> f32"));
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

#[test]
fn the_fixed_roles_are_written_in_both_palettes() {
    let module = module();

    for role in [
        "sharing: Rgb { r: 0xb4, g: 0x84, b: 0xff },",
        "sharing: Rgb { r: 0x80, g: 0x38, b: 0xd1 },",
        "logo_core_warm: Rgb { r: 0xff, g: 0xe6, b: 0xcf },",
        "logo_core_cold: Rgb { r: 0xdc, g: 0xf1, b: 0xff },",
        "microphone: Rgb { r: 0xff, g: 0x85, b: 0x05 },",
        "microphone: Rgb { r: 0x9e, g: 0x50, b: 0x00 },",
        "recording: Rgb { r: 0xff, g: 0x6d, b: 0x78 },",
        "recording: Rgb { r: 0xcb, g: 0x00, b: 0x3a },",
        "success: Rgb { r: 0x3c, g: 0xc9, b: 0x6a },",
        "success: Rgb { r: 0x00, g: 0x75, b: 0x33 },",
    ] {
        assert!(module.contains(role), "{role}");
    }
}

#[test]
fn the_spring_is_a_curve_like_the_others() {
    assert!(module().contains("Easing::Spring => [0.34, 1.25, 0.64, 1.0],"));
}

#[test]
fn an_app_window_has_a_default_and_a_minimum_size() {
    let module = module();

    for size in [
        "Size::AppWidth => 1120,",
        "Size::AppHeight => 720,",
        "Size::AppMinWidth => 720,",
        "Size::AppMinHeight => 480,",
        "Size::Sidebar => 264,",
        "Size::Rail => 236,",
    ] {
        assert!(module.contains(size), "{size}");
    }
}

#[test]
fn a_core_is_written_as_its_tone_and_how_far_to_white() {
    let module = module();

    assert!(module.contains("pub const PRIMARY_CORE: Derived = Derived { tone: Role::Primary, white: 0.5 };"));
    assert!(!module.contains("    primary_core: "), "no palette holds a fixed core");
}

#[test]
fn a_state_model_recipe_is_not_built_into_the_module_yet() {
    let module = module();

    assert!(
        !module.contains("PRIMARY_UNLIT"),
        "an OKLCH recipe isn't a `Derived` or `Darkened`"
    );
    assert!(
        !module.contains("    primary_unlit: "),
        "nor a fixed value in a palette"
    );
}
