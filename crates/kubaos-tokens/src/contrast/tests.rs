use super::ratio;
use super::readings;
use crate::model::Need;

const TOKENS: &str = include_str!("../../../../design/tokens.toml");

#[test]
fn black_on_white_is_the_largest_contrast() {
    assert!((ratio([0.0; 3], [1.0; 3]) - 21.0).abs() < 0.01);
}

#[test]
fn a_known_pair_has_its_published_ratio() {
    // WebAIM's checker gives #777777 on white 4.48:1.
    let grey = 119.0 / 255.0;

    assert!((ratio([grey; 3], [1.0; 3]) - 4.48).abs() < 0.01);
}

#[test]
fn every_pair_is_read_in_both_modes() {
    let tokens = crate::parse(TOKENS).expect("the repo's tokens hold");

    let found = readings(&tokens);

    assert_eq!(found.len(), tokens.contrast.pairs.len() * 2);
    assert!(found.iter().all(|reading| reading.ratio.is_some()));
}

#[test]
fn a_foreground_faded_toward_its_background_loses_contrast() {
    let tokens = crate::parse(TOKENS).expect("the repo's tokens hold");
    let found = readings(&tokens);
    let of = |opacity: Option<&str>| {
        found
            .iter()
            .find(|r| {
                r.pair.fg == "on_secondary_container"
                    && r.pair.opacity.as_deref() == opacity
                    && r.mode.name() == "light"
            })
            .and_then(|r| r.ratio)
            .expect("the pair is listed")
    };

    assert!(of(Some("subdued_text")) < of(None));
}

/// The surfaces a fixed role is drawn on: a page and every container.
const SURFACES: [&str; 6] = [
    "surface",
    "surface_container_lowest",
    "surface_container_low",
    "surface_container",
    "surface_container_high",
    "surface_container_highest",
];

#[test]
fn every_fixed_role_is_held_as_text_and_as_a_mark_on_every_surface() {
    let tokens = crate::parse(TOKENS).expect("the repo's tokens hold");

    for role in ["sharing", "microphone", "camera", "success"] {
        assert_eq!(
            tokens.color.source.get(role).map(String::as_str),
            Some(""),
            "{role} is ours"
        );
        for bg in SURFACES {
            for need in [Need::Text, Need::Graphic] {
                let held = tokens
                    .contrast
                    .pairs
                    .iter()
                    .any(|pair| pair.fg == role && pair.bg == bg && pair.need == need && pair.opacity.is_none());
                assert!(held, "{role} on {bg} as {need:?}");
            }
        }
    }
}

#[test]
fn success_is_the_camera_s_green() {
    let tokens = crate::parse(TOKENS).expect("the repo's tokens hold");

    for palette in [&tokens.color.dark, &tokens.color.light] {
        let success = palette.get("success").expect("success has a value");
        assert_eq!(Some(success), palette.get("camera"));
    }
}
