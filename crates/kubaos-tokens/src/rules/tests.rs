use crate::Error;
use crate::Violation;

const TOKENS: &str = include_str!("../../../../design/tokens.toml");

/// The repo's tokens with `edit` applied, parsed and checked.
fn tokens_with(edit: impl FnOnce(&mut toml::Table)) -> Result<crate::Tokens, Error> {
    let mut table: toml::Table = toml::from_str(TOKENS).expect("the repo's tokens parse");
    edit(&mut table);
    crate::parse(&toml::to_string(&table).expect("the edited tokens serialise"))
}

/// The rules the edited tokens break.
fn violations(edit: impl FnOnce(&mut toml::Table)) -> Vec<Violation> {
    match tokens_with(edit) {
        Err(Error::Invalid(violations)) => violations,
        other => panic!("expected broken rules, got {other:?}"),
    }
}

fn table<'a>(root: &'a mut toml::Table, path: &[&str]) -> &'a mut toml::Table {
    path.iter().fold(root, |table, key| {
        table
            .get_mut(*key)
            .and_then(toml::Value::as_table_mut)
            .unwrap_or_else(|| panic!("no table {key}"))
    })
}

fn set(root: &mut toml::Table, path: &[&str], key: &str, value: impl Into<toml::Value>) {
    table(root, path).insert(key.into(), value.into());
}

/// Adds `token` to the sizes the repo already allows off the grid.
fn allow_off_grid(root: &mut toml::Table, token: &str) {
    table(root, &["rules"])
        .entry("off_grid")
        .or_insert_with(|| toml::Value::Array(Vec::new()))
        .as_array_mut()
        .expect("rules.off_grid is a list")
        .push(token.into());
}

#[test]
fn the_repos_tokens_hold_every_rule() {
    let tokens = crate::parse(TOKENS);

    assert!(tokens.is_ok(), "{tokens:?}");
}

#[test]
fn a_spacing_step_off_the_grid_is_refused() {
    let found = violations(|t| set(t, &["space"], "values", vec![4, 8, 10, 12, 16, 24, 32]));

    assert_eq!(
        found,
        [Violation::OffGrid {
            token: "space 10".into(),
            value: 10,
            grid: 4
        }]
    );
}

#[test]
fn a_terminal_padding_off_the_spacing_scale_is_refused() {
    let found = violations(|t| set(t, &["terminal"], "padding_x", 10));

    assert_eq!(
        found,
        [Violation::OffScalePadding {
            token: "terminal.padding_x".into(),
            value: 10
        }]
    );
}

#[test]
fn a_terminal_set_in_a_proportional_family_is_refused() {
    let found = violations(|t| set(t, &["terminal"], "type", "body"));

    assert_eq!(found, [Violation::TerminalNotMono { style: "body".into() }]);
}

#[test]
fn dim_text_too_close_to_the_surface_is_refused() {
    let found = violations(|t| set(t, &["terminal"], "dim", 20));

    assert!(
        matches!(found.as_slice(), [Violation::LowContrast { mode: "light", .. }]),
        "{found:?}"
    );
}

#[test]
fn a_radius_role_off_the_grid_is_refused() {
    let found = violations(|t| set(t, &["radius"], "control", 10));

    assert_eq!(
        found,
        [Violation::OffGrid {
            token: "radius.control".into(),
            value: 10,
            grid: 4
        }]
    );
}

#[test]
fn radius_roles_grow_from_small_to_surface() {
    let found = violations(|t| set(t, &["radius"], "control", 28));

    assert_eq!(
        found,
        [Violation::NotAscending {
            token: "radius.small, control, surface".into()
        }]
    );
}

#[test]
fn a_radius_beside_the_roles_is_refused() {
    let found = tokens_with(|t| set(t, &["radius"], "values", vec![8, 16]));

    assert!(matches!(found, Err(Error::Parse(_))), "{found:?}");
}

