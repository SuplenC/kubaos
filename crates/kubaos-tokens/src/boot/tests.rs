use std::collections::BTreeSet;
use std::path::Path;

use super::grub_value;
use super::plymouth_value;
use super::references;

const TOKENS: &str = include_str!("../../../../design/tokens.toml");

fn tokens() -> crate::Tokens {
    crate::parse(TOKENS).expect("the repo's tokens hold every rule")
}

fn root() -> &'static Path {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
}

#[test]
fn a_script_names_tokens_wherever_it_reads_them_and_nowhere_else() {
    let mut names = BTreeSet::new();

    references("x = tokens.space.s8 * s + tokens.color.primary.r;", &mut names);
    references("my_tokens.size.row = 1; global.tokens.size.chip = 2;", &mut names);
    references("f(tokens.neon.strike[i], tokens.neon.strike.count)", &mut names);

    let names: Vec<&str> = names.iter().map(String::as_str).collect();
    assert_eq!(
        names,
        ["color.primary.r", "neon.strike", "neon.strike.count", "space.s8"]
    );
}

#[test]
fn a_colour_is_set_as_its_three_channels_from_the_dark_palette() {
    let lines = plymouth_value(&tokens(), "color.surface_container_lowest.r").expect("a colour role");

    assert_eq!(
        lines,
        [
            "tokens.color.surface_container_lowest.r = 0.0784;",
            "tokens.color.surface_container_lowest.g = 0.0510;",
            "tokens.color.surface_container_lowest.b = 0.0275;",
        ]
    );
}

#[test]
fn a_length_a_curve_and_the_strike_are_set_as_the_tokens_hold_them() {
    let tokens = tokens();

    let field = plymouth_value(&tokens, "size.control_md").expect("a size");
    let curve = plymouth_value(&tokens, "easing.standard").expect("a curve");
    let steps = plymouth_value(&tokens, "neon.strike.count").expect("the strike's length");

    assert_eq!(field, ["tokens.size.control_md = 40;"]);
    assert_eq!(curve[0], "tokens.easing.standard[0] = 0.2;");
    assert_eq!(curve.len(), 4);
    assert_eq!(steps, ["tokens.neon.strike.count = 10;"]);
}

#[test]
fn a_script_reading_a_token_that_doesnt_exist_fails_to_generate() {
    let err = plymouth_value(&tokens(), "space.s6").expect_err("6 isn't a spacing step");

    assert!(err.to_string().contains("tokens.space.s6"), "{err}");
}

#[test]
fn grubs_template_takes_colours_fonts_and_sums_at_grubs_scale() {
    let tokens = tokens();

    let colour = grub_value(&tokens, root(), "color.on_surface_variant").expect("a colour");
    let font = grub_value(&tokens, root(), "font.body_strong").expect("a type style");
    let mono = grub_value(&tokens, root(), "font.mono").expect("a type style");
    let sum = grub_value(&tokens, root(), "size.row + space.s4 - type.body_strong.line").expect("a sum");
    let half = grub_value(&tokens, root(), "size.dialog/2").expect("half a size");

    assert_eq!(colour, "#dac2b1");
    assert_eq!(font, "Geist Medium Regular 21");
    assert_eq!(mono, "Geist Mono Regular 18");
    assert_eq!(sum, "42");
    assert_eq!(half, "375");
}

#[test]
fn a_font_size_grub_would_draw_between_pixels_is_refused() {
    let err = grub_value(&tokens(), root(), "font.caption").expect_err("11 at 1.5 is 16.5");

    assert!(err.to_string().contains("font.caption"), "{err}");
}

#[test]
fn grubs_template_refuses_a_name_the_tokens_lack() {
    let err = grub_value(&tokens(), root(), "size.nothing + 4").expect_err("no such size");

    assert!(err.to_string().contains("size.nothing"), "{err}");
}
