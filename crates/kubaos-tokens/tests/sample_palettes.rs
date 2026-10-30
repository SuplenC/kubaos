//! Every contrast pair over the mockups' sample palettes as well as the fallback ones: the labels
//! fitted on their fills hold on any wallpaper, so they are read in each theme the toolbar paints
//! in (`design/mockups/src/site/themes.ts`), with the derived roles computed from its tones.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use kubaos_tokens::contrast::Mode;
use kubaos_tokens::model::Rgb;

/// Each theme's matugen roles in `mode`, read from `themes.ts`'s `dark: { … }` or `light: { … }`
/// objects, by theme in the file's order.
fn themes(source: &str, mode: Mode) -> Vec<BTreeMap<String, Rgb>> {
    let opening = format!("{}: {{", mode.name());
    source
        .match_indices(&opening)
        .map(|(at, _)| {
            let body = &source[at + opening.len()..];
            let body = &body[..body.find('}').expect("each theme's object closes")];
            body.split(',')
                .filter_map(|entry| {
                    let (role, value) = entry.split_once(':')?;
                    Some((role.trim().to_owned(), Rgb::parse(value.trim().trim_matches('"'))?))
                })
                .collect()
        })
        .collect()
}

#[test]
fn every_pair_holds_in_every_sample_palette() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let tokens = kubaos_tokens::load(&root).expect("design/tokens.toml holds every rule");
    let source = fs::read_to_string(root.join("design/mockups/src/site/themes.ts")).expect("read the sample themes");
    let [dark, light] = [Mode::Dark, Mode::Light].map(|mode| themes(&source, mode));
    assert!(
        dark.len() >= 3 && dark.len() == light.len(),
        "{} dark and {} light themes",
        dark.len(),
        light.len()
    );

    let mut failing = Vec::new();
    for (dark, light) in dark.iter().zip(&light) {
        let over = |fallback: &BTreeMap<String, Rgb>, theme: &BTreeMap<String, Rgb>, mode| {
            let mut palette = fallback.clone();
            palette.extend(theme.iter().map(|(role, rgb)| (role.clone(), *rgb)));
            tokens.color.with_derived(&palette, mode)
        };
        let palettes = [
            over(&tokens.color.dark, dark, Mode::Dark),
            over(&tokens.color.light, light, Mode::Light),
        ];
        for reading in kubaos_tokens::contrast::readings_in(&tokens, &palettes) {
            if !reading.holds() {
                failing.push(format!(
                    "{} on {} in {} with primary {:?}: {:?}",
                    reading.pair.fg,
                    reading.pair.bg,
                    reading.mode.name(),
                    palettes[usize::from(reading.mode == Mode::Light)]["primary"],
                    reading.ratio
                ));
            }
        }
    }

    assert!(failing.is_empty(), "{failing:#?}");
}
