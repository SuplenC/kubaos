use std::path::Path;
use std::path::PathBuf;

use super::QmlSymbols;
use super::QmlTokens;
use super::minify;
use super::recolourable;
use crate::Target;

const TOKENS: &str = include_str!("../../../../design/tokens.toml");

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn render(target: &dyn Target) -> String {
    target
        .render(&crate::parse(TOKENS).expect("the repo's tokens hold"), &root())
        .expect("render the file")
}

#[test]
fn names_stay_as_the_tokens_spell_them() {
    let singleton = render(&QmlTokens);

    assert!(singleton.contains("readonly property int control_md: 40"));
    assert!(singleton.contains("readonly property color on_primary: tokens.role(\"on_primary\")"));
    assert!(singleton.contains("readonly property real disabled_content: 0.38"));
}

#[test]
fn a_shadow_with_a_light_opacity_follows_the_mode_and_one_without_keeps_its_own() {
    let singleton = render(&QmlTokens);

    assert!(singleton.contains("readonly property real opacity: tokens.mode === \"light\" ? 0.16 : 0.4"));
    assert!(singleton.contains("readonly property real opacity: 0.55"));
}

#[test]
fn radii_are_the_roles_and_a_pill() {
    let singleton = render(&QmlTokens);

    for role in [
        "readonly property int small: 4",
        "readonly property int control: 12",
        "readonly property int surface: 24",
        "readonly property int full: 4096",
    ] {
        assert!(singleton.contains(role), "{role}");
    }
}

#[test]
fn curves_end_at_one_one_for_bezier_splines() {
    assert!(render(&QmlTokens).contains("readonly property list<real> emphasized_decel: [0.05, 0.7, 0.1, 1, 1, 1]"));
}

#[test]
fn the_spring_overshoots_in_its_bezier_spline_too() {
    assert!(render(&QmlTokens).contains("readonly property list<real> spring: [0.34, 1.25, 0.64, 1, 1, 1]"));
}

#[test]
fn an_app_window_s_sizes_and_the_sidebar_are_sizes() {
    let singleton = render(&QmlTokens);

    for size in [
        "readonly property int app_width: 1120",
        "readonly property int app_height: 720",
        "readonly property int app_min_width: 720",
        "readonly property int app_min_height: 480",
        "readonly property int sidebar: 264",
    ] {
        assert!(singleton.contains(size), "{size}");
    }
}

#[test]
fn the_fixed_roles_are_written_in_both_palettes() {
    let singleton = render(&QmlTokens);

    for role in [
        "sharing: \"#c9a7ff\"",
        "sharing: \"#6b3fc4\"",
        "sharing_core: \"#e9dbff\"",
        "microphone: \"#ff9f45\"",
        "microphone: \"#a04a00\"",
        "camera: \"#5fd27a\"",
        "camera: \"#1b7131\"",
        "success: \"#5fd27a\"",
        "success: \"#1b7131\"",
        "readonly property color sharing: tokens.role(\"sharing\")",
    ] {
        assert!(singleton.contains(role), "{role}");
    }
}

#[test]
fn type_styles_carry_their_size_line_weight_and_family() {
    let singleton = render(&QmlTokens);

    let body = singleton
        .find("readonly property QtObject body: QtObject {")
        .expect("body");
    let end = singleton[body..].find("\n        }").expect("the style's end");
    let style = &singleton[body..body + end];
    assert!(style.contains("size: 14"));
    assert!(style.contains("line: 20"));
    assert!(style.contains("weight: 400"));
    assert!(style.contains("family: \"Geist\""));
}

#[test]
fn both_fallback_palettes_are_written() {
    let singleton = render(&QmlTokens);

    assert!(singleton.contains("primary: \"#ffb779\""));
    assert!(singleton.contains("primary: \"#8b4c00\""));
}

#[test]
fn every_symbol_is_one_line_of_svg() {
    let symbols = render(&QmlSymbols);

    let check = symbols
        .lines()
        .find(|line| line.trim_start().starts_with("check: "))
        .expect("check");
    assert!(check.contains("stroke-width=\\\"2\\\""));
    assert!(check.contains("currentColor"));
    assert!(!check.contains("class="));
}

#[test]
fn minifying_drops_comments_class_names_and_runs_of_space() {
    let svg = "<!-- ours -->\n<svg\n  class=\"icon\"\n  fill=\"none\"\n>\n  <path d=\"M0 0\" />\n</svg>\n";

    assert_eq!(minify(svg), "<svg fill=\"none\"><path d=\"M0 0\" /></svg>");
}

#[test]
fn a_drawing_without_exactly_one_stroke_and_a_colour_is_refused() {
    let one = r#"<svg stroke="currentColor" stroke-width="2"><path d="M1 1"/></svg>"#;
    let filled = r#"<svg stroke="currentColor" stroke-width="2"><path fill="currentColor"/></svg>"#;
    let no_stroke = r#"<svg stroke="currentColor"><path d="M1 1"/></svg>"#;
    let two_strokes = r#"<svg stroke="currentColor" stroke-width="2"><path stroke-width="2"/></svg>"#;
    let no_colour = r#"<svg stroke="black" stroke-width="2"><path d="M1 1"/></svg>"#;

    assert!(recolourable(one).is_ok());
    assert!(recolourable(filled).is_ok());
    assert!(recolourable(no_stroke).is_err());
    assert!(recolourable(two_strokes).is_err());
    assert!(recolourable(no_colour).is_err());
}