#[test]
fn an_icon_size_off_the_grid_is_refused() {
    let found = violations(|t| set(t, &["icon"], "sizes", vec![16, 18, 20, 24, 32, 48]));

    assert_eq!(
        found,
        [Violation::OffGrid {
            token: "icon 18".into(),
            value: 18,
            grid: 4
        }]
    );
}

#[test]
fn a_component_size_off_the_grid_is_refused() {
    let found = violations(|t| set(t, &["size"], "switch_height", 26));

    assert_eq!(
        found,
        [Violation::OffGrid {
            token: "size.switch_height".into(),
            value: 26,
            grid: 4
        }]
    );
}

#[test]
fn a_size_listed_as_off_the_grid_may_leave_it() {
    let tokens = tokens_with(|t| {
        set(t, &["size"], "switch_height", 26);
        allow_off_grid(t, "size.switch_height");
    });

    assert!(tokens.is_ok(), "{tokens:?}");
}

#[test]
fn an_off_grid_listing_naming_no_size_is_refused() {
    let found = violations(|t| allow_off_grid(t, "size.no_such_size"));

    assert_eq!(
        found,
        [Violation::UnknownName {
            token: "rules.off_grid".into(),
            name: "size.no_such_size".into()
        }]
    );
}

#[test]
fn a_line_height_off_the_grid_is_refused() {
    let found = violations(|t| set(t, &["type", "body"], "line", 21));

    assert_eq!(
        found,
        [Violation::OffGrid {
            token: "type.body.line".into(),
            value: 21,
            grid: 4
        }]
    );
}

#[test]
fn a_shadow_off_the_grid_is_refused() {
    let found = violations(|t| set(t, &["shadow", "panel"], "y", 18));

    assert_eq!(
        found,
        [Violation::OffGrid {
            token: "shadow.panel.y".into(),
            value: 18,
            grid: 4
        }]
    );
}

#[test]
fn a_focus_offset_off_the_grid_is_refused() {
    let found = violations(|t| set(t, &["focus"], "offset", 3));

    assert_eq!(
        found,
        [Violation::OffGrid {
            token: "focus.offset".into(),
            value: 3,
            grid: 4
        }]
    );
}

#[test]
fn a_font_size_off_the_list_is_refused() {
    let found = violations(|t| set(t, &["type", "title"], "size", 30));

    assert_eq!(
        found,
        [Violation::OffListFontSize {
            token: "type.title".into(),
            size: 30
        }]
    );
}

#[test]
fn a_line_shorter_than_its_letters_is_refused() {
    let found = violations(|t| {
        set(t, &["type", "display"], "line", 44);
        set(t, &["type", "display"], "icon", 32);
    });

    assert_eq!(
        found,
        [Violation::LineShorterThanSize {
            token: "type.display".into(),
            size: 48,
            line: 44
        }]
    );
}

#[test]
fn a_grid_that_misses_device_pixels_at_a_supported_scale_is_refused() {
    let found = violations(|t| set(t, &["rules"], "grid", 2));

    assert!(
        found.contains(&Violation::GridOffDevicePixels { grid: 2, scale: 1.25 }),
        "{found:?}"
    );
}

#[test]
fn a_thin_stroke_under_a_device_pixel_is_refused() {
    let found = violations(|t| {
        set(t, &["stroke"], "thin", 0);
    });

    assert!(
        found.contains(&Violation::StrokeUnderDevicePixel {
            token: "stroke.thin".into(),
            scale: 1.0
        }),
        "{found:?}"
    );
}

#[test]
fn a_scale_out_of_order_is_refused() {
    let found = violations(|t| set(t, &["space"], "values", vec![4, 12, 8, 16, 24, 32]));

    assert_eq!(
        found,
        [Violation::NotAscending {
            token: "space.values".into()
        }]
    );
}

#[test]
fn a_colour_role_missing_from_a_mode_is_refused() {
    let found = violations(|t| {
        table(t, &["color", "light"]).remove("warning");
    });

    assert_eq!(
        found,
        [Violation::RoleMissing {
            role: "warning".into(),
            table: "light"
        }]
    );
}

