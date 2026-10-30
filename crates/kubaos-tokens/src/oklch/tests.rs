use super::Oklch;
use super::fit_text;
use crate::contrast::channels;
use crate::contrast::ratio;
use crate::model::Rgb;

fn rgb(hex: &str) -> Rgb {
    Rgb::parse(hex).expect("a colour")
}

#[test]
fn a_colour_goes_to_oklch_and_back_unchanged() {
    for hex in ["#ffb779", "#8b4c00", "#19120b", "#ffffff", "#000000", "#5fd27a"] {
        assert_eq!(format!("{:?}", Oklch::of(rgb(hex)).to_rgb()), hex);
    }
}

#[test]
fn a_colour_out_of_gamut_loses_chroma_and_keeps_its_hue_and_lightness() {
    let wanted = Oklch {
        l: 0.66,
        c: 0.25,
        h: 27.0,
    };

    let fitted = Oklch::of(wanted.to_rgb());

    assert!(fitted.c < 0.25, "{fitted:?}");
    assert!((fitted.l - 0.66).abs() < 0.005, "{fitted:?}");
    assert!((fitted.h - 27.0).abs() < 1.0, "{fitted:?}");
}

#[test]
fn a_label_already_holding_is_left_as_it_is() {
    let label = rgb("#ffffff");

    assert_eq!(fit_text(label, &[rgb("#000000")], 4.5), label);
}

#[test]
fn a_dark_label_darkens_until_it_holds_on_every_background() {
    let backgrounds = [rgb("#ff413c"), rgb("#ff7f6a")];

    let label = fit_text(rgb("#c02020"), &backgrounds, 4.5);

    for bg in backgrounds {
        assert!(ratio(channels(label), channels(bg)) >= 4.5, "{label:?} on {bg:?}");
    }
    assert!(
        ratio(channels(label), channels(backgrounds[0])) < 4.7,
        "only as far as it takes"
    );
}
