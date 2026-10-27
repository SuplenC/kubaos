use super::ratio;
use super::readings;

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