#[test]
fn a_colour_not_written_as_hex_is_a_parse_error() {
    let result = tokens_with(|t| set(t, &["color", "dark"], "primary", "orange"));

    assert!(matches!(result, Err(Error::Parse(_))), "{result:?}");
}

#[test]
fn an_easing_that_runs_time_backwards_is_refused() {
    let found = violations(|t| set(t, &["easing", "standard"], "curve", vec![1.2, 0.0, 0.0, 1.0]));

    assert_eq!(
        found,
        [Violation::CurveOutOfRange {
            token: "easing.standard".into()
        }]
    );
}

#[test]
fn a_light_opacity_over_one_is_refused() {
    let found = violations(|t| set(t, &["shadow", "tucked"], "light_opacity", 1.5));

    assert_eq!(
        found,
        [Violation::OpacityOutOfRange {
            token: "shadow.tucked.light_opacity".into(),
            value: 1.5
        }]
    );
}

#[test]
fn an_opacity_over_one_is_refused() {
    let found = violations(|t| set(t, &["opacity"], "hover", 1.5));

    assert_eq!(
        found,
        [Violation::OpacityOutOfRange {
            token: "opacity.hover".into(),
            value: 1.5
        }]
    );
}

#[test]
fn a_name_that_cant_become_an_identifier_is_refused() {
    let found = violations(|t| {
        let size = table(t, &["size"]);
        let row = size.remove("row").expect("size.row");
        size.insert("Row-Height".into(), row);
    });

    assert_eq!(
        found,
        [Violation::BadName {
            token: "size.Row-Height".into()
        }]
    );
}

#[test]
fn the_hero_size_is_on_the_list() {
    let tokens = crate::parse(TOKENS).expect("the repo's tokens hold");

    assert!(tokens.rules.font_sizes.contains(&64));
    assert_eq!(tokens.types["hero"].size, 64);
}

#[test]
fn a_size_between_display_and_hero_is_refused() {
    let found = violations(|t| set(t, &["type", "hero"], "size", 56));

    assert_eq!(
        found,
        [Violation::OffListFontSize {
            token: "type.hero".into(),
            size: 56
        }]
    );
}

#[test]
fn a_symbol_needs_exactly_one_source() {
    let found = violations(|t| {
        let mut both = toml::Table::new();
        both.insert("tabler".into(), "check".into());
        both.insert("drawn".into(), "a tick".into());
        set(t, &["symbols"], "check", toml::Value::Table(both));
    });

    assert_eq!(
        found,
        [Violation::SymbolSource {
            token: "symbols.check".into()
        }]
    );
}

#[test]
fn numbers_take_the_hero_size_in_light() {
    let tokens = crate::parse(TOKENS).expect("the repo's tokens hold");

    let number = &tokens.types["hero_number"];
    assert_eq!(
        (number.size, number.line),
        (tokens.types["hero"].size, tokens.types["hero"].line)
    );
    assert_eq!(number.weight, crate::model::Weight::Light);
}

#[test]
fn a_type_styles_icon_must_be_an_icon_size() {
    let found = violations(|t| set(t, &["type", "body"], "icon", 12));

    assert_eq!(
        found,
        [Violation::IconNotASize {
            token: "type.body".into(),
            icon: 12
        }]
    );
}

#[test]
fn an_icon_taller_than_its_line_is_refused() {
    let found = violations(|t| set(t, &["type", "caption"], "icon", 20));

    assert_eq!(
        found,
        [Violation::IconTallerThanLine {
            token: "type.caption".into(),
            icon: 20,
            line: 16
        }]
    );
}

#[test]
fn a_container_padding_off_the_spacing_scale_is_refused() {
    let found = violations(|t| set(t, &["container"], "padding", vec![16, 28, 32]));

    assert!(found.contains(&Violation::OffScalePadding {
        token: "container.padding 28".into(),
        value: 28
    }));
}

