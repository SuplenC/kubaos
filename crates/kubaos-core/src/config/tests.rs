use super::*;

fn locale(fields: &str) -> Locale {
    toml::from_str(&format!(
        "lang = \"en_US.UTF-8\"\ntimezone = \"UTC\"\n{fields}"
    ))
    .expect("parse [locale]")
}

#[test]
fn an_old_config_with_one_layout_still_reads() {
    let l = locale("keyboard_layout = \"it\"\nkeyboard_variant = \"\"");

    assert_eq!(l.keyboards(), vec![Layout::new("it", "")]);
}

#[test]
fn a_list_of_layouts_reads_with_variants_by_position() {
    let l = locale("keyboard_layout = [\"us\", \"it\"]\nkeyboard_variant = [\"intl\"]");

    assert_eq!(
        l.keyboards(),
        vec![Layout::new("us", "intl"), Layout::new("it", "")]
    );
}

#[test]
fn one_string_reads_as_the_xkb_field() {
    let l = locale("keyboard_layout = \"us,it\"\nkeyboard_variant = \",intl\"");

    assert_eq!(
        l.keyboards(),
        vec![Layout::new("us", ""), Layout::new("it", "intl")]
    );
}

#[test]
fn the_variant_may_be_left_out() {
    assert_eq!(
        locale("keyboard_layout = [\"de\"]").keyboards(),
        vec![Layout::new("de", "")]
    );
}

#[test]
fn layouts_round_trip_through_toml() {
    let mut l = locale("keyboard_layout = \"us\"");
    l.set_keyboards(&[Layout::new("us", ""), Layout::new("it", "intl")]);

    let back: Locale = toml::from_str(&toml::to_string(&l).expect("write")).expect("read back");

    assert_eq!(back, l);
}