#[test]
fn the_containers_steps_start_at_no_width() {
    let found = violations(|t| set(t, &["container"], "from", vec![320, 640, 1024]));

    assert_eq!(found, [Violation::ContainerSteps]);
}

#[test]
fn a_pair_under_its_contrast_minimum_is_refused() {
    let found = violations(|t| set(t, &["opacity"], "subdued_text", 0.5));

    assert!(
        found.iter().any(|v| matches!(
            v,
            Violation::LowContrast { token, mode: "light", .. }
                if token == "contrast on_secondary_container at subdued_text on secondary_container"
        )),
        "{found:?}"
    );
}

#[test]
fn a_pair_naming_no_role_is_refused() {
    let found = violations(|t| {
        let pairs = table(t, &["contrast"])
            .get_mut("pairs")
            .and_then(toml::Value::as_array_mut)
            .expect("contrast.pairs");
        let mut pair = toml::Table::new();
        pair.insert("fg".into(), "on_glass".into());
        pair.insert("bg".into(), "surface".into());
        pair.insert("need".into(), "text".into());
        pair.insert("use".into(), "nothing".into());
        pairs.push(toml::Value::Table(pair));
    });

    assert_eq!(
        found,
        [Violation::UnknownInPair {
            token: "contrast on_glass on surface".into(),
            name: "on_glass".into()
        }]
    );
}

/// The `index`th stroke of the mark, to edit.
fn mark_stroke(root: &mut toml::Table, index: usize) -> &mut toml::Table {
    table(root, &["neon", "mark"])
        .get_mut("strokes")
        .and_then(toml::Value::as_array_mut)
        .and_then(|strokes| strokes.get_mut(index))
        .and_then(toml::Value::as_table_mut)
        .expect("neon.mark.strokes has the stroke")
}

#[test]
fn a_strike_brighter_than_lit_is_refused() {
    let found = violations(|t| {
        let strike = table(t, &["neon"])
            .get_mut("strike")
            .and_then(toml::Value::as_array_mut);
        strike.expect("neon.strike is a list")[3] = toml::Value::Float(1.5);
    });

    assert_eq!(found, vec![Violation::StrikeOutOfRange { index: 3, value: 1.5 }]);
}

#[test]
fn a_mark_stroke_with_a_command_its_readers_lack_is_refused() {
    let found = violations(|t| {
        mark_stroke(t, 0).insert("path".into(), "M29 13 C17 33 1 2 3 4".into());
    });

    assert_eq!(found, vec![Violation::BadMarkPath { index: 0 }]);
}

#[test]
fn a_mark_stroke_that_doesnt_start_with_a_move_is_refused() {
    let found = violations(|t| {
        mark_stroke(t, 5).insert("path".into(), "L90 11 L94 22".into());
    });

    assert_eq!(found, vec![Violation::BadMarkPath { index: 5 }]);
}

#[test]
fn a_mark_stroke_missing_a_number_is_refused() {
    let found = violations(|t| {
        mark_stroke(t, 2).insert("path".into(), "M66 41 Q64.5 63 53".into());
    });

    assert_eq!(found, vec![Violation::BadMarkPath { index: 2 }]);
}

#[test]
fn a_core_wider_than_its_tube_is_refused() {
    let found = violations(|t| set(t, &["neon", "mark"], "core", 1.2));

    assert_eq!(
        found,
        vec![Violation::BadMarkSize {
            token: "neon.mark.core".into()
        }]
    );
}

// Regression: #203.
#[test]
fn a_frame_blur_cut_off_under_the_scrim_and_shadow_is_refused() {
    let found = violations(|t| set(t, &["opacity"], "frame_blur", 0.5));

    assert!(
        matches!(found.as_slice(), [Violation::FrameBlurOffGlass { cutoff, .. }] if (*cutoff - 0.5).abs() < f32::EPSILON),
        "{found:?}"
    );
}

#[test]
fn a_frame_blur_cut_off_over_the_glass_is_refused() {
    let found = violations(|t| set(t, &["opacity"], "frame_blur", 0.96));

    assert!(
        matches!(found.as_slice(), [Violation::FrameBlurOffGlass { .. }]),
        "{found:?}"
    );
}

#[test]
fn a_darker_scrim_moves_the_frame_blur_cut_off_up_with_it() {
    let found = violations(|t| set(t, &["opacity"], "scrim", 0.75));

    assert!(
        matches!(found.as_slice(), [Violation::FrameBlurOffGlass { .. }]),
        "{found:?}"
    );
}

#[test]
fn every_core_is_its_tone_half_way_to_white() {
    let tokens = tokens_with(|_| {}).expect("the repo's tokens hold");
    let dark = tokens.color.with_derived(&tokens.color.dark);

    for (role, core) in [
        ("primary_core", "#ffdbbc"),
        ("cold_core", "#c2e7ff"),
        ("error_core", "#ffdad5"),
        ("success_core", "#afe9bd"),
        ("warning_core", "#f9e0a7"),
        ("sharing_core", "#e4d3ff"),
    ] {
        assert_eq!(format!("{:?}", dark[role]), core, "{role}");
    }
}

#[test]
fn a_core_follows_a_primary_the_wallpaper_set() {
    let tokens = tokens_with(|_| {}).expect("the repo's tokens hold");
    let mut live = tokens.color.dark.clone();
    live.insert("primary".into(), crate::model::Rgb::parse("#3a7bd5").expect("a colour"));

    let core = tokens.color.with_derived(&live)["primary_core"];

    assert_eq!(format!("{core:?}"), "#9dbdea");
}

#[test]
fn the_logo_keeps_its_own_fixed_cores() {
    let tokens = tokens_with(|_| {}).expect("the repo's tokens hold");

    for palette in [&tokens.color.dark, &tokens.color.light] {
        assert_eq!(format!("{:?}", palette["logo_core_warm"]), "#ffe6cf");
        assert_eq!(format!("{:?}", palette["logo_core_cold"]), "#dcf1ff");
    }
}

#[test]
fn a_core_of_a_role_that_doesnt_exist_is_refused() {
    let found = violations(|t| {
        let mut core = toml::Table::new();
        core.insert("mix".into(), "nowhere".into());
        core.insert("white".into(), 0.5.into());
        set(t, &["color", "derived"], "primary_core", core);
    });

    assert!(
        matches!(found.as_slice(), [Violation::UnknownName { name, .. }] if name == "nowhere"),
        "{found:?}"
    );
}

#[test]
fn every_unlit_core_is_its_tone_35_percent_toward_black() {
    let tokens = tokens_with(|_| {}).expect("the repo's tokens hold");
    let light = tokens.color.with_derived(&tokens.color.light);

    for (role, core) in [("primary_unlit", "#5a3100"), ("error_unlit", "#791111"), ("cold_unlit", "#00405a")] {
        assert_eq!(format!("{:?}", light[role]), core, "{role}");
    }
}

#[test]
fn an_unlit_core_follows_a_primary_the_wallpaper_set() {
    let tokens = tokens_with(|_| {}).expect("the repo's tokens hold");
    let mut live = tokens.color.light.clone();
    live.insert("primary".into(), crate::model::Rgb::parse("#3a7bd5").expect("a colour"));

    let core = tokens.color.with_derived(&live)["primary_unlit"];

    assert_eq!(format!("{core:?}"), "#26508a");
}

#[test]
fn a_derived_role_with_both_ways_or_neither_is_refused() {
    for ways in [&["white", "black"][..], &[][..]] {
        let found = violations(|t| {
            let mut core = toml::Table::new();
            core.insert("mix".into(), "primary".into());
            for way in ways {
                core.insert((*way).into(), 0.5.into());
            }
            set(t, &["color", "derived"], "primary_core", core);
        });

        assert!(
            matches!(found.as_slice(), [Violation::DerivedWay { role }] if role == "primary_core"),
            "{ways:?}: {found:?}"
        );
    }
}
